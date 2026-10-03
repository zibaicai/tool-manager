use crate::commands::scanner;
use crate::constants::{
    strip_bom, CAT_MANUAL, CAT_SCAN, CATEGORY_ID_PREFIX, CONFIG_DIR, DEFAULT_WEIGHT,
    EXE_TOOLS_FILE, MENU_FILE,
};
use crate::models::{Category, MenuConfig};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

/// 首次启动且无任何旧配置可迁移时的内置模板（不含任何机器相关路径）
const DEFAULT_MENU_JSON: &str = r#"{
  "scanRoot": null,
  "categories": []
}
"#;

const DEFAULT_EXE_TOOLS_JSON: &str = r#"{
  "tools": []
}
"#;

/// 从当前 exe 所在目录向上逐级查找 config/menu.json，
/// 用于便携模式判断、旧版配置迁移、开发期定位项目配置
fn find_tree_config_dir() -> Option<PathBuf> {
    let mut dir = std::env::current_exe().ok()?.parent()?.to_path_buf();
    for _ in 0..8 {
        if dir.join(CONFIG_DIR).join(MENU_FILE).exists() {
            return Some(dir.join(CONFIG_DIR));
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

/// 解析配置目录，并保证其中的配置文件可用。
///
/// 优先级：
/// 1. 便携模式：exe 同目录 config/menu.json 存在（绿色版 / U 盘）
/// 2. 开发模式：debug 构建时使用项目树内 config/（改配置即时生效）
/// 3. 用户级目录：%APPDATA%/<identifier>/，与 exe 和项目所在位置完全无关；
///    首次运行时从项目 config/ 迁移旧配置，否则写入内置模板
pub(crate) fn ensure_config_dir(app: &AppHandle) -> Result<PathBuf, String> {
    // 1. 便携模式
    if let Some(exe_dir) = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
    {
        let portable = exe_dir.join(CONFIG_DIR);
        if portable.join(MENU_FILE).exists() {
            return Ok(portable);
        }
    }

    // 2. 开发模式：debug 构建直接读写项目配置
    #[cfg(debug_assertions)]
    if let Some(dir) = find_tree_config_dir() {
        return Ok(dir);
    }

    // 3. 用户级配置目录
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("获取配置目录失败: {}", e))?;
    fs::create_dir_all(&dir).map_err(|e| format!("创建配置目录失败: {}", e))?;

    let menu_path = dir.join(MENU_FILE);
    if !menu_path.exists() {
        // 首次运行：优先从项目树 config/ 迁移
        let mut migrated = false;
        if let Some(src) = find_tree_config_dir() {
            if fs::copy(src.join(MENU_FILE), &menu_path).is_ok() {
                migrated = true;
                let exe_src = src.join(EXE_TOOLS_FILE);
                if exe_src.exists() {
                    let _ = fs::copy(exe_src, dir.join(EXE_TOOLS_FILE));
                }
            }
        }
        if !migrated {
            fs::write(&menu_path, DEFAULT_MENU_JSON)
                .map_err(|e| format!("初始化 {} 失败: {}", MENU_FILE, e))?;
        }
    }

    let exe_tools_path = dir.join(EXE_TOOLS_FILE);
    if !exe_tools_path.exists() {
        fs::write(&exe_tools_path, DEFAULT_EXE_TOOLS_JSON)
            .map_err(|e| format!("初始化 {} 失败: {}", EXE_TOOLS_FILE, e))?;
    }

    Ok(dir)
}

#[tauri::command]
pub fn load_menu_config(app: AppHandle) -> Result<MenuConfig, String> {
    load_menu(&app)
}

fn load_menu(app: &AppHandle) -> Result<MenuConfig, String> {
    let p = ensure_config_dir(app)?.join(MENU_FILE);
    let content = fs::read_to_string(&p)
        .map_err(|e| format!("读取 {} 失败: {} ({})", MENU_FILE, e, p.display()))?;
    // 容忍 UTF-8 BOM（部分编辑器/PowerShell 保存时会附加）
    serde_json::from_str(strip_bom(&content))
        .map_err(|e| format!("{} 解析失败: {}", MENU_FILE, e))
}

fn save_menu(app: &AppHandle, cfg: &MenuConfig) -> Result<(), String> {
    let p = ensure_config_dir(app)?.join(MENU_FILE);
    let json = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;
    fs::write(&p, json).map_err(|e| format!("写入 {} 失败: {}", MENU_FILE, e))
}

/// 新增目录（分类）。category_type 为 scan（自动扫描）或 manual（EXE 手动录入）；
/// weight 为排序权重，越大越靠前，同权重按创建先后排列
#[tauri::command]
pub fn add_category(
    app: AppHandle,
    name: String,
    category_type: String,
    weight: Option<i32>,
) -> Result<Category, String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("请填写目录名称".into());
    }
    if category_type != CAT_SCAN && category_type != CAT_MANUAL {
        return Err("目录类型需为 scan 或 manual".into());
    }
    let mut cfg = load_menu(&app)?;
    if cfg.categories.iter().any(|c| c.name == name) {
        return Err("已存在同名目录".into());
    }
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let cat = Category {
        id: format!("{}{}", CATEGORY_ID_PREFIX, scanner::make_id(&format!("{}-{}", name, millis))),
        name,
        category_type,
        scan_path: None,
        dirs: vec![],
        weight: weight.unwrap_or(DEFAULT_WEIGHT),
    };
    cfg.categories.push(cat.clone());
    save_menu(&app, &cfg)?;
    Ok(cat)
}

/// 重命名目录并可同时调整排序权重与分类级扫描目录（id 不变，不影响其下工具关联）。
/// scan_path 为 Some 时更新：空串表示清除（回退到顶层 scanRoot）；
/// 与顶层 scanRoot 一样不校验目录是否存在，避免拦截尚未挂载的盘
#[tauri::command]
pub fn rename_category(
    app: AppHandle,
    id: String,
    name: String,
    weight: Option<i32>,
    scan_path: Option<String>,
) -> Result<(), String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("请填写目录名称".into());
    }
    let mut cfg = load_menu(&app)?;
    let cat = cfg
        .categories
        .iter_mut()
        .find(|c| c.id == id)
        .ok_or("未找到该目录")?;
    cat.name = name;
    if let Some(w) = weight {
        cat.weight = w;
    }
    if let Some(p) = scan_path {
        cat.scan_path = Some(p.trim().to_string()).filter(|p| !p.is_empty());
    }
    save_menu(&app, &cfg)
}

/// 设置顶层 CMD 工具扫描根目录（scanRoot）。
/// 传 None 或空串/纯空白表示清除；保存时不校验目录是否存在，避免拦截尚未挂载的盘
#[tauri::command]
pub fn set_scan_root(app: AppHandle, path: Option<String>) -> Result<(), String> {
    let mut cfg = load_menu(&app)?;
    cfg.scan_root = path
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty());
    save_menu(&app, &cfg)
}

/// 删除目录。其下 EXE 录入会变为无分类数据（保留在 exe-tools.json，重新建同 id 目录可恢复）
#[tauri::command]
pub fn delete_category(app: AppHandle, id: String) -> Result<(), String> {
    let mut cfg = load_menu(&app)?;
    let before = cfg.categories.len();
    cfg.categories.retain(|c| c.id != id);
    if cfg.categories.len() == before {
        return Err("未找到该目录".into());
    }
    save_menu(&app, &cfg)
}
