use std::path::PathBuf;

pub fn get_exe_dir_path() -> anyhow::Result<PathBuf> {
    let exe_path = std::env::current_exe()?;
    let exe_dir = exe_path
        .parent()
        .ok_or(anyhow::anyhow!("exe path parent not found"))?
        .to_path_buf();
    Ok(exe_dir)
}
