use crate::commands::config::ensure_config_dir;
use crate::models::{MenuConfig, Tool};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::AppHandle;

/// 覆盖数据结构版本：
/// v1（旧）：CMD 工具 id = 工具目录绝对路径哈希，scanRoot 整体搬迁后 id 改变、自定义失联；
/// v2：id = 相对扫描根的相对路径哈希（当前为一级目录名），与所在盘符/父目录解耦，
///     迁移能处理"原地升级"与带 dir_path 的 A 类记录；
/// v3：追加"先搬家后升级"B 类记录的标题兜底认领（旧绝对路径哈希已无法路径对照，
///     改用自定义标题与目录名归一化匹配）
const OVERRIDES_VERSION: u32 = 3;

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
    /// 分配时记录的工具目录路径，用于目标分类扫描时补入该工具，也作为搬迁后重定位的依据
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dir_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CmdToolOverrides {
    /// 结构版本号，旧文件无此字段时按 0 处理并触发一次性迁移
    #[serde(default)]
    pub version: u32,
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

/// 汇总菜单中全部 CMD 扫描根：顶层 scanRoot + 各分类自带的 scan_path
fn collect_roots(menu: &MenuConfig) -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = menu
        .scan_root
        .iter()
        .map(PathBuf::from)
        .chain(menu.categories.iter().filter_map(|c| c.scan_path.as_ref().map(PathBuf::from)))
        .collect();
    roots.dedup();
    roots
}

/// CMD 工具的稳定标识 key：相对任一扫描根的相对路径（当前模型工具恒为一级子目录，
/// 即目录名），分隔符统一为 `/`。整体搬迁 scanRoot（换盘/换父目录）后 key 不变；
/// 匹配不到任何扫描根时退化为目录名，效果与一级相对路径一致
fn stable_key(dir: &Path, roots: &[PathBuf]) -> String {
    let rel = roots
        .iter()
        .filter_map(|r| dir.strip_prefix(r).ok())
        .min_by_key(|p| p.as_os_str().len())
        .map(|p| p.to_string_lossy().replace('\\', "/"));
    rel.unwrap_or_else(|| {
        dir.file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default()
    })
}

