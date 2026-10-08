use crate::commands::config::ensure_config_dir;
use crate::constants::{
    strip_bom, CAT_SCAN, CMD_TOOLS_FILE, ICON_FILE, OPS_FILE, README_FILE, TOOL_CMD,
};
use crate::models::{Category, MenuConfig, Tool};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
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
    /// 排序权重；None 表示默认 0（缺省字段不序列化，保持旧文件兼容）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weight: Option<i32>,
    /// 脚本执行目录：工具目录下的相对子路径（或绝对路径），打开终端时作为工作目录
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exec_dir: Option<String>,
    /// 启动模式："terminal"=开终端执行（默认）；"spawn"=直接启动不开终端（GUI 工具如冰蝎/jar）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub launch_mode: Option<String>,
    /// 启动命令：覆盖默认 entry（如 `java -jar Behinder.jar` 或绝对路径 java.exe -jar xxx.jar）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub launch_command: Option<String>,
    /// 环境变量注入（如 PATH 指向自带 JRE）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env: Option<std::collections::HashMap<String, String>>,
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
    Ok(ensure_config_dir(app)?.join(CMD_TOOLS_FILE))
}

fn load_overrides(app: &AppHandle) -> CmdToolOverrides {
    fs::read_to_string(overrides_file(app).unwrap_or_default())
        .ok()
        .and_then(|c| serde_json::from_str(strip_bom(&c)).ok())
        .unwrap_or_default()
}

