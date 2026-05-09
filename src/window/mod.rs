use crate::{config::WindowConfig, ipc::IPCHandler};
use gtk4::{self as gtk, glib, prelude::*};
use gtk4_layer_shell::{Edge, LayerShell};
use serde_json::json;
use std::rc::Rc;
use tracing::debug;
use webkit6::WebView;

pub struct MagnetarWindow {
    pub window: gtk::ApplicationWindow,
    pub webview: WebView,
    pub ipc: IPCHandler,
}

impl MagnetarWindow {
    pub fn new(app: &gtk::Application, config: WindowConfig) -> Self {
        debug!(
            "Creating window: name={}, layer={:?}, height={}",
            config.name, config.layer, config.height
        );

        // Paso 1: IPC crea el manager y registra el preload
        let ipc = IPCHandler::new();

        // Paso 2: WebView con el manager — preload activo desde el inicio
        let webview = WebView::builder()
            .user_content_manager(ipc.manager())
            .vexpand(true)
            .hexpand(true)
            .build();

        // Paso 3: conectar webview — arranca el canal eval y setea transparencia
        ipc.connect_webview(&webview);

        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .title(&config.name)
            .default_height(config.height)
            .build();

        // Ancho explícito solo si se especificó
        if config.width > 0 {
            window.set_default_width(config.width);
        }

        window.init_layer_shell();
        window.set_layer(config.layer);

        window.set_anchor(Edge::Top, config.anchor_top);
        window.set_anchor(Edge::Bottom, config.anchor_bottom);
        window.set_anchor(Edge::Left, config.anchor_left);
        window.set_anchor(Edge::Right, config.anchor_right);

        window.set_margin(Edge::Top, config.margin_top);
        window.set_margin(Edge::Bottom, config.margin_bottom);
        window.set_margin(Edge::Left, config.margin_left);
        window.set_margin(Edge::Right, config.margin_right);

        let zone = if config.exclusive_zone == crate::config::EXCLUSIVE_ZONE_AUTO {
            config.height
        } else {
            config.exclusive_zone
        };
        window.set_exclusive_zone(zone);

        window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::OnDemand);

        let settings = webkit6::prelude::WebViewExt::settings(&webview).unwrap();
        settings.set_enable_developer_extras(true);
        settings.set_javascript_can_access_clipboard(true);
        settings.set_enable_write_console_messages_to_stdout(true);

        webview.set_can_focus(true);
        webview.set_focusable(true);
        webview.set_focus_on_click(true);

        window.set_child(Some(&webview));

        // CSS transparente en la ventana GTK
        let css = gtk::CssProvider::new();
        css.load_from_data("window { background: transparent; }");
        gtk::style_context_add_provider_for_display(
            &gtk::prelude::WidgetExt::display(&window),
            &css,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        Self {
            window,
            webview,
            ipc,
        }
    }

    /// Registra handlers IPC para controlar esta ventana desde JavaScript
    ///
    /// Handlers disponibles:
    /// - `window.resize`  { width?: number, height?: number }
    /// - `window.show`    {}
    /// - `window.hide`    {}
    ///
    /// Ejemplo desde JS:
    ///   magnetar.invoke("window.resize", { width: 300 })
    ///   magnetar.invoke("window.hide")
    pub fn register_window_handlers(self: &Rc<Self>) {
        {
            let window = self.window.clone();
            self.ipc.register_local("window.resize", move |value| {
                let width = value.get("width").and_then(|v| v.as_i64());
                let height = value.get("height").and_then(|v| v.as_i64());

                if let Some(w) = width {
                    window.set_default_width(w as i32);
                }
                if let Some(h) = height {
                    window.set_default_height(h as i32);
                }

                json!({ "success": true })
            });
        }

        {
            let window = self.window.clone();
            self.ipc.register_local("window.show", move |_| {
                window.set_visible(true);
                json!({ "success": true })
            });
        }

        {
            let window = self.window.clone();
            self.ipc.register_local("window.hide", move |_| {
                window.set_visible(false);
                json!({ "success": true })
            });
        }
    }

    pub fn show(&self) {
        self.window.present();
        glib::idle_add_local_once({
            let webview = self.webview.clone();
            move || {
                let _ = webview.grab_focus();
            }
        });
    }
}
