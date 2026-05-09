use super::SharedCompositor;
use crate::ipc::IPCHandler;
use serde_json::json;
use tracing::{error, info};

pub fn register_all(ipc: &IPCHandler, compositor: SharedCompositor) {
    info!("Registering compositor IPC handlers");
    register_workspaces(ipc, compositor.clone());
    register_activeworkspace(ipc, compositor.clone());
    register_activewindow(ipc, compositor.clone());
    register_clients(ipc, compositor.clone());
    register_dispatch(ipc, compositor);
}

fn register_workspaces(ipc: &IPCHandler, compositor: SharedCompositor) {
    ipc.register("hyprland.workspaces", move |_| {
        let c = compositor.clone();
        Box::pin(async move {
            match c.workspaces().await {
                Ok(ws) => json!(ws),
                Err(e) => {
                    error!("workspaces error: {}", e);
                    json!({ "error": e.to_string() })
                }
            }
        })
    });
}

fn register_activeworkspace(ipc: &IPCHandler, compositor: SharedCompositor) {
    ipc.register("hyprland.activeworkspace", move |_| {
        let c = compositor.clone();
        Box::pin(async move {
            match c.activeworkspace().await {
                Ok(ws) => json!({ "id": ws.id, "name": ws.name, "monitor": ws.monitor }),
                Err(e) => {
                    error!("activeworkspace error: {}", e);
                    json!({ "error": e.to_string() })
                }
            }
        })
    });
}

fn register_activewindow(ipc: &IPCHandler, compositor: SharedCompositor) {
    ipc.register("hyprland.activewindow", move |_| {
        let c = compositor.clone();
        Box::pin(async move {
            match c.activewindow().await {
                Ok(w) => json!({ "title": w.title, "class": w.class }),
                Err(e) => {
                    error!("activewindow error: {}", e);
                    json!({ "error": e.to_string() })
                }
            }
        })
    });
}

fn register_clients(ipc: &IPCHandler, compositor: SharedCompositor) {
    ipc.register("hyprland.clients", move |_| {
        let c = compositor.clone();
        Box::pin(async move {
            match c.clients().await {
                Ok(clients) => json!(clients),
                Err(e) => {
                    error!("clients error: {}", e);
                    json!({ "error": e.to_string() })
                }
            }
        })
    });
}

fn register_dispatch(ipc: &IPCHandler, compositor: SharedCompositor) {
    ipc.register("hyprland.exec", move |value| {
        let command = value.as_str().unwrap_or("").to_string();
        info!("Dispatching command: {}", command);
        let c = compositor.clone();
        Box::pin(async move {
            match c.dispatch(&command).await {
                Ok(_) => json!({ "success": true }),
                Err(e) => {
                    error!("dispatch failed: {}", e);
                    json!({ "success": false, "error": e.to_string() })
                }
            }
        })
    });
}
