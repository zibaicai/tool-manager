use crate::commands::{config::ensure_config_dir, launcher, scanner};
use crate::constants::{strip_bom, EXE_TOOLS_FILE, ICON_FILE, OPS_FILE, README_FILE, TOOL_EXE};
use crate::models::Tool;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::AppHandle;

/// exe-tools.json 中一条手动录入记录
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExeToolEntry {
    pub id: String,
    pub exe_path: String,
    pub category_id: String,
    /// 启动参数，如 -c "from zenmapGUI.App import run;run()"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<String>,
    /// 自定义标题；为空时自动取 README.md H1 或 exe 文件名
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// 副标题：工具用途描述，展示在标题下方
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub desc: Option<String>,
    /// 以管理员身份启动（触发 UAC），如 net.exe 启动系统服务
    #[serde(default, skip_serializing_if = "is_false")]
    pub admin: bool,
    /// 关闭脚本（.bat/.cmd）绝对路径：用于启动后还需停止服务的工具；设置后卡片出现「关闭工具」按钮
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_path: Option<String>,
    /// 关闭脚本是否独立提权运行（不继承启动用 admin）
    #[serde(default, skip_serializing_if = "is_false")]
    pub stop_admin: bool,
}

fn is_false(b: &bool) -> bool {
    !*b
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ExeToolConfig {
    #[serde(default)]
    pub tools: Vec<ExeToolEntry>,
}

fn config_file(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(ensure_config_dir(app)?.join(EXE_TOOLS_FILE))
}

fn load_config(app: &AppHandle) -> Result<ExeToolConfig, String> {
    let cfg = fs::read_to_string(config_file(app)?)
        .map_err(|e| format!("读取 {} 失败: {}", EXE_TOOLS_FILE, e))?;
    // 容忍 UTF-8 BOM（部分编辑器/PowerShell 保存时会附加）
    serde_json::from_str(strip_bom(&cfg))
        .map_err(|e| format!("{} 解析失败: {}", EXE_TOOLS_FILE, e))
}

fn save_config(app: &AppHandle, cfg: &ExeToolConfig) -> Result<(), String> {
    let json = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;
    fs::write(config_file(app)?, json)
        .map_err(|e| format!("写入 {} 失败: {}", EXE_TOOLS_FILE, e))
}

/// 所有已录入 EXE 工具的所在目录（供文档读取范围校验等使用）；配置不可读时返回空
pub(crate) fn exe_tool_dirs(app: &AppHandle) -> Vec<PathBuf> {
    load_config(app)
        .map(|c| {
            c.tools
                .iter()
                .filter_map(|t| Path::new(&t.exe_path).parent().map(|p| p.to_path_buf()))
                .collect()
        })
        .unwrap_or_default()
}

/// 把录入记录补全为展示用 Tool：
/// 标题取 exe 同目录 README.md 的 H1，否则取 exe 文件名；
/// 文档取同目录 Ops.md；图标取同目录 icon.png
fn enrich(entry: &ExeToolEntry) -> Tool {
    let exe = Path::new(&entry.exe_path);
    let dir = exe.parent().unwrap_or_else(|| Path::new("."));

    let readme = dir.join(README_FILE);
    let fallback = exe
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "未命名工具".into());
    // 自定义标题优先，否则取 README.md H1，再退化为 exe 文件名
    let title = match &entry.title {
        Some(t) if !t.trim().is_empty() => t.trim().to_string(),
        _ if readme.exists() => scanner::extract_h1(&readme).unwrap_or(fallback),
        _ => fallback,
    };

    let ops = dir.join(OPS_FILE);
    let doc_path = ops
        .exists()
        .then(|| ops.to_string_lossy().to_string());

    let icon_file = dir.join(ICON_FILE);
    let icon = icon_file
        .exists()
        .then(|| icon_file.to_string_lossy().to_string());

    Tool {
        id: entry.id.clone(),
        tool_type: TOOL_EXE.into(),
        title,
        path: entry.exe_path.clone(),
        doc_path,
        icon,
        category_id: entry.category_id.clone(),
        available: exe.is_file(),
        args: entry.args.clone(),
        desc: entry.desc.clone(),
        admin: entry.admin,
        stop_path: entry.stop_path.clone(),
        stop_admin: entry.stop_admin,
    }
}

/// 把用户输入拆成「exe 路径 + 启动参数」。
/// 支持两种写法：
///   D:\tools\app.exe                      （无参数）
///   D:\tools\app.exe -c "code"            （裸路径 + 参数，按第一个可执行扩展名位置切分）
///   "D:\my tools\app.exe" -c "code"       （带引号路径 + 参数）
///   D:\tools\start-svc.bat                （bat/cmd 脚本同样支持）
fn split_command(input: &str) -> (String, Option<String>) {
    let input = input.trim();
    if let Some(rest) = input.strip_prefix('"') {
        if let Some(end) = rest.find('"') {
            let exe = rest[..end].to_string();
            let args = rest[end + 1..].trim();
            return (exe, (!args.is_empty()).then(|| args.to_string()));
        }
    }
    // 裸路径：按最先出现的可执行扩展名（忽略大小写）切分，兼容路径含空格的情况
    let lower = input.to_lowercase();
    let cut = launcher::PROGRAM_EXTENSIONS
        .iter()
        .filter_map(|ext| lower.find(&format!(".{ext}")).map(|p| (p, ext.len())))
        .min_by_key(|(p, _)| *p);
    match cut {
        Some((pos, ext_len)) => {
            let end = pos + 1 + ext_len;
            let exe = input[..end].trim().to_string();
            let args = input[end..].trim();
            (exe, (!args.is_empty()).then(|| args.to_string()))
        }
        None => (input.to_string(), None),
    }
}

