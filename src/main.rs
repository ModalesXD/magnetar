mod cli;
mod compositor;
mod config;
mod error;
mod ipc;
mod logging;
mod runtime;
mod window;

use compositor::CompositorManager;
use config::WidgetLoader;
use gtk4::{self as gtk, glib, prelude::*};
use std::{cell::RefCell, rc::Rc};
use tracing::{error, info, warn};
use webkit6::prelude::WebViewExt;
use window::MagnetarWindow;

thread_local! {
    static WINDOWS: RefCell<Vec<Rc<MagnetarWindow>>> = RefCell::new(Vec::new());
}

fn main() {
    // Crear runtime de Tokio para ejecutar el CLI async
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Ejecutar CLI de forma async
    let cli_result = rt.block_on(cli::run());

    // Si el CLI retorna Ok(()), ejecutar la GUI
    // Si retorna Err, el comando CLI ya se ejecutó
    match cli_result {
        Ok(_) => {
            // Ejecutar GUI
            run_gui();
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_gui() {
    logging::init_default();
    info!("=== Magnetar Starting ===");

    let app = gtk::Application::builder()
        .application_id("com.magnetar.shell")
        .build();

    app.connect_activate(|app| build_ui(app));
    app.run();
}

fn build_ui(app: &gtk::Application) {
    let loader = WidgetLoader::new();

    if let Err(e) = loader.ensure_config_dirs() {
        error!("Failed to create config directories: {}", e);
    }

    let widgets = match loader.load_all() {
        Ok(w) if !w.is_empty() => w,
        Ok(_) => {
            info!("No widgets found, using default topbar");
            vec![create_default_widget()]
        }
        Err(e) => {
            error!("Failed to load widgets: {}", e);
            vec![create_default_widget()]
        }
    };

    info!("Loading {} widget(s)", widgets.len());

    for widget in widgets {
        info!("Loading widget: {}", widget.config.name);
        load_widget(app, widget.config, widget.html);
    }
}

fn create_default_widget() -> config::Widget {
    let html = std::fs::read_to_string("topbar.html").unwrap_or_else(|_| {
        r#"<html><body style="background:#1e1e2e;color:#cdd6f4;padding:20px;">
            <h1>ERROR: topbar.html not found</h1>
        </body></html>"#
            .to_string()
    });

    config::Widget {
        config: config::WindowConfig {
            name: "topbar".into(),
            layer: gtk4_layer_shell::Layer::Overlay,
            height: 40,
            ..config::WindowConfig::default()
        },
        html,
    }
}

fn load_widget(app: &gtk::Application, config: config::WindowConfig, html: String) {
    let window = Rc::new(MagnetarWindow::new(app, config));

    window.register_window_handlers();

    WINDOWS.with(|windows| {
        windows.borrow_mut().push(window.clone());
    });

    register_broadcast_handler(&window);

    window.show();

    let setup_done = Rc::new(std::cell::Cell::new(false));
    let window_for_load = window.clone();
    let setup_done_clone = setup_done.clone();

    window.webview.connect_load_changed(move |_wv, event| {
        if event == webkit6::LoadEvent::Finished {
            if !setup_done_clone.get() {
                setup_done_clone.set(true);
                setup_compositor_events(window_for_load.clone());
            }
        }
    });

    window.webview.load_html(&html, None);
}

fn register_broadcast_handler(window: &Rc<MagnetarWindow>) {
    window.ipc.register_local("broadcast", |value| {
        let event = value.get("event").and_then(|v| v.as_str()).unwrap_or("");
        let data = value
            .get("data")
            .cloned()
            .unwrap_or(serde_json::Value::Null);

        WINDOWS.with(|windows| {
            for win in windows.borrow().iter() {
                win.ipc.emit(event, data.clone());
            }
        });

        serde_json::json!({ "success": true })
    });
}

#[derive(Debug, Clone)]
enum FrontendEvent {
    WorkspaceChanged { id: i32 },
    ActiveWindowChanged { title: String, class: String },
}

fn setup_compositor_events(window: Rc<MagnetarWindow>) {
    let ipc = window.ipc.clone();

    let (tx, rx) = async_channel::unbounded::<FrontendEvent>();

    runtime::spawn(async move {
        let manager = CompositorManager::new();

        if let Err(e) = manager.detect().await {
            warn!("Compositor not detected: {}", e);
            return;
        }

        let Some(compositor) = manager.get().await else {
            return;
        };

        let tx_clone = tx.clone();
        let comp_clone = compositor.clone();

        if let Err(e) = compositor::hyprland::Hyprland::subscribe_events(move |event| {
            let tx = tx_clone.clone();
            let comp = comp_clone.clone();

            runtime::spawn(async move {
                match event {
                    compositor::hyprland::HyprlandEvent::WorkspaceChanged => {
                        if let Ok(ws) = comp.activeworkspace().await {
                            let _ = tx.send(FrontendEvent::WorkspaceChanged { id: ws.id }).await;
                        }
                    }
                    compositor::hyprland::HyprlandEvent::ActiveWindowChanged { title, class } => {
                        let _ = tx
                            .send(FrontendEvent::ActiveWindowChanged { title, class })
                            .await;
                    }
                    compositor::hyprland::HyprlandEvent::Unknown => {}
                }
            });
        })
        .await
        {
            error!("Failed to subscribe to Hyprland events: {}", e);
        }
    });

    let ipc_for_handlers = ipc.clone();
    glib::MainContext::default().spawn_local(async move {
        let manager = CompositorManager::new();
        if manager.detect().await.is_ok() {
            if let Some(compositor) = manager.get().await {
                compositor::handlers::register_all(&ipc_for_handlers, compositor);
            }
        }
    });

    glib::MainContext::default().spawn_local(async move {
        while let Ok(event) = rx.recv().await {
            match event {
                FrontendEvent::WorkspaceChanged { id } => {
                    info!("Emitting workspace:changed id={}", id);
                    ipc.emit("workspace:changed", serde_json::json!({ "id": id }));
                }
                FrontendEvent::ActiveWindowChanged { title, class } => {
                    info!(
                        "Emitting activewindow:changed title={} class={}",
                        title, class
                    );
                    ipc.emit(
                        "activewindow:changed",
                        serde_json::json!({ "title": title, "class": class }),
                    );
                }
            }
        }
    });
}
