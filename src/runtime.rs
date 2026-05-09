use once_cell::sync::Lazy;
use tokio::runtime::Runtime;
use tracing::info;

/// Runtime compartido de Tokio optimizado para Magnetar
///
/// Configuración:
/// - 2 worker threads (suficiente para IPC y compositor)
/// - Thread names descriptivos para debugging
/// - Todas las features habilitadas
pub static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    info!("Initializing Tokio runtime with 2 worker threads");

    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .thread_name("magnetar-async")
        .enable_all()
        .build()
        .expect("Failed to build Tokio runtime")
});

/// Ejecuta una tarea async en el runtime compartido
#[inline]
pub fn spawn<F>(future: F) -> tokio::task::JoinHandle<F::Output>
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    RUNTIME.spawn(future)
}

/// Ejecuta una tarea async y bloquea hasta obtener el resultado
/// ADVERTENCIA: No usar desde contextos async
#[inline]
pub fn block_on<F>(future: F) -> F::Output
where
    F: std::future::Future,
{
    RUNTIME.block_on(future)
}
