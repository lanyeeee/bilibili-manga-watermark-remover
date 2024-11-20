pub mod prelude {
    pub use crate::commands::{
        generate_background::generate_background,
        get_background_dir_abs_path::get_background_dir_abs_path,
        get_background_dir_relative_path::get_background_dir_relative_path, get_config::get_config,
        get_jpg_image_infos::get_jpg_image_infos, get_manga_dir_data::get_manga_dir_data,
        open_image::open_image, remove_watermark::remove_watermark, save_config::save_config,
        show_path_in_file_manager::show_path_in_file_manager,
    };
}

mod generate_background;
mod get_background_dir_abs_path;
mod get_background_dir_relative_path;
mod get_config;
mod get_jpg_image_infos;
mod get_manga_dir_data;
mod open_image;
mod remove_watermark;
mod save_config;
mod show_path_in_file_manager;