/// 解析单个工具目录：标题取 README.md 首个 H1，检测 Ops.md / icon.png。
/// id 基于相对扫描根的稳定 key（roots 为空时退化为目录名），不随 scanRoot 搬迁而变
fn inspect_dir(dir: &Path, category_id: &str, roots: &[PathBuf]) -> Option<Tool> {
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
        id: make_id(&stable_key(dir, roots)),
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

/// v1 → v2 一次性迁移：把以"绝对路径哈希"为键的覆盖记录改写为以"相对扫描根 key 哈希"为键。
/// 两类旧记录：
/// A. 带 dir_path（手动分配）：按目录名在当前扫描根下重定位，更新 dir_path，搬家/未搬均可迁；
/// B. 仅标题/副标题（无 dir_path）：哈希不可逆，只能枚举当前扫描根下的目录，
///    用旧规则（当前绝对路径哈希）精确对照认领——原地升级可救，跨 scanRoot 搬迁后才升级则无法认领。
/// 无法认领的记录原样保留（不再生效，但不破坏数据）。
/// 返回 true 表示当前存在扫描根、迁移流程已执行完毕（可把版本号升到 v2，幂等）
fn migrate_legacy_overrides(menu: &MenuConfig, overrides: &mut CmdToolOverrides) -> bool {
    let roots = collect_roots(menu);
    if roots.is_empty() {
        return false;
    }

    // 旧绝对路径 id → 新稳定 id 的对照表（枚举当前扫描根一级子目录）
    let mut abs_to_stable: HashMap<String, String> = HashMap::new();
    for root in &roots {
        if let Ok(entries) = fs::read_dir(root) {
            for entry in entries.flatten() {
                let p = entry.path();
                if !p.is_dir() {
                    continue;
                }
                let old_id = make_id(&p.to_string_lossy());
                let new_id = make_id(&stable_key(&p, &roots));
                abs_to_stable.insert(old_id, new_id);
            }
        }
    }

    let old_map = std::mem::take(&mut overrides.tools);
    for (old_id, mut ov) in old_map {
        let new_id = match ov.dir_path.clone() {
            // A 类：以目录名在任一扫描根下重定位；找不到现存目录也按目录名生成新 id
            Some(dp) => {
                let old_path = Path::new(&dp);
                match old_path.file_name().map(|f| f.to_owned()) {
                    Some(fname) => {
                        let relocated = roots
                            .iter()
                            .map(|r| r.join(&fname))
                            .find(|p| p.is_dir());
                        let new_id = make_id(&fname.to_string_lossy());
                        if let Some(rel) = relocated {
                            ov.dir_path = Some(rel.to_string_lossy().to_string());
                        }
                        Some(new_id)
                    }
                    None => Some(old_id.clone()),
                }
            }
            // B 类：靠当前根枚举对照
            None => abs_to_stable.get(&old_id).cloned(),
        };

        match new_id {
            Some(nid) if nid != old_id => {
                overrides
                    .tools
                    .entry(nid)
                    .and_modify(|existing| {
                        // 目标键已有记录时，仅补空字段，不覆盖已有内容
                        merge_override(existing, ov.clone())
                    })
                    .or_insert(ov);
            }
            _ => {
                // 无需迁移或无法认领：原样放回
                overrides.tools.insert(old_id, ov);
            }
        }
    }
    true
}

/// 合并两条覆盖记录：仅补空字段，不覆盖目标已有内容（用于多来源迁到同一新 id 时）
fn merge_override(existing: &mut CmdToolOverride, incoming: CmdToolOverride) {
    if existing.title.is_none() {
        existing.title = incoming.title;
    }
    if existing.desc.is_none() {
        existing.desc = incoming.desc;
    }
    if existing.category_id.is_none() {
        existing.category_id = incoming.category_id;
    }
    if existing.dir_path.is_none() {
        existing.dir_path = incoming.dir_path;
    }
}

/// 名称归一化：小写、按非字母数字切段并拼接、剥离常见版本库后缀 token。
/// 例：JSFinder-master → jsfinder；URLFinder → urlfinder
fn normalize_tool_name(s: &str) -> String {
    const STRIPPED_TOKENS: &[&str] = &["master", "main", "dev"];
    s.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty() && !STRIPPED_TOKENS.contains(t))
        .collect()
}

/// v2 → v3 补救：把"先搬家、后升级"导致无法路径对照的 B 类残留记录，
/// 按"自定义标题 ↔ 目录名"归一化后认领到现存目录的新稳定 id，并补写 dir_path。
/// 只认领导向唯一者（归一化名对应恰好一个现存目录）；键已是现存新 id 的正常记录不动；
/// 匹配不到的（如工具已删除）原样保留。
fn adopt_orphans_by_title(menu: &MenuConfig, overrides: &mut CmdToolOverrides) {
    let roots = collect_roots(menu);
    if roots.is_empty() {
        return;
    }

    // 现存目录：归一名 → (新id, 绝对路径)，同名多目录记为歧义、放弃认领
    let mut unique: HashMap<String, (String, String)> = HashMap::new();
    let mut ambiguous: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut live_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
    for root in &roots {
        if let Ok(entries) = fs::read_dir(root) {
            for entry in entries.flatten() {
                let p = entry.path();
                if !p.is_dir() {
                    continue;
                }
                let Some(fname) = p.file_name() else { continue };
                let fname = fname.to_string_lossy().to_string();
                let new_id = make_id(&stable_key(&p, &roots));
                live_ids.insert(new_id.clone());
                let norm = normalize_tool_name(&fname);
                if norm.is_empty() || ambiguous.contains(&norm) {
                    continue;
                }
                if unique.contains_key(&norm) {
                    unique.remove(&norm);
                    ambiguous.insert(norm);
                } else {
                    unique.insert(norm, (new_id, p.to_string_lossy().to_string()));
                }
            }
        }
    }

    let old_map = std::mem::take(&mut overrides.tools);
    for (key, mut ov) in old_map {
        // 已是现存工具的新 id、或根本没有自定义标题可作线索：不动
        if live_ids.contains(&key) || ov.title.is_none() {
            overrides.tools.insert(key, ov);
            continue;
        }
        let norm = normalize_tool_name(ov.title.as_deref().unwrap_or_default());
        if let Some((new_id, abs_path)) = unique.get(&norm) {
            ov.dir_path.get_or_insert_with(|| abs_path.clone());
            overrides
                .tools
                .entry(new_id.clone())
                .and_modify(|existing| merge_override(existing, ov.clone()))
                .or_insert(ov);
        } else {
            overrides.tools.insert(key, ov);
        }
    }
}

