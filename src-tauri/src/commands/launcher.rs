use crate::constants::{TOOL_CMD, TOOL_EXE};
use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

/// 脚本类可执行文件扩展名：无 PE 头、经 cmd 解释执行；
/// 既是允许录入/启动的脚本，也是合法的关闭脚本（扩展名规则的唯一来源）
pub const SCRIPT_EXTENSIONS: &[&str] = &["bat", "cmd"];

/// 允许录入/启动的可执行文件扩展名 = PE 程序（exe）+ 脚本（bat/cmd）
pub const PROGRAM_EXTENSIONS: &[&str] = &["exe", "bat", "cmd"];

/// 判断路径扩展名（忽略大小写）是否在给定白名单内
pub fn has_extension_in(path: &Path, exts: &[&str]) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .map(|s| exts.iter().any(|e| s.eq_ignore_ascii_case(e)))
        .unwrap_or(false)
}

#[tauri::command]
pub fn launch_tool(
    tool_type: String,
    path: String,
    args: Option<String>,
    admin: Option<bool>,
) -> Result<(), String> {
    match tool_type.as_str() {
        TOOL_CMD => open_cmd_window(&path),
        TOOL_EXE => launch_exe(&path, args.as_deref(), admin.unwrap_or(false)),
        other => Err(format!("未知工具类型: {}", other)),
    }
}

/// 在系统文件管理器中打开目录；若传入的是文件（exe），则打开其所在目录
#[tauri::command]
pub fn open_path(app: AppHandle, path: String) -> Result<(), String> {
    let p = Path::new(&path);
    let target: PathBuf = if p.is_file() {
        p.parent().map(|x| x.to_path_buf()).unwrap_or_else(|| p.to_path_buf())
    } else {
        p.to_path_buf()
    };

    if !target.exists() {
        return Err(format!("路径不存在: {}", target.display()));
    }

    app
        .opener()
        .open_path(target.to_string_lossy().to_string(), None::<&str>)
        .map_err(|e| format!("打开目录失败: {}", e))
}

// ---------- Windows ----------

#[cfg(windows)]
fn open_cmd_window(path: &str) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    if !Path::new(path).is_dir() {
        return Err(format!("目录不存在: {}", path));
    }

    // 经 `start` 启动：外层辅助进程不可见，内层 cmd 获得独立控制台，
    // 不继承父进程 stdin（否则管道 EOF 会导致窗口一闪而过）
    Command::new("cmd.exe")
        .args(["/C", "start", "", "/D", path, "cmd.exe", "/K"])
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("打开命令窗口失败: {}", e))
}

#[cfg(windows)]
fn launch_exe(exe_path: &str, args: Option<&str>, admin: bool) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    let p = Path::new(exe_path);
    if !p.is_file() {
        return Err(format!("exe 不存在: {}", exe_path));
    }

    let work_dir = p
        .parent()
        .map(|x| x.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    // 系统目录（如 C:\Windows\System32 下的 net.exe）不作为工作目录，
    // 避免普通权限下启动失败；此时退化为 exe 所在卷根目录
    let dir_str = work_dir.to_string_lossy().to_string();
    let sys_root = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".into());
    let sys_root = sys_root.trim_end_matches('\\');
    let is_system_dir = {
        let d = dir_str.trim_end_matches('\\');
        d.eq_ignore_ascii_case(sys_root)
            || d.eq_ignore_ascii_case(&format!(r"{}\System32", sys_root))
    };
    let dir = if is_system_dir {
        work_dir
            .ancestors()
            .last()
            .map(|x| x.to_string_lossy().to_string())
            .filter(|s| s.ends_with('\\'))
            .unwrap_or_else(|| sys_root.to_string())
    } else {
        dir_str
    };

    let extra = args.filter(|s| !s.trim().is_empty());
    // bat/cmd 脚本没有 PE 头，直接按控制台程序处理（经 cmd 解释执行）
    let is_script = has_extension_in(p, SCRIPT_EXTENSIONS);
    let console = is_script || is_console_exe(p);

    // 管理员启动：经 ShellExecuteEx 的 "runas" 谓词触发 UAC
    if admin {
        return launch_elevated(exe_path, extra, &dir, console);
    }

    // 控制台程序（如 net.exe）：打开可见 cmd 窗口执行，/K 使命令结束后窗口保留，
    // 避免输出（如“服务已启动”/“拒绝访问”）随进程退出一闪而过
    if console {
        // 外层再包一对引号：cmd /K 对首尾引号的剥离规则会把内层引号留给命令本身
        let line = match extra {
            Some(a) => format!("\"\"{}\" {}\"", exe_path, a),
            None => format!("\"\"{}\"\"", exe_path),
        };
        let mut c = Command::new("cmd.exe");
        c.arg("/K").current_dir(&dir);
        // 原样透传（用户参数可能自带引号），不能交给 Rust 自动转义
        c.raw_arg(line);
        return c
            .spawn()
            .map(|_| ())
            .map_err(|e| format!("启动失败: {}", e));
    }

    // 经由 cmd 的 start（ShellExecute 语义）启动：
    // GUI 程序不会附带控制台，控制台程序会自动获得新窗口
    let mut cmd = Command::new("cmd.exe");
    cmd.args(vec!["/C", "start", "", "/D", dir.as_str()])
        .arg(exe_path)
        .creation_flags(CREATE_NO_WINDOW);
    // 启动参数原样透传（其中可能自带引号，如 -c "code"），不能交给 Rust 自动转义
    if let Some(a) = extra {
        cmd.raw_arg(a);
    }
    cmd.spawn()
        .map(|_| ())
        .map_err(|e| format!("启动 exe 失败: {}", e))
}

