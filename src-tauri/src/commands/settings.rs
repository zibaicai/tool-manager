use crate::commands::config::ensure_config_dir;
use crate::constants::{
    strip_bom, DEFAULT_DIALOG_BLUR, DEFAULT_DIALOG_OPACITY, DEFAULT_PAGE_OPACITY, DIALOG_BLUR_MAX,
    OPACITY_MAX, OPACITY_MIN, SETTINGS_FILE, THEME_MODE_DEFAULT,
};
use serde::{Deserialize, Serialize};
use std::fs;
use tauri::AppHandle;

/// 用户导入的自定义背景图
#[derive(Debug, Serialize, Deserialize, Clone, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/types/bindings.ts", rename_all = "camelCase")]
pub struct CustomBg {
    /// 图片绝对路径
    pub path: String,
    /// 归属主题：light / dark
    pub theme: String,
}

#[derive(Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/types/bindings.ts", rename_all = "camelCase")]
pub struct ThemeSettings {
    /// default（保持原样）/ light / dark
    #[serde(default = "default_mode")]
    pub mode: String,
    /// 页面透明度 0 ~ 1.0
    #[serde(default = "default_opacity")]
    pub opacity: f64,
    /// 弹窗背景不透明度 0 ~ 1.0（1.0 = 完全不透明）
    #[serde(default = "default_dialog_opacity")]
    pub dialog_opacity: f64,
    /// 弹窗背景高斯模糊半径（px），0 = 不模糊
    #[serde(default = "default_dialog_blur")]
    pub dialog_blur: f64,
    /// 背景图："builtin:light-1" 或自定义图片绝对路径
    #[serde(default)]
    pub bg: Option<String>,
    #[serde(default)]
    pub custom_bgs: Vec<CustomBg>,
}

fn default_mode() -> String {
    THEME_MODE_DEFAULT.into()
}
fn default_opacity() -> f64 {
    DEFAULT_PAGE_OPACITY
}
fn default_dialog_opacity() -> f64 {
    DEFAULT_DIALOG_OPACITY
}
fn default_dialog_blur() -> f64 {
    DEFAULT_DIALOG_BLUR
}

impl Default for ThemeSettings {
    fn default() -> Self {
        Self {
            mode: default_mode(),
            opacity: default_opacity(),
            dialog_opacity: default_dialog_opacity(),
            dialog_blur: default_dialog_blur(),
            bg: None,
            custom_bgs: vec![],
        }
    }
}

fn settings_file(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    Ok(ensure_config_dir(app)?.join(SETTINGS_FILE))
}

#[tauri::command]
pub fn load_theme_settings(app: AppHandle) -> Result<ThemeSettings, String> {
    let p = settings_file(&app)?;
    if !p.exists() {
        return Ok(ThemeSettings::default());
    }
    let content = fs::read_to_string(&p)
        .map_err(|e| format!("读取 {} 失败: {}", SETTINGS_FILE, e))?;
    // 容忍 BOM；损坏时回退默认而不是让界面崩掉
    Ok(serde_json::from_str(strip_bom(&content)).unwrap_or_default())
}

#[tauri::command]
pub fn save_theme_settings(app: AppHandle, settings: ThemeSettings) -> Result<(), String> {
    let opacity = settings.opacity.clamp(OPACITY_MIN, OPACITY_MAX);
    let dialog_opacity = settings.dialog_opacity.clamp(OPACITY_MIN, OPACITY_MAX);
    let dialog_blur = settings.dialog_blur.clamp(OPACITY_MIN, DIALOG_BLUR_MAX);
    let fixed = ThemeSettings {
        opacity,
        dialog_opacity,
        dialog_blur,
        ..settings
    };
    let json = serde_json::to_string_pretty(&fixed).map_err(|e| e.to_string())?;
    fs::write(settings_file(&app)?, json)
        .map_err(|e| format!("写入 {} 失败: {}", SETTINGS_FILE, e))
}
