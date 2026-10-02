//! 全局约定常量与小工具：配置文件名、约定字符串、主题默认值/范围。
//! 跨语言（Rust ↔ TypeScript）无法共享定义，前端对应常量见 src/constants.ts，
//! 修改任一侧时必须同步另一侧。

// ---------- 配置文件与目录 ----------

/// 便携/开发配置目录名（exe 同目录或项目树根下）
pub const CONFIG_DIR: &str = "config";
/// 菜单配置：scanRoot 与分类列表
pub const MENU_FILE: &str = "menu.json";
/// EXE 手动录入工具列表
pub const EXE_TOOLS_FILE: &str = "exe-tools.json";
/// CMD 工具覆盖项（标题/副标题/手动分配）
pub const CMD_TOOLS_FILE: &str = "cmd-tools.json";
/// 主题设置
pub const SETTINGS_FILE: &str = "settings.json";

// ---------- 分类（目录）类型，与 menu.json 落盘值一致 ----------

/// CMD 自动扫描分类
pub const CAT_SCAN: &str = "scan";
/// EXE 手动录入分类
pub const CAT_MANUAL: &str = "manual";

// ---------- 工具类型，与 Tool.type 落盘/传输值一致 ----------

pub const TOOL_CMD: &str = "cmd";
pub const TOOL_EXE: &str = "exe";

// ---------- 主题模式 ----------
// 后端只在反序列化缺省时填 default；light/dark 仅前端解释、后端原样透传，
// 完整三值集合定义在前端 src/constants.ts 的 THEME_MODES。

/// 保持原样（不注入主题）
pub const THEME_MODE_DEFAULT: &str = "default";

// ---------- 工具目录内约定资源文件名（CMD 扫描与 EXE 录入共用） ----------

pub const README_FILE: &str = "README.md";
pub const OPS_FILE: &str = "Ops.md";
pub const ICON_FILE: &str = "icon.png";

// ---------- 其他约定字符串 ----------

/// 用户新建分类 id 的统一前缀
pub const CATEGORY_ID_PREFIX: &str = "cat-";

// ---------- 主题数值默认值与范围（前后端各持一份，须与 src/constants.ts 同步） ----------

/// 页面透明度默认值
pub const DEFAULT_PAGE_OPACITY: f64 = 0.85;
/// 弹窗不透明度默认值
pub const DEFAULT_DIALOG_OPACITY: f64 = 1.0;
/// 弹窗高斯模糊半径默认值（px）
pub const DEFAULT_DIALOG_BLUR: f64 = 0.0;
/// 不透明度下限/上限
pub const OPACITY_MIN: f64 = 0.0;
pub const OPACITY_MAX: f64 = 1.0;
/// 弹窗模糊半径持久化上限（px）；前端滑杆上限是更小的 UI 取值范围，与此独立
pub const DIALOG_BLUR_MAX: f64 = 60.0;
/// 目录排序权重默认值
pub const DEFAULT_WEIGHT: i32 = 0;

/// 剥掉 UTF-8 BOM（部分编辑器/PowerShell 保存 JSON 时会附加），解析配置前统一调用
pub fn strip_bom(s: &str) -> &str {
    s.trim_start_matches('\u{feff}')
}
