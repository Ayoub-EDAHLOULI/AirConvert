use tauri::{Manager, Theme};

mod audio_convert;
mod image_convert;
mod output_path;

#[tauri::command]
fn set_window_theme(window: tauri::WebviewWindow, theme: String) {
    let theme = match theme.as_str() {
        "dark" => Some(Theme::Dark),
        _ => Some(Theme::Light),
    };
    let _ = window.set_theme(theme);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            image_convert::convert_images,
            audio_convert::convert_audio_files,
            set_window_theme
        ])
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            let _ = window.set_theme(Some(Theme::Light));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
