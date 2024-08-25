use tauri::State;

use crate::download_manager::DownloadManager;
use crate::errors::CommandResult;
use crate::types::CommandResponse;

#[tauri::command(async)]
#[specta::specta]
pub async fn download_episodes(
    download_manager: State<'_, DownloadManager>,
    ep_ids: Vec<u32>,
) -> CommandResult<CommandResponse<()>> {
    for ep_id in ep_ids {
        download_manager.submit_episode(ep_id).await?;
    }

    Ok(CommandResponse {
        code: 0,
        msg: String::new(),
        data: (),
    })
}
