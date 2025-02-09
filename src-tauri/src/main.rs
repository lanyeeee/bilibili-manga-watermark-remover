// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![warn(clippy::unwrap_used)]

use parking_lot::RwLock;
use tauri::{Context, Manager, Wry};

use crate::commands::prelude::*;
use crate::config::Config;
use crate::events::prelude::*;

mod commands;
mod config;
mod errors;
mod events;
mod extensions;
mod types;
mod utils;

fn generate_context() -> Context<Wry> {
    tauri::generate_context!()
}

fn main() {
    let builder = tauri_specta::Builder::<Wry>::new()
        .commands(tauri_specta::collect_commands![
            generate_background,
            remove_watermark,
            open_image,
            get_manga_dir_data,
            get_jpg_image_infos,
            show_path_in_file_manager,
            get_background_dir_relative_path,
            get_background_dir_abs_path,
            get_config,
            save_config,
        ])
        .events(tauri_specta::collect_events![
            RemoveWatermarkStartEvent,
            RemoveWatermarkSuccessEvent,
            RemoveWatermarkErrorEvent,
            RemoveWatermarkEndEvent,
        ]);
    // 只有在debug模式下才会生成bindings.ts
    #[cfg(debug_assertions)]
    builder
        .export(
            specta_typescript::Typescript::default()
                .bigint(specta_typescript::BigIntExportBehavior::Number)
                .formatter(specta_typescript::formatter::prettier)
                .header("// @ts-nocheck"), // 跳过检查
            "../src/bindings.ts",
        )
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);
            let config = RwLock::new(Config::new(app.handle())?);
            app.manage(config);
            Ok(())
        })
        .run(generate_context())
        .expect("error while running tauri application");
}
