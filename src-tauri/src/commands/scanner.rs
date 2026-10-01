use crate::commands::config::ensure_config_dir;
use crate::models::Tool;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::AppHandle;

/// CMD 工具的标题/副标题自定义覆盖（扫描结果本身不落盘，修改存这里）
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CmdToolOverride {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub desc: Option<String>,
    /// 手动分配到的目录（分类）id；设置后覆盖扫描归属
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category_id: Option<String>,
    /// 分配时记录的工具目录路径，用于目标分类扫描时补入该工具
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dir_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CmdToolOverrides {
    #[serde(default)]
    pub tools: HashMap<String, CmdToolOverride>,
}

fn overrides_file(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(ensure_config_dir(app)?.join("cmd-tools.json"))
}

fn load_overrides(app: &AppHandle) -> CmdToolOverrides {
    fs::read_to_string(overrides_file(app).unwrap_or_default())
        .ok()
        .and_then(|c| serde_json::from_str(c.trim_start_matches('\u{feff}')).ok())
        .unwrap_or_default()
}

fn save_overrides(app: &AppHandle, o: &CmdToolOverrides) -> Result<(), String> {
    let json = serde_json::to_string_pretty(o).map_err(|e| e.to_string())?;
    fs::write(overrides_file(app)?, json).map_err(|e| format!("写入 cmd-tools.json 失败: {}", e))
}

/// 把自定义覆盖应用到扫描出的工具上
fn apply_override(tool: &mut Tool, overrides: &CmdToolOverrides) {
    if let Some(ov) = overrides.tools.get(&tool.id) {
        if let Some(t) = &ov.title {
            if !t.trim().is_empty() {
                tool.title = t.trim().to_string();
            }
        }
        tool.desc = ov.desc.clone();
        if let Some(cid) = &ov.category_id {
            tool.category_id = cid.clone();
        }
    }
}

/// 从 Markdown 中提取第一行 H1（# 开头）
pub(crate) fn extract_h1(md_path: &Path) -> Option<String> {
    let content = fs::read_to_string(md_path).ok()?;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("# ") {
            return Some(trimmed[2..].trim().to_string());
        }
    }
    None
}

/// 生成稳定的 id（简单用路径 hash）
pub(crate) fn make_id(path: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    path.hash(&mut h);
    format!("{:x}", h.finish())
}

/// 解析单个工具目录：标题取 README.md 首个 H1，检测 Ops.md / icon.png
fn inspect_dir(dir: &Path, category_id: &str) -> Option<Tool> {
    if !dir.is_dir() {
        return None;
    }

    let dir_name = dir
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    // 标题：优先 README.md 的 H1，否则用目录名
    let readme = dir.join("README.md");
    let title = if readme.exists() {
        extract_h1(&readme).unwrap_or(dir_name.clone())
    } else {
        dir_name
    };

    // 文档：Ops.md
    let ops = dir.join("Ops.md");
    let doc_path = if ops.exists() {
        Some(ops.to_string_lossy().to_string())
    } else {
        None
    };

    // 图标：icon.png
    let icon_file = dir.join("icon.png");
    let icon = if icon_file.exists() {
        Some(icon_file.to_string_lossy().to_string())
    } else {
        None
    };

    let path_str = dir.to_string_lossy().to_string();
    Some(Tool {
        id: make_id(&path_str),
        tool_type: "cmd".into(),
        title,
        path: path_str,
        doc_path,
        icon,
        category_id: category_id.to_string(),
        available: true,
        args: None,
        desc: None,
        admin: false,
        stop_path: None,
    })
}