/// 读 PE 头判断 exe 子系统：3 = 控制台程序（CUI），其余（GUI/未知）一律按 GUI 处理
#[cfg(windows)]
fn is_console_exe(path: &Path) -> bool {
    use std::io::{Read, Seek, SeekFrom};
    (|| -> std::io::Result<bool> {
        let mut f = std::fs::File::open(path)?;
        // DOS 头 0x3C 处是 PE 头偏移（e_lfanew）
        f.seek(SeekFrom::Start(0x3C))?;
        let mut b4 = [0u8; 4];
        f.read_exact(&mut b4)?;
        let pe_off = u32::from_le_bytes(b4) as u64;
        // Subsystem 字段 = PE 签名(4) + COFF 头(20) + 可选头内偏移 68（PE32/PE32+ 相同）
        f.seek(SeekFrom::Start(pe_off + 4 + 20 + 68))?;
        let mut b2 = [0u8; 2];
        f.read_exact(&mut b2)?;
        Ok(u16::from_le_bytes(b2) == 3)
    })()
    .unwrap_or(false)
}

/// 以管理员身份启动：ShellExecuteEx + "runas" 谓词触发 UAC 授权。
/// 控制台程序提权 cmd /K（窗口保留可看输出）；GUI 程序直接提权 exe 本身
#[cfg(windows)]
fn launch_elevated(
    exe_path: &str,
    args: Option<&str>,
    work_dir: &str,
    console: bool,
) -> Result<(), String> {
    use windows_sys::Win32::Foundation::GetLastError;
    use windows_sys::Win32::UI::Shell::{ShellExecuteExW, SHELLEXECUTEINFOW};
    use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    fn to_wide(s: &str) -> Vec<u16> {
        use std::os::windows::ffi::OsStrExt;
        std::ffi::OsStr::new(s)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    let (file, params) = if console {
        let line = match args {
            Some(a) => format!("/K \"\"{}\" {}\"", exe_path, a),
            None => format!("/K \"\"{}\"\"", exe_path),
        };
        ("cmd.exe".to_string(), line)
    } else {
        (exe_path.to_string(), args.unwrap_or("").to_string())
    };

    let verb = to_wide("runas");
    let file_w = to_wide(&file);
    let params_w = to_wide(&params);
    let dir_w = to_wide(work_dir);

    let mut info: SHELLEXECUTEINFOW = unsafe { std::mem::zeroed() };
    info.cbSize = std::mem::size_of::<SHELLEXECUTEINFOW>() as u32;
    info.lpVerb = verb.as_ptr();
    info.lpFile = file_w.as_ptr();
    info.lpParameters = if params.is_empty() {
        std::ptr::null()
    } else {
        params_w.as_ptr()
    };
    info.lpDirectory = dir_w.as_ptr();
    info.nShow = SW_SHOWNORMAL;

    let ok = unsafe { ShellExecuteExW(&mut info) };
    if ok == 0 {
        let err = unsafe { GetLastError() };
        // 1223 = ERROR_CANCELLED：用户在 UAC 弹窗点了“否”
        if err == 1223 {
            return Err("已取消管理员授权（UAC）".into());
        }
        return Err(format!("以管理员身份启动失败（错误码 {}）", err));
    }
    Ok(())
}

// ---------- 非 Windows 兜底 ----------

#[cfg(not(windows))]
fn open_cmd_window(path: &str) -> Result<(), String> {
    if !Path::new(path).is_dir() {
        return Err(format!("目录不存在: {}", path));
    }
    Err("当前仅支持 Windows 命令行窗口".into())
}

#[cfg(not(windows))]
fn launch_exe(exe_path: &str, args: Option<&str>, _admin: bool) -> Result<(), String> {
    let p = Path::new(exe_path);
    if !p.is_file() {
        return Err(format!("可执行文件不存在: {}", exe_path));
    }
    let work_dir = p.parent().unwrap_or_else(|| Path::new("."));
    match args.filter(|s| !s.trim().is_empty()) {
        Some(a) => Command::new("sh")
            .arg("-c")
            .arg(format!("\"{}\" {}", exe_path, a))
            .current_dir(work_dir)
            .spawn()
            .map(|_| ())
            .map_err(|e| format!("启动失败: {}", e)),
        None => Command::new(exe_path)
            .current_dir(work_dir)
            .spawn()
            .map(|_| ())
            .map_err(|e| format!("启动失败: {}", e)),
    }
}
