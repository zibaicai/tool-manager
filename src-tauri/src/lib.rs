mod commands;
mod models;

use commands::{config, exetools, launcher, scanner, settings};
use tauri::Manager;

#[cfg(windows)]
mod winframe {
    use core::sync::atomic::{AtomicUsize, Ordering};
    use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CallWindowProcW, DefWindowProcW, GetSystemMetrics, IsZoomed, SetWindowLongPtrW,
        SetWindowPos, WNDPROC, GWLP_WNDPROC, HWND_TOP, SM_CXPADDEDBORDER, SM_CXFRAME,
        SM_CYFRAME, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER,
        WM_NCCALCSIZE,
    };

    /// WM_NCCALCSIZE 的 lParam 指向该结构，rgrc[0] 是 DWM 提议的窗口矩形
    #[repr(C)]
    struct NcCalcSizeParams {
        rgrc: [RECT; 3],
        lppos: *mut core::ffi::c_void,
    }

    static ORIGINAL_PROC: AtomicUsize = AtomicUsize::new(0);

    unsafe extern "system" fn subclass_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if msg == WM_NCCALCSIZE && wparam != 0 {
            let params = &mut *(lparam as *mut NcCalcSizeParams);
            if IsZoomed(hwnd) != 0 {
                // 最大化时窗口矩形会超出屏幕若干像素（用于隐藏缩放边框），
                // 需把这些像素加回来，避免内容盖住任务栏
                let xpad = GetSystemMetrics(SM_CXFRAME) + GetSystemMetrics(SM_CXPADDEDBORDER);
                let ypad = GetSystemMetrics(SM_CYFRAME) + GetSystemMetrics(SM_CXPADDEDBORDER);
                params.rgrc[0].left += xpad;
                params.rgrc[0].right -= xpad;
                params.rgrc[0].top += ypad;
                params.rgrc[0].bottom -= ypad;
            }
            // 普通状态直接采用整个窗口矩形作为客户区：
            // 非客户区边框消失，DWM 无法再绘制灰边或顶部白色高光线
            return 0;
        }

        let original = ORIGINAL_PROC.load(Ordering::Relaxed);
        if original != 0 {
            CallWindowProcW(
                core::mem::transmute::<usize, WNDPROC>(original),
                hwnd,
                msg,
                wparam,
                lparam,
            )
        } else {
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
    }

    /// 安装子类化（仅主窗口一次）
    pub unsafe fn install(hwnd: HWND) {
        let proc_ptr = subclass_proc
            as unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT;
        let previous = SetWindowLongPtrW(hwnd, GWLP_WNDPROC, proc_ptr as usize as isize);
        ORIGINAL_PROC.store(previous as usize, Ordering::Relaxed);
        // 子类化安装晚于窗口首次布局，必须强制重算一次非客户区，
        // 否则 WM_NCCALCSIZE 拦截不生效，顶部残留白色标题栏底色
        SetWindowPos(
            hwnd,
            HWND_TOP,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
        );
    }
}

/// 去掉 Windows 11 DWM 给无边框窗口默认绘制的 1px 系统描边
#[cfg(windows)]
fn remove_dwm_border(window: &tauri::WebviewWindow) {
    use windows_sys::Win32::Graphics::Dwm::DwmSetWindowAttribute;

    if let Ok(hwnd) = window.hwnd() {
        unsafe {
            // DWMWA_BORDER_COLOR = 34；DWMWA_COLOR_NONE = 0xFFFFFFFE 关闭系统描边
            const DWMWA_BORDER_COLOR: u32 = 34;
            const DWMWA_COLOR_NONE: u32 = 0xFFFFFFFE;
            let border: u32 = DWMWA_COLOR_NONE;
            let _ = DwmSetWindowAttribute(
                hwnd.0 as _,
                DWMWA_BORDER_COLOR,
                &border as *const _ as _,
                std::mem::size_of::<u32>() as u32,
            );
            // WM_NCCALCSIZE 去除非客户区后，DWM 不再自动套用圆角；
            // 显式恢复 Win11 默认圆角（DWMWA_WINDOW_CORNER_PREFERENCE=33, DWMWCP_ROUND=2），
            // 原生阴影由窗口样式驱动，保持系统默认，不受影响
            const DWMWA_WINDOW_CORNER_PREFERENCE: u32 = 33;
            const DWMWCP_ROUND: u32 = 2;
            let corner: u32 = DWMWCP_ROUND;
            let _ = DwmSetWindowAttribute(
                hwnd.0 as _,
                DWMWA_WINDOW_CORNER_PREFERENCE,
                &corner as *const _ as _,
                std::mem::size_of::<u32>() as u32,
            );
            // 客户区覆盖整个窗口，消除 DWM 顶部残留的白色高光线（保留缩放能力）
            winframe::install(hwnd.0 as _);
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            #[cfg(windows)]
            if let Some(window) = app.get_webview_window("main") {
                remove_dwm_border(&window);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            scanner::scan_all_cmd_tools,
            scanner::update_cmd_tool,
            scanner::assign_cmd_tool,
            scanner::read_text_file,
            config::load_menu_config,
            config::add_category,
            config::rename_category,
            config::delete_category,
            config::set_scan_root,
            settings::load_theme_settings,
            settings::save_theme_settings,
            exetools::load_exe_tools,
            exetools::add_exe_tool,
            exetools::update_exe_tool,
            exetools::remove_exe_tool,
            launcher::launch_tool,
            launcher::open_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
