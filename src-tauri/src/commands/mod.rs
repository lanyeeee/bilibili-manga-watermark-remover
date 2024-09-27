pub mod prelude {
    pub use crate::commands::{
        download_episodes::download_episodes, generate_background::generate_background,
        generate_qr_code::generate_qr_code,
        get_background_dir_abs_path::get_background_dir_abs_path,
        get_background_dir_relative_path::get_background_dir_relative_path,
        get_bili_cookie_status_data::get_bili_cookie_status_data, get_config::get_config,
        get_jpg_image_infos::get_jpg_image_infos, get_manga_dir_data::get_manga_dir_data,
        get_manga_episodes::get_manga_episodes, get_qr_code_status_data::get_qr_code_status_data,
        open_image::open_image, remove_watermark::remove_watermark, save_config::save_config,
        search_manga::search_manga, show_path_in_file_manager::show_path_in_file_manager,
    };
}

mod download_episodes;
mod generate_background;
mod generate_qr_code;
mod get_background_dir_abs_path;
mod get_background_dir_relative_path;
mod get_bili_cookie_status_data;
mod get_config;
mod get_jpg_image_infos;
mod get_manga_dir_data;
mod get_manga_episodes;
mod get_qr_code_status_data;
mod open_image;
mod remove_watermark;
mod save_config;
mod search_manga;
mod show_path_in_file_manager;
