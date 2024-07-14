// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![warn(clippy::unwrap_used)]

use crate::commands::{
    generate_background, get_image_size_count, get_jpg_image_infos, open_image, remove_watermark,
    show_path_in_file_manager,
};
use crate::events::{
    RemoveWatermarkEndEvent, RemoveWatermarkErrorEvent, RemoveWatermarkStartEvent,
    RemoveWatermarkSuccessEvent,
};
use tauri::Wry;

mod commands;
mod errors;
mod events;
mod types;
mod watermark;
mod extensions;

#[allow(clippy::unwrap_used)]
fn main() {
    let (invoke_handler, register_events) = {
        let builder = tauri_specta::ts::builder::<Wry>()
            .commands(tauri_specta::collect_commands![
                generate_background,
                remove_watermark,
                open_image,
                get_image_size_count,
                get_jpg_image_infos,
                show_path_in_file_manager,
            ])
            .events(tauri_specta::collect_events![
                RemoveWatermarkStartEvent,
                RemoveWatermarkSuccessEvent,
                RemoveWatermarkErrorEvent,
                RemoveWatermarkEndEvent
            ])
            .header("// @ts-nocheck"); // 跳过检查

        #[cfg(debug_assertions)] // 只有在debug模式下才会生成bindings.ts
        let builder = builder.path("../src/bindings.ts");

        builder.build().unwrap()
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(invoke_handler)
        .setup(|app| {
            register_events(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
