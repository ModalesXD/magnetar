use async_channel::Sender as AsyncSender;
use gtk4::glib;
use serde_json::{json, Value};
use std::{
    cell::RefCell,
    collections::HashMap,
    future::Future,
    pin::Pin,
    rc::Rc,
    sync::{Arc, Mutex},
};
use tracing::{debug, error, trace, warn};
use webkit6::{
    prelude::*, UserContentInjectedFrames, UserContentManager, UserScript, UserScriptInjectionTime,
    WebView,
};

pub type HandlerFut = Pin<Box<dyn Future<Output = Value> + Send>>;
pub type Handler = Arc<dyn Fn(Value) -> HandlerFut + Send + Sync>;
pub type LocalHandler = Rc<dyn Fn(Value) -> Value>;

#[derive(Clone)]
pub struct IPCHandler {
    handlers: Arc<Mutex<HashMap<String, Handler>>>,
    local_handlers: Rc<RefCell<HashMap<String, LocalHandler>>>,
    manager: UserContentManager,
    eval_tx: Arc<Mutex<Option<AsyncSender<String>>>>,
}

impl IPCHandler {
    /// Paso 1: crea el manager y registra el preload.
    /// Llamar ANTES de construir el WebView.
    pub fn new() -> Self {
        let manager = UserContentManager::new();
        manager.register_script_message_handler("magnetar", None);

        let script = UserScript::new(
            include_str!("preload.js"),
            UserContentInjectedFrames::TopFrame,
            UserScriptInjectionTime::Start,
            &[],
            &[],
        );
        manager.add_script(&script);

        let handlers: Arc<Mutex<HashMap<String, Handler>>> = Arc::new(Mutex::new(HashMap::new()));
        let local_handlers: Rc<RefCell<HashMap<String, LocalHandler>>> =
            Rc::new(RefCell::new(HashMap::new()));
        let eval_tx: Arc<Mutex<Option<AsyncSender<String>>>> = Arc::new(Mutex::new(None));

        let handlers_clone = handlers.clone();
        let local_handlers_clone = local_handlers.clone();
        let eval_tx_clone = eval_tx.clone();

        manager.connect_script_message_received(Some("magnetar"), move |_manager, message| {
            let payload = message.to_string();
            trace!("IPC message received: {}", payload);

            let parsed: Value = match serde_json::from_str(&payload) {
                Ok(v) => v,
                Err(e) => {
                    error!("Failed to parse IPC message: {} — payload: {}", e, payload);
                    return;
                }
            };

            let command = parsed
                .get("command")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let data = parsed.get("data").cloned().unwrap_or(Value::Null);

            let id = parsed
                .get("id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            debug!("IPC call: command={}, id={:?}", command, id);

            let local_handler = {
                let local_handlers = local_handlers_clone.borrow();
                local_handlers.get(&command).cloned()
            };

            if let Some(handler) = local_handler {
                let result = handler(data);
                trace!("IPC local handler result: {}", result);

                if let Some(msg_id) = id {
                    let tx_opt = eval_tx_clone.lock().unwrap().clone();
                    if let Some(tx) = tx_opt {
                        let response = json!({ "id": msg_id, "data": result });
                        let js = format!(
                            "if (window.__magnetar) {{ window.__magnetar({}); }}",
                            response
                        );
                        glib::MainContext::default().spawn_local(async move {
                            let _ = tx.send(js).await;
                        });
                    }
                }
                return;
            }

            let handler = {
                let handlers = handlers_clone.lock().unwrap();
                handlers.get(&command).cloned()
            };

            match handler {
                Some(handler) => {
                    let tx_opt = eval_tx_clone.lock().unwrap().clone();
                    let Some(tx) = tx_opt else {
                        warn!("IPC not connected to webview yet, dropping: {}", command);
                        return;
                    };

                    // Spawn en el runtime compartido
                    crate::runtime::spawn(async move {
                        let result = handler(data).await;
                        trace!("IPC handler result: {}", result);

                        if let Some(msg_id) = id {
                            let response = json!({ "id": msg_id, "data": result });
                            let js = format!(
                                "if (window.__magnetar) {{ window.__magnetar({}); }}",
                                response
                            );
                            let _ = tx.send(js).await;
                        }
                    });
                }
                None => {
                    warn!("No handler registered for command: {}", command);
                    if tracing::enabled!(tracing::Level::DEBUG) {
                        let available: Vec<String> =
                            handlers_clone.lock().unwrap().keys().cloned().collect();
                        debug!("Available IPC handlers: {:?}", available);
                    }
                }
            }
        });

        Self {
            handlers,
            local_handlers,
            manager,
            eval_tx,
        }
    }

    /// Paso 2: conecta el WebView al canal eval.
    /// Llamar DESPUÉS de construir el WebView con ipc.manager().
    pub fn connect_webview(&self, webview: &WebView) {
        // Transparencia — fondo negro con alpha 0.
        // Sin esto webkit pinta fondo blanco aunque el HTML diga transparent.
        webview.set_background_color(&gtk4::gdk::RGBA::new(0.0, 0.0, 0.0, 0.0));

        let (tx, rx) = async_channel::unbounded::<String>();
        *self.eval_tx.lock().unwrap() = Some(tx);

        let wv = webview.clone();
        glib::MainContext::default().spawn_local(async move {
            while let Ok(js) = rx.recv().await {
                wv.evaluate_javascript(
                    &js,
                    None,
                    None,
                    None::<&gtk4::gio::Cancellable>,
                    |result| {
                        if let Err(e) = result {
                            error!("evaluate_javascript failed: {:?}", e);
                        }
                    },
                );
            }
            debug!("IPC eval loop terminated");
        });

        debug!("IPC connected to webview");
    }

    pub fn emit(&self, event: &str, data: Value) {
        let tx_opt = self.eval_tx.lock().unwrap().clone();
        let Some(tx) = tx_opt else {
            warn!("Cannot emit '{}': webview not connected", event);
            return;
        };

        let payload = json!({ "event": event, "data": data });
        let js = format!(
            "if (window.__magnetar) {{ window.__magnetar({}); }}",
            payload
        );

        glib::MainContext::default().spawn_local(async move {
            let _ = tx.send(js).await;
        });
    }

    pub fn register<F>(&self, name: impl Into<String>, handler: F)
    where
        F: Fn(Value) -> HandlerFut + Send + Sync + 'static,
    {
        let name = name.into();
        debug!("Registering IPC handler: {}", name);
        self.handlers
            .lock()
            .unwrap()
            .insert(name, Arc::new(handler));
    }

    pub fn register_local<F>(&self, name: impl Into<String>, handler: F)
    where
        F: Fn(Value) -> Value + 'static,
    {
        let name = name.into();
        debug!("Registering local IPC handler: {}", name);
        self.local_handlers
            .borrow_mut()
            .insert(name, Rc::new(handler));
    }

    pub fn manager(&self) -> &UserContentManager {
        &self.manager
    }
}