#[tauri::command]
pub fn scan_cmd_tools(
    app: AppHandle,
    path: String,
    category_id: String,
    dirs: Option<Vec<String>>,
) -> Result<Vec<Tool>, String> {
    let root = PathBuf::from(&path);
    if !root.exists() {
        return Err(format!("扫描目录不存在: {}", path));
    }
    if !root.is_dir() {
        return Err(format!("不是目录: {}", path));
    }

    let mut tools = Vec::new();

    // 仅收集 dirs 名单中的一级子目录（按填写顺序展示）。
    // 空名单表示目录为空，工具通过手动分配（cmd-tools.json）补入，新建目录初始为空。
    if let Some(names) = dirs {
        for name in names {
            if let Some(tool) = inspect_dir(&root.join(name.trim()), &category_id) {
                tools.push(tool);
            }
        }
    }

    let overrides = load_overrides(&app);

    // 手动分配到其他分类的工具从本分类剔除（目标分类仍存在才剔除，避免删分类后工具消失）
    let menu = crate::commands::config::load_menu_config(app.clone()).ok();
    tools.retain(|t| {
        match overrides
            .tools
            .get(&t.id)
            .and_then(|o| o.category_id.as_ref())
        {
            Some(cid) if *cid != category_id => !menu
                .as_ref()
                .map_or(false, |m| m.categories.iter().any(|c| &c.id == cid)),
            _ => true,
        }
    });

    // 手动分配到本分类但未被扫描规则覆盖的工具（如目标分类配了 dirs 名单），按记录的目录路径补入
    for (id, ov) in &overrides.tools {
        if ov.category_id.as_deref() == Some(category_id.as_str())
            && !tools.iter().any(|t| t.id == *id)
        {
            if let Some(dp) = &ov.dir_path {
                if let Some(tool) = inspect_dir(Path::new(dp), &category_id) {
                    tools.push(tool);
                }
            }
        }
    }

    // 兜底：未被任何 CMD 目录取走的孤儿工具，统一挂到第一个 CMD（scan）目录下
    if let Some(menu) = &menu {
        let first_scan_id = menu
            .categories
            .iter()
            .find(|c| c.category_type == "scan")
            .map(|c| c.id.as_str());
        if first_scan_id == Some(category_id.as_str()) {
            if let Some(scan_root) = &menu.scan_root {
                use std::collections::HashSet;
                // 已认领 = 所有 scan 目录 dirs 名单的并集
                let mut claimed: HashSet<String> = HashSet::new();
                for c in &menu.categories {
                    if c.category_type == "scan" {
                        for d in &c.dirs {
                            claimed.insert(d.trim().to_lowercase());
                        }
                    }
                }
                // 手动分配到现存目录的工具也算已认领
                for ov in overrides.tools.values() {
                    if let (Some(cid), Some(dp)) = (&ov.category_id, &ov.dir_path) {
                        if menu.categories.iter().any(|c| &c.id == cid) {
                            if let Some(name) = Path::new(dp).file_name() {
                                claimed.insert(name.to_string_lossy().to_lowercase());
                            }
                        }
                    }
                }
                if let Ok(entries) = fs::read_dir(scan_root) {
                    let mut orphans: Vec<String> = Vec::new();
                    for entry in entries.flatten() {
                        let p = entry.path();
                        if !p.is_dir() {
                            continue;
                        }
                        let Some(fname) = p.file_name() else { continue };
                        let name = fname.to_string_lossy().to_string();
                        if !claimed.contains(&name.to_lowercase()) {
                            orphans.push(name);
                        }
                    }
                    orphans.sort();
                    for name in orphans {
                        let dp = Path::new(scan_root).join(&name);
                        if let Some(tool) = inspect_dir(&dp, &category_id) {
                            if !tools.iter().any(|t| t.id == tool.id) {
                                tools.push(tool);
                            }
                        }
                    }
                }
            }
        }
    }

    for t in &mut tools {
        apply_override(t, &overrides);
    }

    Ok(tools)
}

/// 手动分配 CMD 工具到指定目录（分类）。category_id 传 None 表示清除分配、恢复自动归属。
/// 要求工具目录位于目标分类的扫描根路径下，否则拒绝。
#[tauri::command]
pub fn assign_cmd_tool(
    app: AppHandle,
    id: String,
    path: String,
    category_id: Option<String>,
) -> Result<Tool, String> {
    let mut overrides = load_overrides(&app);
    let mut ov = overrides.tools.get(&id).cloned().unwrap_or_default();

    match &category_id {
        Some(cid) => {
            let menu = crate::commands::config::load_menu_config(app.clone())?;
            let cat = menu
                .categories
                .iter()
                .find(|c| &c.id == cid)
                .ok_or("未找到目标目录")?;
            if cat.category_type != "scan" {
                return Err("目标目录不是 CMD 自动扫描类型".into());
            }
            let root = cat
                .scan_path
                .clone()
                .or(menu.scan_root.clone())
                .ok_or("目标目录未配置扫描根路径")?;
            let norm = |p: &str| p.trim_end_matches(['\\', '/']).to_lowercase();
            let parent = Path::new(&path)
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .ok_or("工具路径无效")?;
            if norm(&parent) != norm(&root) {
                return Err(format!(
                    "该工具不在目标目录的扫描根（{}）下，无法分配",
                    root
                ));
            }
            ov.category_id = Some(cid.clone());
            ov.dir_path = Some(path.clone());
        }
        None => {
            ov.category_id = None;
            ov.dir_path = None;
        }
    }

    if ov.title.is_none() && ov.desc.is_none() && ov.category_id.is_none() {
        overrides.tools.remove(&id);
    } else {
        overrides.tools.insert(id.clone(), ov);
    }
    save_overrides(&app, &overrides)?;

    let view_cat = category_id.clone().unwrap_or_default();
    let mut tool =
        inspect_dir(Path::new(&path), &view_cat).ok_or(format!("目录不存在: {}", path))?;
    apply_override(&mut tool, &overrides);
    Ok(tool)
}

/// 更新 CMD 工具的标题/副标题（传空表示清除，标题恢复自动派生）。
/// 需要传入工具的 path/categoryId 以便重新派生并返回最新 Tool。
#[tauri::command]
pub fn update_cmd_tool(
    app: AppHandle,
    id: String,
    path: String,
    category_id: String,
    title: Option<String>,
    desc: Option<String>,
) -> Result<Tool, String> {
    let title = title.filter(|t| !t.trim().is_empty()).map(|t| t.trim().to_string());
    let desc = desc.filter(|d| !d.trim().is_empty()).map(|d| d.trim().to_string());

    let mut overrides = load_overrides(&app);
    let mut ov = overrides.tools.get(&id).cloned().unwrap_or_default();
    ov.title = title;
    ov.desc = desc;
    // 保留已有的手动分配（category_id/dir_path）
    if ov.title.is_none() && ov.desc.is_none() && ov.category_id.is_none() {
        overrides.tools.remove(&id);
    } else {
        overrides.tools.insert(id.clone(), ov);
    }
    save_overrides(&app, &overrides)?;

    let mut tool =
        inspect_dir(Path::new(&path), &category_id).ok_or(format!("目录不存在: {}", path))?;
    apply_override(&mut tool, &overrides);
    Ok(tool)
}

#[tauri::command]
pub fn read_text_file(path: String) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|e| format!("读取失败 {}: {}", path, e))
}
