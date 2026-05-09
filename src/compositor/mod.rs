pub mod handlers;
pub mod hyprland;

use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: i32,
    pub name: String,
    pub monitor: String,
    pub windows: i32,
    pub urgent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Client {
    pub address: String,
    pub title: String,
    pub class: String,
    pub pid: i32,
    pub workspace: i32,
    pub monitor: i32,
    pub floating: bool,
    pub fullscreen: bool,
    pub focused: bool,
}

#[async_trait]
pub trait Compositor: Send + Sync {
    async fn workspaces(&self) -> Result<Vec<Workspace>>;
    async fn activeworkspace(&self) -> Result<Workspace>;
    async fn activewindow(&self) -> Result<Client>;
    async fn clients(&self) -> Result<Vec<Client>>;
    async fn dispatch(&self, command: &str) -> Result<()>;
}

pub type SharedCompositor = Arc<dyn Compositor>;

pub struct CompositorManager {
    compositor: RwLock<Option<SharedCompositor>>,
}

impl CompositorManager {
    pub fn new() -> Self {
        Self {
            compositor: RwLock::new(None),
        }
    }

    pub async fn detect(&self) -> Result<()> {
        if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
            info!("Hyprland compositor detected");
            let compositor = Arc::new(hyprland::Hyprland::connect().await?);
            *self.compositor.write().unwrap() = Some(compositor);
            return Ok(());
        }

        Err(crate::error::MagnetarError::Compositor(
            "No compositor detected".into(),
        ))
    }

    pub async fn get(&self) -> Option<SharedCompositor> {
        self.compositor.read().unwrap().clone()
    }
}

impl Default for CompositorManager {
    fn default() -> Self {
        Self::new()
    }
}