#[tauri::command]
pub fn load_exe_tools(app: AppHandle) -> Result<Vec<Tool>, String> {
    Ok(load_config(&app)?.tools.iter().map(enrich).collect())
}

#[tauri::command]
pub fn add_exe_tool(
    app: AppHandle,
    exe_path: String,
    category_id: String,
    title: Option<String>,
    desc: Option<String>,
    admin: Option<bool>,
    stop_path: Option<String>,
    stop_admin: Option<bool>,
) -> Result<Tool, String> {
    if exe_path.trim().is_empty() {
        return Err("请填写 exe 的绝对路径或启动命令".into());
    }
    let title = title.filter(|t| !t.trim().is_empty()).map(|t| t.trim().to_string());
    let desc = desc.filter(|d| !d.trim().is_empty()).map(|d| d.trim().to_string());
    let stop_path = validate_stop_path(stop_path)?;
    let (exe_path, args) = split_command(&exe_path);
    let exe = Path::new(&exe_path);

    if !exe.is_absolute() {
        return Err("请输入绝对路径（如 D:\\tools\\xxx.exe，可附带启动参数）".into());
    }
    if !exe.is_file() {
        return Err(format!("文件不存在: {}", exe_path));
    }
    if !launcher::has_extension_in(exe, launcher::PROGRAM_EXTENSIONS) {
        return Err("请选择 .exe / .bat / .cmd 文件".into());
    }

    let mut cfg = load_config(&app)?;
    if cfg
        .tools
        .iter()
        .any(|t| same_path(&t.exe_path, &exe_path) && t.args == args)
    {
        return Err("该工具已录入".into());
    }

    let entry = ExeToolEntry {
        id: scanner::make_id(&format!(
            "{}|{}",
            exe_path,
            args.as_deref().unwrap_or("")
        )),
        exe_path,
        category_id,
        args,
        title,
        desc,
        admin: admin.unwrap_or(false),
        stop_path,
        stop_admin: stop_admin.unwrap_or(false),
    };
    cfg.tools.push(entry.clone());
    save_config(&app, &cfg)?;
    Ok(enrich(&entry))
}

/// 校验关闭脚本路径：非空时必须存在且为 bat/cmd；空则返回 None
fn validate_stop_path(stop_path: Option<String>) -> Result<Option<String>, String> {
    let Some(p) = stop_path.filter(|s| !s.trim().is_empty()) else {
        return Ok(None);
    };
    let p = p.trim().trim_matches('"').to_string();
    let path = Path::new(&p);
    if !path.is_absolute() || !path.is_file() {
        return Err(format!("关闭脚本不存在: {}", p));
    }
    if !launcher::has_extension_in(path, launcher::SCRIPT_EXTENSIONS) {
        return Err("关闭脚本请选择 .bat / .cmd 文件".into());
    }
    Ok(Some(p))
}

/// 更新已录入工具的标题/副标题/管理员启动标记/关闭脚本及其提权标记；标题传空字符串表示清除（恢复自动派生）
#[tauri::command]
pub fn update_exe_tool(
    app: AppHandle,
    id: String,
    title: Option<String>,
    desc: Option<String>,
    admin: Option<bool>,
    stop_path: Option<String>,
    stop_admin: Option<bool>,
) -> Result<Tool, String> {
    let title = title.filter(|t| !t.trim().is_empty()).map(|t| t.trim().to_string());
    let desc = desc.filter(|d| !d.trim().is_empty()).map(|d| d.trim().to_string());
    let stop_path = validate_stop_path(stop_path)?;

    let mut cfg = load_config(&app)?;
    let entry = cfg
        .tools
        .iter_mut()
        .find(|t| t.id == id)
        .ok_or("未找到该工具")?;
    entry.title = title;
    entry.desc = desc;
    entry.stop_path = stop_path;
    if let Some(a) = admin {
        entry.admin = a;
    }
    if let Some(a) = stop_admin {
        entry.stop_admin = a;
    }
    let updated = enrich(entry);
    save_config(&app, &cfg)?;
    Ok(updated)
}

#[tauri::command]
pub fn remove_exe_tool(app: AppHandle, id: String) -> Result<(), String> {
    let mut cfg = load_config(&app)?;
    let before = cfg.tools.len();
    cfg.tools.retain(|t| t.id != id);
    if cfg.tools.len() == before {
        return Err("未找到该工具".into());
    }
    save_config(&app, &cfg)
}

/// Windows 路径大小写不敏感的简单比较
fn same_path(a: &str, b: &str) -> bool {
    a.trim_end_matches('\\').eq_ignore_ascii_case(b.trim_end_matches('\\'))
}

