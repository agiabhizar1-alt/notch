#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

#[tauri::command]
fn set_always_on_top(window: tauri::WebviewWindow, enabled: bool) -> Result<bool, String> {
    window
        .set_always_on_top(enabled)
        .map(|()| enabled)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn set_expanded(
    window: tauri::WebviewWindow,
    expanded: bool,
    position: String,
    collapsed_width: f64,
    collapsed_height: f64,
    expanded_width: f64,
    expanded_height: f64,
) -> Result<bool, String> {
    validate_position(&position)?;
    validate_widget_sizes(collapsed_width, collapsed_height, expanded_width, expanded_height)?;
    let (width, height) = widget_size(
        expanded,
        collapsed_width,
        collapsed_height,
        expanded_width,
        expanded_height,
    );
    resize_and_place(&window, &position, width, height)?;
    Ok(expanded)
}

#[tauri::command]
fn set_appbar(
    window: tauri::WebviewWindow,
    enabled: bool,
    position: String,
    expanded: bool,
    collapsed_width: f64,
    collapsed_height: f64,
    expanded_width: f64,
    expanded_height: f64,
) -> Result<bool, String> {
    validate_position(&position)?;
    validate_widget_sizes(collapsed_width, collapsed_height, expanded_width, expanded_height)?;

    #[cfg(target_os = "windows")]
    {
        windows_appbar::set_appbar(&window, enabled, &position, collapsed_width, collapsed_height)
            .map_err(|error| error.to_string())?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = enabled;
        return Err("Fitur AppBar hanya tersedia di Windows.".to_string());
    }

    let (width, height) = widget_size(
        expanded,
        collapsed_width,
        collapsed_height,
        expanded_width,
        expanded_height,
    );
    resize_and_place(&window, &position, width, height)?;
    Ok(enabled)
}

#[tauri::command]
fn set_dock_position(
    window: tauri::WebviewWindow,
    position: String,
    appbar_enabled: bool,
    expanded: bool,
    collapsed_width: f64,
    collapsed_height: f64,
    expanded_width: f64,
    expanded_height: f64,
) -> Result<bool, String> {
    validate_position(&position)?;
    validate_widget_sizes(collapsed_width, collapsed_height, expanded_width, expanded_height)?;

    #[cfg(target_os = "windows")]
    {
        if appbar_enabled {
            windows_appbar::set_appbar(&window, true, &position, collapsed_width, collapsed_height)
                .map_err(|error| error.to_string())?;
        }
    }

    let (width, height) = widget_size(
        expanded,
        collapsed_width,
        collapsed_height,
        expanded_width,
        expanded_height,
    );
    resize_and_place(&window, &position, width, height)?;
    Ok(true)
}

#[tauri::command]
fn set_widget_position(
    app: tauri::AppHandle,
    position: String,
    appbar_enabled: bool,
    expanded: bool,
    collapsed_width: f64,
    collapsed_height: f64,
    expanded_width: f64,
    expanded_height: f64,
) -> Result<bool, String> {
    validate_position(&position)?;
    validate_widget_sizes(collapsed_width, collapsed_height, expanded_width, expanded_height)?;

    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "Jendela widget tidak ditemukan.".to_string())?;

    #[cfg(target_os = "windows")]
    if appbar_enabled {
        windows_appbar::set_appbar(&window, true, &position, collapsed_width, collapsed_height)
            .map_err(|error| error.to_string())?;
    }

    let (width, height) = widget_size(
        expanded,
        collapsed_width,
        collapsed_height,
        expanded_width,
        expanded_height,
    );
    resize_and_place(&window, &position, width, height)?;
    Ok(true)
}

#[tauri::command]
fn set_widget_dimensions(
    app: tauri::AppHandle,
    position: String,
    appbar_enabled: bool,
    expanded: bool,
    collapsed_width: f64,
    collapsed_height: f64,
    expanded_width: f64,
    expanded_height: f64,
) -> Result<bool, String> {
    validate_position(&position)?;
    validate_widget_sizes(collapsed_width, collapsed_height, expanded_width, expanded_height)?;
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "Jendela widget tidak ditemukan.".to_string())?;

    #[cfg(target_os = "windows")]
    if appbar_enabled {
        windows_appbar::set_appbar(&window, true, &position, collapsed_width, collapsed_height)
            .map_err(|error| error.to_string())?;
    }

    let (width, height) = widget_size(
        expanded,
        collapsed_width,
        collapsed_height,
        expanded_width,
        expanded_height,
    );
    resize_and_place(&window, &position, width, height)?;
    Ok(true)
}