fn save_overrides(app: &AppHandle, o: &CmdToolOverrides) -> Result<(), String> {
    let json = serde_json::to_string_pretty(o).map_err(|e| e.to_string())?;
    fs::write(overrides_file(app)?, json)
        .map_err(|e| format!("写入 {} 失败: {}", CMD_TOOLS_FILE, e))
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
        if let Some(w) = ov.weight {
            tool.weight = w;
        }
        if let Some(ed) = &ov.exec_dir {
            tool.exec_dir = Some(ed.clone());
        }
        if let Some(lm) = &ov.launch_mode {
            tool.launch_mode = Some(lm.clone());
        }
        if let Some(lc) = &ov.launch_command {
            tool.launch_command = Some(lc.clone());
        }
        if let Some(e) = &ov.env {
            tool.env = Some(e.clone());
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
/// 匹配不到任何扫描根时（外部目录工具，如天狐工具箱等第三方工具源），
/// 用绝对路径归一化（小写 + 去尾部斜杠）作为 key——外部目录由第三方管理，
/// 不会像 scanRoot 那样被用户随意移动，绝对路径哈希可接受
fn stable_key(dir: &Path, roots: &[PathBuf]) -> String {
    let rel = roots
        .iter()
        .filter_map(|r| dir.strip_prefix(r).ok())
        .min_by_key(|p| p.as_os_str().len())
        .map(|p| p.to_string_lossy().replace('\\', "/"));
    rel.unwrap_or_else(|| {
        dir.to_string_lossy()
            .trim_end_matches(['\\', '/'])
            .to_lowercase()
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
    let readme = dir.join(README_FILE);
    let title = if readme.exists() {
        extract_h1(&readme).unwrap_or(dir_name.clone())
    } else {
        dir_name
    };

    // 文档：Ops.md
    let ops = dir.join(OPS_FILE);
    let doc_path = if ops.exists() {
        Some(ops.to_string_lossy().to_string())
    } else {
        None
    };

    // 图标：icon.png
    let icon_file = dir.join(ICON_FILE);
    let icon = if icon_file.exists() {
        Some(icon_file.to_string_lossy().to_string())
    } else {
        None
    };

    let path_str = dir.to_string_lossy().to_string();
    // 基线：不在任何扫描根下视为外部工具。手动录入/分配路径的工具由调用方
    // 按 is_scan_product 复核（位于顶层 scanRoot 下或已列入 dirs 名单的不可移除）
    let external = !roots
        .iter()
        .any(|r| is_under_root(&path_str, &r.to_string_lossy()));
    Some(Tool {
        id: make_id(&stable_key(dir, roots)),
        tool_type: TOOL_CMD.into(),
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
        stop_admin: false,
        weight: 0,
        exec_dir: None,
        launch_mode: None,
        launch_command: None,
        env: None,
        external,
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

/// 分类的生效扫描根：分类自带 scan_path 优先，缺省回退顶层 scanRoot。
/// 这是"扫描根生效规则"的唯一来源，扫描分配与手动分配校验共用，前端不再重复计算
fn effective_root(cat: &Category, menu: &MenuConfig) -> Option<String> {
    cat.scan_path.clone().or_else(|| menu.scan_root.clone())
}

/// 已被认领的一级目录名（归一化为小写）：所有 scan 分类 dirs 名单的并集，
/// 加上手动分配到现存目录的工具。孤儿扫描据此排除，全菜单只算一次
fn collect_claimed(menu: &MenuConfig, overrides: &CmdToolOverrides) -> HashSet<String> {
    let mut claimed: HashSet<String> = HashSet::new();
    for c in &menu.categories {
        if c.category_type == CAT_SCAN {
            for d in &c.dirs {
                claimed.insert(d.trim().trim_end_matches(['\\', '/']).to_lowercase());
            }
        }
    }
    for ov in overrides.tools.values() {
        if let (Some(cid), Some(dp)) = (&ov.category_id, &ov.dir_path) {
            if menu.categories.iter().any(|c| &c.id == cid) {
                if let Some(name) = Path::new(dp).file_name() {
                    claimed.insert(name.to_string_lossy().to_lowercase());
                }
            }
        }
    }
    claimed
}

/// 扫描单个 scan 分类：dirs 名单 → 剔除手动分走的 → 补入手动分来的 →
/// （仅第一个 scan 分类）顶层 scanRoot 孤儿兜底 → 应用自定义覆盖。
/// 生效根缺失/不是目录时该分类返回空列表，不影响其他分类
fn scan_category(
    menu: &MenuConfig,
    overrides: &CmdToolOverrides,
    roots: &[PathBuf],
    cat: &Category,
    is_first_scan: bool,
    claimed: &HashSet<String>,
) -> Vec<Tool> {
    let mut tools = Vec::new();
    let Some(root_str) = effective_root(cat, menu) else {
        return tools;
    };
    let root = PathBuf::from(&root_str);
    if !root.is_dir() {
        return tools;
    }

    // 仅收集 dirs 名单中的一级子目录（按填写顺序展示）。
    // 空名单表示目录初始为空，工具通过手动分配（cmd-tools.json）补入
    for name in &cat.dirs {
        if let Some(tool) = inspect_dir(&root.join(name.trim()), &cat.id, roots) {
            tools.push(tool);
        }
    }

    // 手动分配到其他现存分类的工具从本分类剔除（目标分类已删除时不剔除，避免工具消失）
    tools.retain(|t| match overrides
        .tools
        .get(&t.id)
        .and_then(|o| o.category_id.as_ref())
    {
        Some(cid) if cid != &cat.id => !menu.categories.iter().any(|c| &c.id == cid),
        _ => true,
    });

    // 手动分配到本分类但未被 dirs 名单覆盖的工具，按记录的目录路径补入
    for tool in collect_assigned(menu, overrides, roots, cat) {
        if !tools.iter().any(|t| t.id == tool.id) {
            tools.push(tool);
        }
    }

    // 兜底：未被任何 CMD 目录取走的孤儿工具，统一挂到第一个 scan 分类下（枚举顶层 scanRoot）
    if is_first_scan {
        if let Some(scan_root) = &menu.scan_root {
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
                    if let Some(tool) = inspect_dir(&dp, &cat.id, roots) {
                        if !tools.iter().any(|t| t.id == tool.id) {
                            tools.push(tool);
                        }
                    }
                }
            }
        }
    }

    for t in &mut tools {
        apply_override(t, overrides);
    }
    tools
}

/// 收集通过覆盖记录手动分配到指定分类的 CMD 工具（按记录的目录路径补入）。
/// scan 与 manual 两类分类通用：manual 分类不扫描目录，这是其获得 CMD 工具的唯一途径。
/// 这类工具的存在依赖手动记录：非扫描产物者标记 external（卡片提供移除入口）。
/// 注意：返回的工具尚未应用覆盖（标题/副标题），由调用方统一 apply_override
fn collect_assigned(
    menu: &MenuConfig,
    overrides: &CmdToolOverrides,
    roots: &[PathBuf],
    cat: &Category,
) -> Vec<Tool> {
    let mut tools: Vec<Tool> = Vec::new();
    for (id, ov) in &overrides.tools {
        if ov.category_id.as_deref() == Some(cat.id.as_str()) {
            if let Some(dp) = &ov.dir_path {
                if let Some(mut tool) = inspect_dir(Path::new(dp), &cat.id, roots) {
                    if !tools.iter().any(|t| &t.id == id) {
                        tool.external = !is_scan_product(menu, &tool.path);
                        tools.push(tool);
                    }
                }
            }
        }
    }
    tools
}

/// 按菜单扫描全部 scan 分类（纯逻辑，无 IO 配置读取；菜单/覆盖由命令层统一加载一次）
fn scan_all(menu: &MenuConfig, overrides: &CmdToolOverrides) -> Vec<Tool> {
    let roots = collect_roots(menu);
    let claimed = collect_claimed(menu, overrides);
    let first_scan_id = menu
        .categories
        .iter()
        .find(|c| c.category_type == CAT_SCAN)
        .map(|c| c.id.clone());

    let mut all = Vec::new();
    for cat in &menu.categories {
        if cat.category_type != CAT_SCAN {
            // 手动型分类不扫描目录，仅收集手动分配过来的 CMD 工具
            let mut assigned = collect_assigned(menu, overrides, &roots, cat);
            for t in &mut assigned {
                apply_override(t, overrides);
            }
            all.extend(assigned);
            continue;
        }
        let is_first = first_scan_id.as_deref() == Some(cat.id.as_str());
        all.extend(scan_category(
            menu,
            overrides,
            &roots,
            cat,
            is_first,
            &claimed,
        ));
    }
    all
}

/// 一次性加载菜单与覆盖（含旧版本迁移），扫描全部 CMD 工具分类。
/// 替代旧的按分类逐个调用：menu.json / cmd-tools.json 各只读一次，
/// 生效扫描根完全由后端按 effective_root 规则解析
#[tauri::command]
pub fn scan_all_cmd_tools(app: AppHandle) -> Result<Vec<Tool>, String> {
    let menu = crate::commands::config::load_menu_config(app.clone())?;
    let overrides = load_overrides_migrated(&app, Some(&menu));
    Ok(scan_all(&menu, &overrides))
}

/// 判断 path 是否位于 root 之下（不含 root 本身）；分隔符与大小写不敏感（Windows 语义）
fn is_under_root(path: &str, root: &str) -> bool {
    let norm = |p: &str| p.trim_end_matches(['\\', '/']).to_lowercase();
    let (p, r) = (norm(path), norm(root));
    p.len() > r.len()
        && p.starts_with(&r)
        && (p.as_bytes()[r.len()] == b'\\' || p.as_bytes()[r.len()] == b'/')
}

/// 扫描产物判定：位于顶层 scanRoot 之下（删掉覆盖记录后会被孤儿兜底重新扫出），
/// 或列在某 scan 分类的 dirs 名单中（删掉记录后仍会被该分类按名单扫出）。
/// 非扫描产物的工具仅靠手动录入/分配记录存在，移除记录即从列表消失 —— 这类才允许移除
fn is_scan_product(menu: &MenuConfig, path: &str) -> bool {
    if let Some(sr) = &menu.scan_root {
        if is_under_root(path, sr) {
            return true;
        }
    }
    let norm = |p: &Path| {
        p.to_string_lossy()
            .trim_end_matches(['\\', '/'])
            .to_lowercase()
    };
    let target = norm(Path::new(path));
    menu.categories
        .iter()
        .filter(|c| c.category_type == CAT_SCAN)
        .filter_map(|c| effective_root(c, menu).map(|r| (c, r)))
        .any(|(c, root)| {
            c.dirs
                .iter()
                .any(|d| norm(&Path::new(&root).join(d.trim())) == target)
        })
}

/// 手动分配 CMD 工具到指定目录（分类）。category_id 传 None 表示清除分配、恢复自动归属。
/// 目标为扫描型分类时要求工具目录位于其生效扫描根之下（支持任意深度，含一级）；
/// 手动型分类无此限制。
#[tauri::command]
pub fn assign_cmd_tool(
    app: AppHandle,
    id: String,
    path: String,
    category_id: Option<String>,
) -> Result<Tool, String> {
    // 菜单与覆盖各加载一次：迁移、目标分类校验、生效根解析共用同一份
    let menu = crate::commands::config::load_menu_config(app.clone())?;
    let roots = collect_roots(&menu);
    let mut overrides = load_overrides_migrated(&app, Some(&menu));
    let mut ov = overrides.tools.get(&id).cloned().unwrap_or_default();

    match &category_id {
        Some(cid) => {
            let cat = menu
                .categories
                .iter()
                .find(|c| &c.id == cid)
                .ok_or("未找到目标目录")?;
            // 扫描型分类要求工具物理上位于其生效扫描根之下（任意深度，含一级）；
            // 手动型分类（原 EXE 目录）仅作归属覆盖，不参与目录扫描，跳过根校验
            if cat.category_type == CAT_SCAN {
                let root = effective_root(cat, &menu).ok_or("目标目录未配置扫描根路径")?;
                if !is_under_root(&path, &root) {
                    return Err(format!(
                        "该工具不在目标目录的扫描根（{}）下，无法分配",
                        root
                    ));
                }
            }
            ov.category_id = Some(cid.clone());
            ov.dir_path = Some(path.clone());
        }
        None => {
            ov.category_id = None;
            ov.dir_path = None;
        }
    }

    if ov.title.is_none()
        && ov.desc.is_none()
        && ov.category_id.is_none()
        && ov.weight.is_none()
    {
        overrides.tools.remove(&id);
    } else {
        overrides.tools.insert(id.clone(), ov);
    }
    save_overrides(&app, &overrides)?;

    let view_cat = category_id.clone().unwrap_or_default();
    let mut tool = inspect_dir(Path::new(&path), &view_cat, &roots)
        .ok_or(format!("目录不存在: {}", path))?;
    tool.external = !is_scan_product(&menu, &path);
    apply_override(&mut tool, &overrides);
    Ok(tool)
}

/// 注册外部目录为 CMD 工具并分配到指定分类。用于把不在扫描根下的工具目录
/// （如天狐工具箱等第三方工具源）纳入管理。id 由 stable_key 的 fallback 分支
/// （绝对路径归一化哈希）派生，与 collect_assigned 调 inspect_dir 重新解析时一致。
/// 校验：目录存在 + 目录内含可执行文件（.exe/.bat/.ps1/.py 等），避免注册空目录
#[tauri::command]
pub fn register_cmd_tool(
    app: AppHandle,
    path: String,
    category_id: String,
    weight: Option<i32>,
) -> Result<Tool, String> {
    let dir = Path::new(&path);
    if !dir.is_dir() {
        return Err(format!("目录不存在: {}", path));
    }
    const CMD_ENTRY_EXTS: &[&str] = &["exe", "bat", "cmd", "ps1", "py", "sh", "jar"];
    let has_entry = fs::read_dir(dir)
        .map(|it| {
            it.flatten().any(|e| {
                e.path()
                    .extension()
                    .and_then(|x| x.to_str())
                    .map(|x| CMD_ENTRY_EXTS.contains(&x.to_lowercase().as_str()))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false);
    if !has_entry {
        return Err("该目录下未找到可执行文件（.exe/.bat/.ps1/.py 等）".into());
    }

    let menu = crate::commands::config::load_menu_config(app.clone())?;
    let cat = menu
        .categories
        .iter()
        .find(|c| c.id == category_id)
        .ok_or("未找到目标目录")?;
    let roots = collect_roots(&menu);

    let mut tool = inspect_dir(dir, &cat.id, &roots)
        .ok_or(format!("无法解析目录: {}", path))?;
    tool.external = !is_scan_product(&menu, &path);

    let mut overrides = load_overrides_migrated(&app, Some(&menu));
    let mut ov = overrides.tools.get(&tool.id).cloned().unwrap_or_default();
    ov.category_id = Some(category_id.clone());
    ov.dir_path = Some(path.clone());
    if let Some(w) = weight {
        ov.weight = Some(w);
    }
    overrides.tools.insert(tool.id.clone(), ov);
    save_overrides(&app, &overrides)?;

    apply_override(&mut tool, &overrides);
    Ok(tool)
}

/// 移除手动录入/分配的 CMD 工具：删除其覆盖记录（不删除磁盘上的目录）。
/// 扫描产物（位于顶层 scanRoot 下，或列在某 scan 分类的 dirs 名单中）删掉记录
/// 也会被重新扫出，拒绝移除
#[tauri::command]
pub fn remove_cmd_tool(app: AppHandle, id: String, path: String) -> Result<(), String> {
    let menu = crate::commands::config::load_menu_config(app.clone())?;
    if is_scan_product(&menu, &path) {
        return Err("该工具是扫描产物（位于扫描根下或已列入目录扫描名单），无法移除".into());
    }
    let mut overrides = load_overrides_migrated(&app, Some(&menu));
    overrides.tools.remove(&id);
    save_overrides(&app, &overrides)
}

/// 更新 CMD 工具的标题/副标题/权重/脚本执行目录/启动模式/启动命令/环境变量
/// （标题传空表示清除，标题恢复自动派生；exec_dir 传空表示清除、恢复工具目录本身）。
/// 需要传入工具的 path/categoryId 以便重新派生并返回最新 Tool。
#[tauri::command]
pub fn update_cmd_tool(
    app: AppHandle,
    id: String,
    path: String,
    category_id: String,
    title: Option<String>,
    desc: Option<String>,
    weight: Option<i32>,
    exec_dir: Option<String>,
    launch_mode: Option<String>,
    launch_command: Option<String>,
    env: Option<std::collections::HashMap<String, String>>,
) -> Result<Tool, String> {
    let title = title.filter(|t| !t.trim().is_empty()).map(|t| t.trim().to_string());
    let desc = desc.filter(|d| !d.trim().is_empty()).map(|d| d.trim().to_string());
    let weight = weight.filter(|w| *w != 0);
    let exec_dir = exec_dir
        .map(|s| s.trim().trim_matches('"').to_string())
        .filter(|s| !s.is_empty());
    // 相对子路径按工具目录拼出实际位置校验；绝对路径直接校验，避免保存无效值
    if let Some(sub) = &exec_dir {
        let p = Path::new(sub);
        let resolved = if p.is_absolute() {
            p.to_path_buf()
        } else {
            Path::new(&path).join(p)
        };
        if !resolved.is_dir() {
            return Err(format!("脚本执行目录不存在: {}", resolved.display()));
        }
    }
    let launch_mode = launch_mode
        .map(|s| s.trim().to_string())
        .filter(|s| s == "terminal" || s == "spawn");
    let launch_command = launch_command
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let env = env.filter(|m| !m.is_empty());

    let menu_opt = crate::commands::config::load_menu_config(app.clone()).ok();
    let roots = menu_opt.as_ref().map(collect_roots).unwrap_or_default();
    let mut overrides = load_overrides_migrated(&app, menu_opt.as_ref());
    let mut ov = overrides.tools.get(&id).cloned().unwrap_or_default();
    ov.title = title;
    ov.desc = desc;
    ov.weight = weight;
    ov.exec_dir = exec_dir;
    ov.launch_mode = launch_mode;
    ov.launch_command = launch_command;
    ov.env = env;
    // 补记工具目录绝对路径，使仅改过标题的记录在将来 scanRoot 搬迁时也能按目录名迁移
    ov.dir_path.get_or_insert_with(|| path.clone());
    // 保留已有的手动分配（category_id）；全空时整条删除（dir_path 不单独构成记录）
    if ov.title.is_none()
        && ov.desc.is_none()
        && ov.category_id.is_none()
        && ov.weight.is_none()
        && ov.exec_dir.is_none()
        && ov.launch_mode.is_none()
        && ov.launch_command.is_none()
        && ov.env.is_none()
    {
        overrides.tools.remove(&id);
    } else {
        overrides.tools.insert(id.clone(), ov);
    }
    save_overrides(&app, &overrides)?;

    let mut tool = inspect_dir(Path::new(&path), &category_id, &roots)
        .ok_or(format!("目录不存在: {}", path))?;
    if let Some(m) = &menu_opt {
        tool.external = !is_scan_product(m, &path);
    }
    apply_override(&mut tool, &overrides);
    Ok(tool)
}

/// 读取工具文档（Markdown）。出于安全考虑仅允许读取：
/// 1. 已配置扫描根（含分类专属扫描目录）之下的 .md；
/// 2. 已录入 EXE 工具所在目录之下的 .md。
/// 所有路径 canonicalize 后再比较以防 ../ 绕越；配置不可读或根不存在时拒绝。
#[tauri::command]
pub fn read_text_file(app: AppHandle, path: String) -> Result<String, String> {
    let target = Path::new(&path);
    if !target.is_file() {
        return Err(format!("文件不存在: {}", path));
    }
    if !target
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("md"))
    {
        return Err("仅支持读取工具目录内的 Markdown (.md) 文档".into());
    }
    let menu = crate::commands::config::load_menu_config(app.clone())?;
    let mut roots = collect_roots(&menu);
    roots.extend(crate::commands::exetools::exe_tool_dirs(&app));

    let real =
        fs::canonicalize(target).map_err(|e| format!("路径解析失败 {}: {}", path, e))?;
    let allowed = roots
        .iter()
        .filter_map(|r| fs::canonicalize(r).ok())
        .any(|r| real == r || real.starts_with(&r));
    if !allowed {
        return Err("文档不在已配置的工具目录范围内，拒绝读取".into());
    }
    fs::read_to_string(&real).map_err(|e| format!("读取失败 {}: {}", path, e))
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
                category_type: CAT_SCAN.into(),
                scan_path: None,
                dirs: vec![],
                weight: 0,
            }],
            scan_root: Some(root.to_string_lossy().to_string()),
        }
    }

    fn make_cat(id: &str, dirs: &[&str], scan_path: Option<&str>) -> Category {
        Category {
            id: id.into(),
            name: id.into(),
            category_type: CAT_SCAN.into(),
            scan_path: scan_path.map(|s| s.to_string()),
            dirs: dirs.iter().map(|s| s.to_string()).collect(),
            weight: 0,
        }
    }

    /// dirs 名单分类拿到名单工具；首个 scan 分类（空名单）兜底拿到全部孤儿（按名排序）
    #[test]
    fn scan_all_partitions_listed_and_orphans() {
        let root = temp_dir("scanall");
        for d in ["nuclei", "URLFinder", "JSFinder-master"] {
            fs::create_dir_all(root.join(d)).unwrap();
        }
        let menu = MenuConfig {
            categories: vec![
                make_cat("empty-first", &[], None),
                make_cat("listed", &["nuclei"], None),
            ],
            scan_root: Some(root.to_string_lossy().to_string()),
        };

        let tools = scan_all(&menu, &CmdToolOverrides::default());
        let ids_of = |cid: &str| {
            tools
                .iter()
                .filter(|t| t.category_id == cid)
                .map(|t| t.id.clone())
                .collect::<Vec<_>>()
        };
        assert_eq!(ids_of("listed"), vec![make_id("nuclei")]);
        // 孤儿按名称排序：JSFinder-master < URLFinder
        assert_eq!(
            ids_of("empty-first"),
            vec![make_id("JSFinder-master"), make_id("URLFinder")]
        );
        assert_eq!(tools.len(), 3);

        let _ = fs::remove_dir_all(&root);
    }

    /// 手动分配到空名单分类：目标分类补入该工具，原 dirs 分类将其剔除
    #[test]
    fn scan_all_assignment_overrides_dir_partition() {
        let root = temp_dir("assign");
        fs::create_dir_all(root.join("nuclei")).unwrap();
        fs::create_dir_all(root.join("URLFinder")).unwrap();
        let nuclei_abs = root.join("nuclei").to_string_lossy().to_string();
        let menu = MenuConfig {
            categories: vec![
                make_cat("target", &[], None),
                make_cat("listed", &["nuclei"], None),
            ],
            scan_root: Some(root.to_string_lossy().to_string()),
        };
        let mut ov = CmdToolOverrides::default();
        ov.version = OVERRIDES_VERSION;
        ov.tools.insert(
            make_id("nuclei"),
            CmdToolOverride {
                category_id: Some("target".into()),
                dir_path: Some(nuclei_abs),
                ..Default::default()
            },
        );

        let tools = scan_all(&menu, &ov);
        let in_cat = |cid: &str| {
            tools
                .iter()
                .filter(|t| t.category_id == cid)
                .map(|t| t.id.clone())
                .collect::<Vec<_>>()
        };
        assert_eq!(in_cat("target"), vec![make_id("nuclei"), make_id("URLFinder")]);
        assert!(in_cat("listed").is_empty(), "分走的工具应从名单分类剔除");
        assert_eq!(tools.len(), 2);
        assert!(tools.iter().any(|t| t.id == make_id("URLFinder") && t.category_id == "target"));

        let _ = fs::remove_dir_all(&root);
    }

    /// 生效根规则：分类自带 scan_path 优先于顶层 scanRoot；
    /// 自带根只按 dirs 名单扫描（其下未列入的目录不会成为孤儿，顶层根不存在时兜底静默为空）
    #[test]
    fn effective_root_prefers_category_scan_path() {
        let own = temp_dir("ownroot");
        fs::create_dir_all(own.join("solo")).unwrap();
        fs::create_dir_all(own.join("distractor")).unwrap();
        let menu = MenuConfig {
            categories: vec![make_cat("c1", &["solo"], Some(own.to_str().unwrap()))],
            scan_root: Some("Z:\\no-such-root".into()),
        };
        let tools = scan_all(&menu, &CmdToolOverrides::default());
        assert_eq!(tools.len(), 1, "自带根只扫描 dirs 名单，distractor 不出现");
        assert_eq!(tools[0].id, make_id("solo"));
        assert_eq!(tools[0].category_id, "c1");

        // 直接验证规则函数
        let cat = make_cat("c2", &[], None);
        let mut m2 = menu.clone();
        m2.categories = vec![cat];
        assert_eq!(
            effective_root(&m2.categories[0], &m2).as_deref(),
            Some("Z:\\no-such-root")
        );

        let _ = fs::remove_dir_all(&own);
    }

    /// 未配置任何扫描根时扫描结果为空且不报错
    #[test]
    fn scan_all_empty_without_roots() {
        let menu = MenuConfig {
            categories: vec![make_cat("c1", &["nuclei"], None)],
            scan_root: None,
        };
        assert!(scan_all(&menu, &CmdToolOverrides::default()).is_empty());
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
                weight: None,
                exec_dir: None,
            launch_mode: None,
            launch_command: None,
            env: None,
            },
        );
        ov.tools.insert(
            make_id(&js_abs),
            CmdToolOverride {
                title: Some("JSFinder".into()),
                desc: Some("666".into()),
                category_id: None,
                dir_path: Some(js_abs.clone()),
                weight: None,
                exec_dir: None,
            launch_mode: None,
            launch_command: None,
            env: None,
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
                weight: None,
                exec_dir: None,
            launch_mode: None,
            launch_command: None,
            env: None,
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

    /// 稳定 key：扫描根下的目录恒为相对路径（一级即目录名），与盘符/父目录无关；
    /// 匹配不到任何扫描根的外部目录（如天狐工具箱）走绝对路径归一化（小写 + 去尾斜杠）
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
        // 外部目录：不在任何扫描根下，走绝对路径归一化（与 register_cmd_tool 的 id 策略一致）
        assert_eq!(
            stable_key(&PathBuf::from("E:\\other\\toolbox\\nuclei"), &[]),
            "e:\\other\\toolbox\\nuclei"
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

    /// 手动录入/分配的工具：不在顶层 scanRoot 下且未列入任何 dirs 名单 → external（可移除）；
    /// 位于 scanRoot 下（孤儿兜底会扫回）或已列入 dirs 名单的仍是扫描产物（不可移除）
    #[test]
    fn assigned_tool_external_flag_follows_scan_product_rule() {
        let top = temp_dir("toproot");
        let own = temp_dir("ownroot2");
        fs::create_dir_all(top.join("nuclei")).unwrap();
        fs::create_dir_all(own.join("heapdump")).unwrap();
        fs::create_dir_all(own.join("listed")).unwrap();

        let nuclei_abs = top.join("nuclei").to_string_lossy().to_string();
        let heapdump_abs = own.join("heapdump").to_string_lossy().to_string();
        let listed_abs = own.join("listed").to_string_lossy().to_string();
        let menu = MenuConfig {
            categories: vec![
                make_cat("c1", &[], None),
                make_cat("c2", &["listed"], Some(own.to_str().unwrap())),
            ],
            scan_root: Some(top.to_string_lossy().to_string()),
        };
        let roots = collect_roots(&menu);
        let mut ov = CmdToolOverrides::default();
        ov.version = OVERRIDES_VERSION;
        // heapdump：分类自带根下、未列入 dirs，仅靠分配记录出现
        ov.tools.insert(
            make_id(&stable_key(&own.join("heapdump"), &roots)),
            CmdToolOverride {
                category_id: Some("c2".into()),
                dir_path: Some(heapdump_abs.clone()),
                ..Default::default()
            },
        );
        // nuclei：顶层 scanRoot 下，被分配到 c2
        ov.tools.insert(
            make_id(&stable_key(&top.join("nuclei"), &roots)),
            CmdToolOverride {
                category_id: Some("c2".into()),
                dir_path: Some(nuclei_abs.clone()),
                ..Default::default()
            },
        );

        let tools = scan_all(&menu, &ov);
        let heapdump = tools
            .iter()
            .find(|t| t.path == heapdump_abs)
            .expect("heapdump 应被补入 c2");
        assert!(heapdump.external, "仅靠手动记录存在的外部工具应可移除");
        let nuclei = tools
            .iter()
            .find(|t| t.path == nuclei_abs)
            .expect("nuclei 应被补入 c2");
        assert!(!nuclei.external, "位于顶层 scanRoot 下，删记录会被孤儿兜底扫回");
        let listed = tools
            .iter()
            .find(|t| t.path == listed_abs)
            .expect("listed 应被 dirs 名单扫出");
        assert!(!listed.external, "已列入 dirs 名单的扫描产物不可移除");

        // is_scan_product 直测
        assert!(is_scan_product(&menu, &nuclei_abs));
        assert!(is_scan_product(&menu, &listed_abs));
        assert!(!is_scan_product(&menu, &heapdump_abs));

        let _ = fs::remove_dir_all(&top);
        let _ = fs::remove_dir_all(&own);
    }

    /// apply_override 把 launch_mode/launch_command/env 应用到 Tool
    #[test]
    fn apply_override_applies_launch_fields() {
        let mut overrides = CmdToolOverrides::default();
        let mut env = std::collections::HashMap::new();
        env.insert("PATH".into(), "D:\\jre\\bin".into());
        overrides.tools.insert(
            "id1".into(),
            CmdToolOverride {
                launch_mode: Some("spawn".into()),
                launch_command: Some("java -jar app.jar".into()),
                env: Some(env),
                ..Default::default()
            },
        );
        let mut tool = Tool {
            id: "id1".into(),
            tool_type: TOOL_CMD.into(),
            title: "t".into(),
            path: "/x".into(),
            doc_path: None,
            icon: None,
            category_id: "c1".into(),
            available: true,
            args: None,
            desc: None,
            admin: false,
            stop_path: None,
            stop_admin: false,
            weight: 0,
            exec_dir: None,
            launch_mode: None,
            launch_command: None,
            env: None,
            external: false,
        };
        apply_override(&mut tool, &overrides);
        assert_eq!(tool.launch_mode.as_deref(), Some("spawn"));
        assert_eq!(tool.launch_command.as_deref(), Some("java -jar app.jar"));
        assert_eq!(
            tool.env.as_ref().unwrap().get("PATH"),
            Some(&"D:\\jre\\bin".to_string())
        );
    }

    /// 新字段在 None 时不参与序列化；旧 cmd-tools.json（无新字段）反序列化兼容
    #[test]
    fn cmd_tool_override_optional_fields_serialize_compat() {
        let ov = CmdToolOverride {
            title: Some("t".into()),
            ..Default::default()
        };
        let json = serde_json::to_string(&ov).unwrap();
        assert!(!json.contains("launchMode"), "launchMode 应不序列化");
        assert!(!json.contains("launchCommand"));
        assert!(!json.contains("env"));
        // 旧 JSON（无新字段）反序列化得到默认 None
        let legacy = r#"{"title":"旧工具"}"#;
        let parsed: CmdToolOverride = serde_json::from_str(legacy).unwrap();
        assert_eq!(parsed.title.as_deref(), Some("旧工具"));
        assert!(parsed.launch_mode.is_none());
        assert!(parsed.launch_command.is_none());
        assert!(parsed.env.is_none());
    }
}
