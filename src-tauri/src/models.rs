use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Tool {
    pub id: String,
    #[serde(rename = "type")]
    pub tool_type: String,
    pub title: String,
    pub path: String,
    pub doc_path: Option<String>,
    pub icon: Option<String>,
    pub category_id: String,
    pub available: bool,
    /// EXE 类工具的启动参数（如 pythonw.exe -c "..."）；CMD 类恒为 None
    #[serde(default)]
    pub args: Option<String>,
    /// 副标题：工具用途描述；CMD 类恒为 None
    #[serde(default)]
    pub desc: Option<String>,
    /// 以管理员身份启动（EXE 类）；CMD 类恒为 false
    #[serde(default)]
    pub admin: bool,
    /// 关闭脚本（.bat）路径（EXE 类可选）；CMD 类恒为 None
    #[serde(default)]
    pub stop_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub category_type: String,
    /// 分类级扫描目录，缺省时回退到顶层 scanRoot
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scan_path: Option<String>,
    /// 手动归属：汇总目录下划入本分类的一级子目录名（按填写顺序展示）；
    /// 为空表示目录初始为空，工具通过编辑卡片的「所属目录」手动分配
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dirs: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuConfig {
    pub categories: Vec<Category>,
    /// 工具目录汇总根目录（CMD 类工具统一存放处）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scan_root: Option<String>,
}
