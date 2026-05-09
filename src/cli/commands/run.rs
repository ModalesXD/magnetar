use std::path::PathBuf;

pub async fn execute(_config_dir: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    // Esta función será llamada desde main.rs para iniciar la GUI
    // Por ahora solo retornamos Ok para indicar que debe ejecutarse el modo GUI
    Ok(())
}