#[tauri::command]
fn open_settings(app: tauri::AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("settings")
        .ok_or_else(|| "Jendela pengaturan tidak ditemukan.".to_string())?;
    window.show().map_err(|error| error.to_string())?;
    window.set_focus().map_err(|error| error.to_string())
}

#[tauri::command]
fn get_autostart(app: tauri::AppHandle) -> Result<bool, String> {
    use tauri_plugin_autostart::ManagerExt;

    app.autolaunch()
        .is_enabled()
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn set_autostart(app: tauri::AppHandle, enabled: bool) -> Result<bool, String> {
    use tauri_plugin_autostart::ManagerExt;

    let manager = app.autolaunch();
    let result = if enabled {
        manager.enable()
    } else {
        manager.disable()
    };
    result.map(|()| enabled).map_err(|error| error.to_string())
}

fn validate_position(position: &str) -> Result<(), String> {
    if position == "top" || position == "bottom" {
        Ok(())
    } else {
        Err("Posisi widget tidak valid.".to_string())
    }
}

fn validate_widget_sizes(
    collapsed_width: f64,
    collapsed_height: f64,
    expanded_width: f64,
    expanded_height: f64,
) -> Result<(), String> {
    let valid = collapsed_width.is_finite()
        && (160.0..=640.0).contains(&collapsed_width)
        && collapsed_height.is_finite()
        && (42.0..=110.0).contains(&collapsed_height)
        && expanded_width.is_finite()
        && (320.0..=900.0).contains(&expanded_width)
        && expanded_height.is_finite()
        && (360.0..=900.0).contains(&expanded_height);
    if valid {
        Ok(())
    } else {
        Err("Ukuran widget berada di luar batas yang diizinkan.".to_string())
    }
}

fn widget_size(
    expanded: bool,
    collapsed_width: f64,
    collapsed_height: f64,
    expanded_width: f64,
    expanded_height: f64,
) -> (f64, f64) {
    if expanded {
        (expanded_width, expanded_height)
    } else {
        (collapsed_width, collapsed_height)
    }
}

fn resize_and_place(
    window: &tauri::WebviewWindow,
    position: &str,
    width: f64,
    height: f64,
) -> Result<(), String> {
    window
        .set_size(tauri::Size::Logical(tauri::LogicalSize::new(width, height)))
        .map_err(|error| error.to_string())?;

    #[cfg(target_os = "windows")]
    {
        windows_appbar::place_window(window, position, width, height)
            .map_err(|error| error.to_string())
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = (window, position);
        Ok(())
    }
}

#[cfg(target_os = "windows")]
mod windows_appbar {
    use std::mem::size_of;
    use tauri::platform::windows::WindowExtWindows;
    use tauri::WebviewWindow;
    use windows::Win32::Foundation::{HWND, RECT};
    use windows::Win32::UI::Shell::{
        SHAppBarMessage, ABM_NEW, ABM_QUERYPOS, ABM_REMOVE, ABM_SETPOS, ABE_BOTTOM, ABE_TOP,
        APPBARDATA,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        GetSystemMetrics, SetWindowPos, SM_CXSCREEN, SM_CYSCREEN, SWP_NOACTIVATE,
        SWP_SHOWWINDOW,
    };

    fn appbar_data(hwnd: HWND, position: &str, width: i32, height: i32) -> APPBARDATA {
        let screen_height = unsafe { GetSystemMetrics(SM_CYSCREEN) };
        let screen_width = unsafe { GetSystemMetrics(SM_CXSCREEN) };
        let edge = if position == "bottom" {
            ABE_BOTTOM
        } else {
            ABE_TOP
        };
        let left = (screen_width - width).max(0) / 2;
        let top = if position == "bottom" {
            screen_height - height
        } else {
            0
        };
        APPBARDATA {
            cbSize: size_of::<APPBARDATA>() as u32,
            hWnd: hwnd,
            uCallbackMessage: 0,
            uEdge: edge,
            rc: RECT {
                left,
                top,
                right: (left + width).min(screen_width),
                bottom: top + height,
            },
            lParam: Default::default(),
        }
    }

    fn physical_dimensions(
        window: &WebviewWindow,
        width: f64,
        height: f64,
    ) -> Result<(i32, i32), String> {
        let scale = window.scale_factor().map_err(|error| error.to_string())?;
        Ok(((width * scale).round() as i32, (height * scale).round() as i32))
    }

    pub fn place_window(
        window: &WebviewWindow,
        position: &str,
        logical_width: f64,
        logical_height: f64,
    ) -> Result<(), String> {
        let (width, height) = physical_dimensions(window, logical_width, logical_height)?;
        let screen_width = unsafe { GetSystemMetrics(SM_CXSCREEN) };
        let screen_height = unsafe { GetSystemMetrics(SM_CYSCREEN) };
        let x = (screen_width - width).max(0) / 2;
        let y = if position == "bottom" {
            screen_height - height
        } else {
            0
        };
        let raw_hwnd = window.hwnd().map_err(|error| error.to_string())?;
        let hwnd = HWND(raw_hwnd.0 as *mut std::ffi::c_void);
        let positioned = unsafe {
            SetWindowPos(
                hwnd,
                HWND(-1isize as *mut std::ffi::c_void),
                x,
                y,
                width,
                height,
                SWP_NOACTIVATE | SWP_SHOWWINDOW,
            )
        };
        if !positioned.as_bool() {
            return Err("Windows tidak dapat memosisikan widget.".to_string());
        }
        Ok(())
    }

    pub fn set_appbar(
        window: &WebviewWindow,
        enabled: bool,
        position: &str,
        logical_width: f64,
        logical_height: f64,
    ) -> Result<(), String> {
        let (width, height) = physical_dimensions(window, logical_width, logical_height)?;
        let raw_hwnd = window.hwnd().map_err(|error| error.to_string())?;
        let hwnd = HWND(raw_hwnd.0 as *mut std::ffi::c_void);
        if !enabled {
            let mut data = appbar_data(hwnd, position, width, height);
            unsafe {
                SHAppBarMessage(ABM_REMOVE, &mut data);
            }
            return Ok(());
        }

        let mut data = appbar_data(hwnd, position, width, height);
        unsafe {
            SHAppBarMessage(ABM_REMOVE, &mut data);
            if SHAppBarMessage(ABM_NEW, &mut data) == 0 {
                return Err("Windows menolak pendaftaran AppBar.".to_string());
            }
            SHAppBarMessage(ABM_QUERYPOS, &mut data);
            if position == "bottom" {
                data.rc.top = data.rc.bottom - height;
            } else {
                data.rc.bottom = data.rc.top + height;
            }
            if SHAppBarMessage(ABM_SETPOS, &mut data) == 0 {
                return Err("Windows tidak dapat menetapkan area AppBar.".to_string());
            }
        }

        let positioned = unsafe {
            SetWindowPos(
                hwnd,
                HWND(-1isize as *mut std::ffi::c_void),
                data.rc.left,
                data.rc.top,
                data.rc.right - data.rc.left,
                data.rc.bottom - data.rc.top,
                SWP_NOACTIVATE | SWP_SHOWWINDOW,
            )
        };
        if !positioned.as_bool() {
            return Err("Windows tidak dapat memosisikan AppBar.".to_string());
        }
        Ok(())
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .on_window_event(|window, event| {
            if window.label() == "settings" {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    if let Err(error) = window.hide() {
                        eprintln!("Tidak dapat menyembunyikan jendela pengaturan: {error}");
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            set_always_on_top,
            set_expanded,
            set_appbar,
            set_dock_position,
            set_widget_position,
            set_widget_dimensions,
            open_settings,
            get_autostart,
            set_autostart
        ])
        .run(tauri::generate_context!())
        .expect("Gagal menjalankan aplikasi Notch");
}