/// 读取覆盖数据；对旧版本逐级执行一次性迁移并落盘（v1→v2 路径对照，v2→v3 标题兜底）
fn load_overrides_migrated(
    app: &AppHandle,
    menu: Option<&MenuConfig>,
) -> CmdToolOverrides {
    let mut overrides = load_overrides(app);
    let Some(menu) = menu else { return overrides };
    let mut changed = false;

    if overrides.version < 2 {
        if migrate_legacy_overrides(menu, &mut overrides) {
            overrides.version = 2;
            changed = true;
        }
    }
    // 仅在已具备 v2 结构后才做标题兜底（v1 迁移因无扫描根而未完成时，等下次再试）
    if overrides.version >= 2 && overrides.version < OVERRIDES_VERSION {
        adopt_orphans_by_title(menu, &mut overrides);
        overrides.version = OVERRIDES_VERSION;
        changed = true;
    }

    if changed {
        let _ = save_overrides(app, &overrides);
    }
    overrides
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

    // 菜单提前加载：用于 id 稳定 key 计算与旧覆盖迁移
    let menu = crate::commands::config::load_menu_config(app.clone()).ok();
    let roots = menu.as_ref().map(collect_roots).unwrap_or_default();
    let overrides = load_overrides_migrated(&app, menu.as_ref());

    let mut tools = Vec::new();

    // 仅收集 dirs 名单中的一级子目录（按填写顺序展示）。
    // 空名单表示目录为空，工具通过手动分配（cmd-tools.json）补入，新建目录初始为空。
    if let Some(names) = dirs {
        for name in names {
            if let Some(tool) = inspect_dir(&root.join(name.trim()), &category_id, &roots) {
                tools.push(tool);
            }
        }
    }

    // 手动分配到其他分类的工具从本分类剔除（目标分类仍存在才剔除，避免删分类后工具消失）
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
                if let Some(tool) = inspect_dir(Path::new(dp), &category_id, &roots) {
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
                        if let Some(tool) = inspect_dir(&dp, &category_id, &roots) {
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
    // 先确保旧版本覆盖已迁移到稳定 id（此处 menu 仅用于迁移，业务校验仍单独加载以保留错误信息）
    let menu_opt = crate::commands::config::load_menu_config(app.clone()).ok();
    let roots = menu_opt.as_ref().map(collect_roots).unwrap_or_default();
    let mut overrides = load_overrides_migrated(&app, menu_opt.as_ref());
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
    let mut tool = inspect_dir(Path::new(&path), &view_cat, &roots)
        .ok_or(format!("目录不存在: {}", path))?;
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

    let menu_opt = crate::commands::config::load_menu_config(app.clone()).ok();
    let roots = menu_opt.as_ref().map(collect_roots).unwrap_or_default();
    let mut overrides = load_overrides_migrated(&app, menu_opt.as_ref());
    let mut ov = overrides.tools.get(&id).cloned().unwrap_or_default();
    ov.title = title;
    ov.desc = desc;
    // 补记工具目录绝对路径，使仅改过标题的记录在将来 scanRoot 搬迁时也能按目录名迁移
    ov.dir_path.get_or_insert_with(|| path.clone());
    // 保留已有的手动分配（category_id）；标题/副标题/分配全空时整条删除（dir_path 不单独构成记录）
    if ov.title.is_none() && ov.desc.is_none() && ov.category_id.is_none() {
        overrides.tools.remove(&id);
    } else {
        overrides.tools.insert(id.clone(), ov);
    }
    save_overrides(&app, &overrides)?;

    let mut tool = inspect_dir(Path::new(&path), &category_id, &roots)
        .ok_or(format!("目录不存在: {}", path))?;
    apply_override(&mut tool, &overrides);
    Ok(tool)
}

#[tauri::command]
pub fn read_text_file(path: String) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|e| format!("读取失败 {}: {}", path, e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Category, MenuConfig};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(tag: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let d = std::env::temp_dir().join(format!("tm_migrate_{}_{}", tag, nanos));
        fs::create_dir_all(&d).unwrap();
        d
    }

    fn menu_with_root(root: &Path) -> MenuConfig {
        MenuConfig {
            categories: vec![Category {
                id: "c1".into(),
                name: "测试目录".into(),
                category_type: "scan".into(),
                scan_path: None,
                dirs: vec![],
                weight: 0,
            }],
            scan_root: Some(root.to_string_lossy().to_string()),
        }
    }

    /// 原地升级场景：A 类（带 dir_path）与 B 类（纯标题）均应迁到稳定 id，且迁移幂等
    #[test]
    fn v1_to_v2_migrates_both_kinds_and_is_idempotent() {
        let root = temp_dir("normal");
        fs::create_dir_all(root.join("nuclei")).unwrap();
        fs::create_dir_all(root.join("JSFinder-master")).unwrap();

        let nuclei_abs = root.join("nuclei").to_string_lossy().to_string();
        let js_abs = root.join("JSFinder-master").to_string_lossy().to_string();
        let menu = menu_with_root(&root);

        // 旧版覆盖：键 = 工具目录绝对路径哈希
        let mut ov = CmdToolOverrides::default();
        ov.version = 0;
        ov.tools.insert(
            make_id(&nuclei_abs),
            CmdToolOverride {
                title: Some("nuclei".into()),
                desc: None,
                category_id: Some("framework-scan".into()),
                dir_path: Some(nuclei_abs.clone()),
            },
        );
        ov.tools.insert(
            make_id(&js_abs),
            CmdToolOverride {
                title: Some("JSFinder".into()),
                desc: Some("666".into()),
                category_id: None,
                dir_path: None,
            },
        );

        assert!(migrate_legacy_overrides(&menu, &mut ov));

        // A 类：新键为目录名哈希，分配关系保留，dir_path 重定位
        let a = ov
            .tools
            .get(&make_id("nuclei"))
            .expect("A 类应迁移到目录名 id");
        assert_eq!(a.category_id.as_deref(), Some("framework-scan"));
        assert_eq!(a.dir_path.as_deref(), Some(nuclei_abs.as_str()));
        // B 类：枚举当前根、用旧绝对路径 id 精确认领
        let b = ov
            .tools
            .get(&make_id("JSFinder-master"))
            .expect("B 类应被枚举认领");
        assert_eq!(b.title.as_deref(), Some("JSFinder"));
        assert_eq!(b.desc.as_deref(), Some("666"));
        // 旧键全部消失
        assert_eq!(ov.tools.len(), 2);
        assert!(ov.tools.get(&make_id(&nuclei_abs)).is_none());
        assert!(ov.tools.get(&make_id(&js_abs)).is_none());

        // 幂等：再跑一次结果不变
        assert!(migrate_legacy_overrides(&menu, &mut ov));
        assert_eq!(ov.tools.len(), 2);
        assert!(ov.tools.contains_key(&make_id("nuclei")));
        assert!(ov.tools.contains_key(&make_id("JSFinder-master")));

        let _ = fs::remove_dir_all(&root);
    }

    /// scanRoot 先搬迁、后升级：A 类按目录名仍可迁移并重定位 dir_path；
    /// B 类旧绝对路径 id 无法对照，原样保留不破坏
    #[test]
    fn migration_after_relocation() {
        let old_root = temp_dir("old");
        let new_root = temp_dir("new");
        fs::create_dir_all(old_root.join("nuclei")).unwrap();
        fs::create_dir_all(new_root.join("nuclei")).unwrap();
        let old_nuclei = old_root.join("nuclei").to_string_lossy().to_string();
        let new_nuclei = new_root.join("nuclei").to_string_lossy().to_string();

        let menu = menu_with_root(&new_root);
        let mut ov = CmdToolOverrides::default();
        ov.tools.insert(
            make_id(&old_nuclei),
            CmdToolOverride {
                title: None,
                desc: None,
                category_id: Some("c1".into()),
                dir_path: Some(old_nuclei.clone()),
            },
        );
        let orphan_old_id = make_id(&old_root.join("URLFinder").to_string_lossy());
        ov.tools.insert(
            orphan_old_id.clone(),
            CmdToolOverride {
                title: Some("URLFinder".into()),
                ..Default::default()
            },
        );

        assert!(migrate_legacy_overrides(&menu, &mut ov));
        let a = ov
            .tools
            .get(&make_id("nuclei"))
            .expect("A 类搬家后仍应按目录名迁移");
        assert_eq!(a.dir_path.as_deref(), Some(new_nuclei.as_str()));
        assert!(
            ov.tools.contains_key(&orphan_old_id),
            "无法认领的 B 类应原样保留"
        );
        assert_eq!(ov.tools.len(), 2);

        let _ = fs::remove_dir_all(&old_root);
        let _ = fs::remove_dir_all(&new_root);
    }

    /// 未配置任何扫描根时不迁移、不升版本，等配置后下次再试
    #[test]
    fn migration_skipped_without_root() {
        let menu = MenuConfig {
            categories: vec![],
            scan_root: None,
        };
        let mut ov = CmdToolOverrides::default();
        ov.version = 0;
        ov.tools.insert(
            "abc".into(),
            CmdToolOverride {
                title: Some("x".into()),
                ..Default::default()
            },
        );
        assert!(!migrate_legacy_overrides(&menu, &mut ov));
        assert!(ov.tools.contains_key("abc"));
    }

    /// 稳定 key 与盘符/父目录无关，恒为相对路径（一级即目录名）
    #[test]
    fn stable_key_is_location_independent() {
        let roots = vec![PathBuf::from("D:\\ScriptingTool\\cmd_tool")];
        assert_eq!(
            stable_key(&PathBuf::from("D:\\ScriptingTool\\cmd_tool\\nuclei"), &roots),
            "nuclei"
        );
        let roots2 = vec![PathBuf::from("E:\\other\\toolbox")];
        assert_eq!(
            stable_key(&PathBuf::from("E:\\other\\toolbox\\nuclei"), &roots2),
            "nuclei"
        );
        assert_eq!(
            stable_key(&PathBuf::from("E:\\other\\toolbox\\nuclei"), &[]),
            "nuclei"
        );
    }

    /// 名称归一化：去大小写/分隔符/VCS 后缀
    #[test]
    fn normalize_strips_separators_and_vcs_suffix() {
        assert_eq!(normalize_tool_name("JSFinder-master"), "jsfinder");
        assert_eq!(normalize_tool_name("JSFinder_Main"), "jsfinder");
        assert_eq!(normalize_tool_name("URLFinder"), "urlfinder");
        assert_eq!(normalize_tool_name("Nuclei"), "nuclei");
    }

    /// v2→v3：先搬家后升级的 B 类残留（键为旧绝对路径哈希），
    /// 靠自定义标题与目录名归一化兜底认领到新稳定 id，并补写 dir_path
    #[test]
    fn v2_to_v3_adopts_title_orphans_after_relocation() {
        let new_root = temp_dir("v3new");
        fs::create_dir_all(new_root.join("JSFinder-master")).unwrap();
        fs::create_dir_all(new_root.join("URLFinder")).unwrap();
        // nuclei 已在 v2 阶段迁好（键为新稳定 id），属于"现存正常记录"
        fs::create_dir_all(new_root.join("nuclei")).unwrap();
        let menu = menu_with_root(&new_root);

        // 构造 v2 落盘后的状态：nuclei 新键 + 两条 B 类旧键（旧路径哈希，标题线索）
        let old_js_path = format!("D:\\Py Scripting tool\\cmd_tool_web\\JSFinder-master");
        let old_url_path = format!("D:\\Py Scripting tool\\cmd_tool_web\\URLFinder");
        let mut ov = CmdToolOverrides::default();
        ov.version = 2;
        ov.tools.insert(
            make_id("nuclei"),
            CmdToolOverride {
                title: Some("nuclei".into()),
                category_id: Some("framework-scan".into()),
                dir_path: Some(new_root.join("nuclei").to_string_lossy().to_string()),
                ..Default::default()
            },
        );
        ov.tools.insert(
            make_id(&old_js_path),
            CmdToolOverride {
                title: Some("JSFinder".into()),
                desc: Some("666".into()),
                ..Default::default()
            },
        );
        ov.tools.insert(
            make_id(&old_url_path),
            CmdToolOverride {
                title: Some("URLFinder".into()),
                ..Default::default()
            },
        );

        adopt_orphans_by_title(&menu, &mut ov);

        let js = ov
            .tools
            .get(&make_id("JSFinder-master"))
            .expect("JSFinder 应按标题兜底认领");
        assert_eq!(js.desc.as_deref(), Some("666"));
        assert_eq!(
            js.dir_path.as_deref(),
            Some(new_root.join("JSFinder-master").to_string_lossy().as_ref())
        );
        let url = ov
            .tools
            .get(&make_id("URLFinder"))
            .expect("URLFinder 应按标题兜底认领");
        assert_eq!(url.title.as_deref(), Some("URLFinder"));
        // nuclei 等正常记录不受影响，旧键全部消失
        assert_eq!(ov.tools.len(), 3);
        assert!(ov.tools.contains_key(&make_id("nuclei")));
        assert!(ov.tools.get(&make_id(&old_js_path)).is_none());
        assert!(ov.tools.get(&make_id(&old_url_path)).is_none());

        let _ = fs::remove_dir_all(&new_root);
    }

    /// v2→v3：归一化后出现歧义（两个目录同名）时放弃认领；匹配不到也原样保留
    #[test]
    fn v2_to_v3_skips_ambiguous_and_missing() {
        let root = temp_dir("v3amb");
        fs::create_dir_all(root.join("JSFinder-master")).unwrap();
        fs::create_dir_all(root.join("JSFinder-main")).unwrap();
        let menu = menu_with_root(&root);

        let old_id = make_id("D:\\old\\JSFinder-master");
        let mut ov = CmdToolOverrides::default();
        ov.version = 2;
        ov.tools.insert(
            old_id.clone(),
            CmdToolOverride {
                title: Some("JSFinder".into()),
                ..Default::default()
            },
        );
        // 已删除工具：标题在现存目录中无对应
        let gone_id = make_id("D:\\old\\GoneTool");
        ov.tools.insert(
            gone_id.clone(),
            CmdToolOverride {
                title: Some("GoneTool".into()),
                ..Default::default()
            },
        );

        adopt_orphans_by_title(&menu, &mut ov);
        assert!(ov.tools.contains_key(&old_id), "歧义目录不应认领");
        assert!(ov.tools.contains_key(&gone_id), "无对应目录应保留");
        assert_eq!(ov.tools.len(), 2);

        let _ = fs::remove_dir_all(&root);
    }
}
