use crate::error::{MagnetarError, Result};
use gtk4_layer_shell::Layer;
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

pub const EXCLUSIVE_ZONE_AUTO: i32 = -1;

/// Configuración de una ventana Magnetar
#[derive(Debug, Clone)]
pub struct WindowConfig {
    pub name: String,
    pub layer: Layer,
    pub height: i32,
    pub width: i32,
    pub margin_top: i32,
    pub margin_bottom: i32,
    pub margin_left: i32,
    pub margin_right: i32,
    pub anchor_top: bool,
    pub anchor_bottom: bool,
    pub anchor_left: bool,
    pub anchor_right: bool,
    pub exclusive_zone: i32,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            name: "magnetar".into(),
            layer: Layer::Top,
            height: 40,
            width: 0,
            margin_top: 0,
            margin_bottom: 0,
            margin_left: 0,
            margin_right: 0,
            anchor_top: true,
            anchor_bottom: false,
            anchor_left: true,
            anchor_right: true,
            exclusive_zone: EXCLUSIVE_ZONE_AUTO,
        }
    }
}

impl WindowConfig {
    /// Parsea la configuración desde comentarios HTML
    /// Formato: <!-- magnetar: key=value, key=value -->
    pub fn from_html(html: &str, name: String) -> Self {
        let mut config = Self {
            name,
            ..Default::default()
        };

        let start = match html.find("<!-- magnetar:") {
            Some(i) => i,
            None => return config,
        };

        let end = match html[start..].find("-->") {
            Some(i) => start + i,
            None => return config,
        };

        let cfg_str = &html[start + 14..end];
        debug!("Parsing config: {}", cfg_str);

        for part in cfg_str.split(',') {
            let Some((key, value)) = part.trim().split_once('=') else {
                continue;
            };

            let key = key.trim();
            let value = value.trim();

            match key {
                "layer" => config.layer = Self::parse_layer(value),
                "height" => config.height = value.parse().unwrap_or(40),
                "width" => config.width = value.parse().unwrap_or(0),
                "margin_top" => config.margin_top = value.parse().unwrap_or(0),
                "margin_bottom" => config.margin_bottom = value.parse().unwrap_or(0),
                "margin_left" => config.margin_left = value.parse().unwrap_or(0),
                "margin_right" => config.margin_right = value.parse().unwrap_or(0),
                "exclusive_zone" => {
                    config.exclusive_zone = value.parse().unwrap_or(EXCLUSIVE_ZONE_AUTO)
                }
                "anchor" => Self::parse_anchors(&mut config, value),
                _ => warn!("Unknown config key: {}", key),
            }
        }

        config
    }

    fn parse_layer(value: &str) -> Layer {
        match value {
            "background" => Layer::Background,
            "bottom" => Layer::Bottom,
            "top" => Layer::Top,
            "overlay" => Layer::Overlay,
            _ => {
                warn!("Unknown layer '{}', using Overlay", value);
                Layer::Overlay
            }
        }
    }

    fn parse_anchors(config: &mut Self, value: &str) {
        config.anchor_top = false;
        config.anchor_bottom = false;
        config.anchor_left = false;
        config.anchor_right = false;

        for edge in value.split('|') {
            match edge.trim() {
                "top" => config.anchor_top = true,
                "bottom" => config.anchor_bottom = true,
                "left" => config.anchor_left = true,
                "right" => config.anchor_right = true,
                _ => warn!("Unknown anchor edge: {}", edge),
            }
        }
    }
}

/// Widget HTML con su configuración
#[derive(Debug)]
pub struct Widget {
    pub config: WindowConfig,
    pub html: String,
}

/// Carga widgets desde directorios
pub struct WidgetLoader {
    search_paths: Vec<PathBuf>,
}

impl WidgetLoader {
    pub fn new() -> Self {
        let mut search_paths = Vec::new();

        // Directorio de configuración del usuario
        if let Ok(home) = std::env::var("HOME") {
            search_paths.push(PathBuf::from(format!("{}/.config/magnetar", home)));
        }

        // Directorio actual
        search_paths.push(PathBuf::from("."));

        Self { search_paths }
    }

    /// Carga todos los widgets encontrados
    pub fn load_all(&self) -> Result<Vec<Widget>> {
        let mut widgets = Vec::new();

        for path in &self.search_paths {
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let entry_path = entry.path();

                    if entry_path.extension().and_then(|s| s.to_str()) == Some("html") {
                        match self.load_widget(&entry_path) {
                            Ok(widget) => widgets.push(widget),
                            Err(e) => {
                                warn!("Failed to load widget {}: {}", entry_path.display(), e)
                            }
                        }
                    }
                }
            }
        }

        Ok(widgets)
    }

    /// Carga un widget específico
    fn load_widget(&self, path: &Path) -> Result<Widget> {
        let html = std::fs::read_to_string(path)
            .map_err(|e| MagnetarError::Io(format!("Failed to read {}: {}", path.display(), e)))?;

        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("widget")
            .to_string();

        let config = WindowConfig::from_html(&html, name);

        debug!("Loaded widget: {} from {}", config.name, path.display());

        Ok(Widget { config, html })
    }

    /// Crea directorios de configuración si no existen
    pub fn ensure_config_dirs(&self) -> Result<()> {
        for path in &self.search_paths {
            if !path.exists() {
                std::fs::create_dir_all(path)?;
                debug!("Created config directory: {}", path.display());
            }
        }
        Ok(())
    }
}

impl Default for WidgetLoader {
    fn default() -> Self {
        Self::new()
    }
}
