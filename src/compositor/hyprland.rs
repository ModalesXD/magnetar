use super::{Client, Compositor, Workspace};
use crate::error::{MagnetarError, Result};
use async_trait::async_trait;
use serde::Deserialize;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::UnixStream;
use tokio::process::Command;
use tracing::{debug, error, info, trace, warn};

pub struct Hyprland {
    // Cachear la signature para evitar lookups repetidos
    signature: Arc<String>,
}

#[derive(Debug, Clone)]
pub enum HyprlandEvent {
    WorkspaceChanged,
    ActiveWindowChanged { title: String, class: String },
    Unknown,
}

impl HyprlandEvent {
    pub fn parse(line: &str) -> Option<Self> {
        let line = line.trim();
        if line.is_empty() {
            return None;
        }

        let (event_type, data) = line.split_once(">>")?;

        match event_type {
            "workspace" | "focusedmon" => Some(HyprlandEvent::WorkspaceChanged),
            "activewindow" => {
                let (class, title) = data.split_once(',')?;
                Some(HyprlandEvent::ActiveWindowChanged {
                    class: class.to_string(),
                    title: title.to_string(),
                })
            }
            _ => {
                trace!("Unknown Hyprland event: {}", event_type);
                Some(HyprlandEvent::Unknown)
            }
        }
    }
}

#[derive(Deserialize)]
struct WorkspaceRaw {
    id: i32,
    name: String,
    monitor: String,
    windows: i32,
    #[serde(default)]
    urgent: bool,
}

#[derive(Deserialize)]
struct WorkspaceId {
    id: i32,
}

#[derive(Deserialize)]
struct ClientRaw {
    address: String,
    title: String,
    class: String,
    pid: i32,
    workspace: WorkspaceId,
    monitor: i32,
    #[serde(deserialize_with = "deserialize_bool_flexible")]
    floating: bool,
    #[serde(deserialize_with = "deserialize_bool_flexible")]
    fullscreen: bool,
    #[serde(rename = "focusHistoryID")]
    focus_history_id: i32,
}

fn deserialize_bool_flexible<'de, D>(deserializer: D) -> std::result::Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Unexpected};
    use serde_json::Value;

    match Value::deserialize(deserializer)? {
        Value::Bool(b) => Ok(b),
        Value::Number(n) => n.as_i64().map(|i| i != 0).ok_or_else(|| {
            de::Error::invalid_type(
                Unexpected::Other("non-integer number"),
                &"boolean or integer",
            )
        }),
        other => Err(de::Error::invalid_type(
            Unexpected::Other(&format!("{:?}", other)),
            &"boolean or integer",
        )),
    }
}

impl From<WorkspaceRaw> for Workspace {
    fn from(r: WorkspaceRaw) -> Self {
        Self {
            id: r.id,
            name: r.name,
            monitor: r.monitor,
            windows: r.windows,
            urgent: r.urgent,
        }
    }
}

impl From<ClientRaw> for Client {
    fn from(r: ClientRaw) -> Self {
        Self {
            address: r.address,
            title: r.title,
            class: r.class,
            pid: r.pid,
            workspace: r.workspace.id,
            monitor: r.monitor,
            floating: r.floating,
            fullscreen: r.fullscreen,
            focused: r.focus_history_id == 0,
        }
    }
}

impl Hyprland {
    pub async fn connect() -> Result<Self> {
        let signature = std::env::var("HYPRLAND_INSTANCE_SIGNATURE")
            .map_err(|_| MagnetarError::Compositor("HYPRLAND_INSTANCE_SIGNATURE not set".into()))?;

        debug!("Connected to Hyprland instance: {}", signature);

        Ok(Self {
            signature: Arc::new(signature),
        })
    }

    pub async fn subscribe_events<F>(callback: F) -> Result<()>
    where
        F: Fn(HyprlandEvent) + Send + 'static,
    {
        let signature = std::env::var("HYPRLAND_INSTANCE_SIGNATURE")
            .map_err(|_| MagnetarError::Compositor("HYPRLAND_INSTANCE_SIGNATURE not set".into()))?;

        let socket_path = Self::get_socket_path(&signature, ".socket2.sock");

        info!("Connecting to Hyprland event socket: {}", socket_path);

        let stream = UnixStream::connect(&socket_path).await.map_err(|e| {
            MagnetarError::Compositor(format!("Failed to connect to Hyprland socket: {}", e))
        })?;

        info!("Connected to Hyprland event stream");

        let mut lines = BufReader::new(stream).lines();

        crate::runtime::spawn(async move {
            while let Ok(Some(line)) = lines.next_line().await {
                trace!("Hyprland event: {}", line);
                if let Some(event) = HyprlandEvent::parse(&line) {
                    callback(event);
                }
            }
            warn!("Hyprland event stream closed");
        });

        Ok(())
    }

    /// Obtiene la ruta del socket de Hyprland
    fn get_socket_path(signature: &str, socket_name: &str) -> String {
        if let Ok(xdg_runtime) = std::env::var("XDG_RUNTIME_DIR") {
            let xdg_path = format!("{}/hypr/{}/{}", xdg_runtime, signature, socket_name);
            if std::path::Path::new(&xdg_path).exists() {
                return xdg_path;
            }
        }

        format!("/tmp/hypr/{}/{}", signature, socket_name)
    }

    async fn run_hyprctl(&self, args: &[&str]) -> Result<Vec<u8>> {
        trace!("hyprctl {:?}", args);

        let output = Command::new("hyprctl")
            .args(args)
            .output()
            .await
            .map_err(|e| {
                error!("Failed to execute hyprctl: {}", e);
                MagnetarError::Compositor(format!("hyprctl execution failed: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("hyprctl failed: {}", stderr);
            return Err(MagnetarError::Compositor(stderr.to_string()));
        }

        Ok(output.stdout)
    }

    async fn exec<T: for<'de> Deserialize<'de>>(&self, args: &[&str]) -> Result<T> {
        let bytes = self.run_hyprctl(args).await?;
        serde_json::from_slice(&bytes).map_err(|e| {
            error!("Failed to deserialize hyprctl output: {}", e);
            MagnetarError::Compositor(format!("JSON parse error: {}", e))
        })
    }
}

#[async_trait]
impl Compositor for Hyprland {
    async fn workspaces(&self) -> Result<Vec<Workspace>> {
        let raw: Vec<WorkspaceRaw> = self.exec(&["-j", "workspaces"]).await?;
        Ok(raw.into_iter().map(Workspace::from).collect())
    }

    async fn activeworkspace(&self) -> Result<Workspace> {
        let raw: WorkspaceRaw = self.exec(&["-j", "activeworkspace"]).await?;
        Ok(Workspace::from(raw))
    }

    async fn activewindow(&self) -> Result<Client> {
        let raw: ClientRaw = self.exec(&["-j", "activewindow"]).await?;
        Ok(Client::from(raw))
    }

    async fn clients(&self) -> Result<Vec<Client>> {
        let raw: Vec<ClientRaw> = self.exec(&["-j", "clients"]).await?;
        Ok(raw.into_iter().map(Client::from).collect())
    }

    async fn dispatch(&self, command: &str) -> Result<()> {
        self.run_hyprctl(&["dispatch", command]).await.map(|_| ())
    }
}
