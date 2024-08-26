use tauri::State;

use crate::download_manager::DownloadManager;
use crate::errors::CommandResult;
use crate::types::{CommandResponse, Episode};

#[tauri::command(async)]
#[specta::specta]
pub async fn download_episodes(
    download_manager: State<'_, DownloadManager>,
    episodes: Vec<Episode>,
) -> CommandResult<CommandResponse<()>> {
    for ep in episodes {
        download_manager.submit_episode(ep).await?;
    }

    Ok(CommandResponse {
        code: 0,
        msg: String::new(),
        data: (),
    })
}
