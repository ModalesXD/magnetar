use tracing::Level;
use tracing_subscriber::EnvFilter;

/// Configuración del sistema de logging
#[derive(Debug, Clone)]
pub struct LogConfig {
    pub level: Level,
    pub show_target: bool,
    pub show_thread_ids: bool,
    pub show_thread_names: bool,
    pub show_line_number: bool,
    pub show_file: bool,
    pub compact: bool,
    pub use_colors: bool,
    pub filter_noisy_crates: bool,
}

impl LogConfig {
    /// Configuración para desarrollo (verbose)
    pub fn debug() -> Self {
        Self {
            level: Level::DEBUG,
            show_target: true,
            show_thread_ids: false,
            show_thread_names: true,
            show_line_number: true,
            show_file: false,
            compact: true,
            use_colors: true,
            filter_noisy_crates: true,
        }
    }

    /// Configuración para producción (menos verbose)
    pub fn release() -> Self {
        Self {
            level: Level::INFO,
            show_target: false,
            show_thread_ids: false,
            show_thread_names: false,
            show_line_number: false,
            show_file: false,
            compact: true,
            use_colors: true,
            filter_noisy_crates: true,
        }
    }

    /// Configuración mínima (solo errores)
    pub fn minimal() -> Self {
        Self {
            level: Level::ERROR,
            show_target: false,
            show_thread_ids: false,
            show_thread_names: false,
            show_line_number: false,
            show_file: false,
            compact: true,
            use_colors: false,
            filter_noisy_crates: true,
        }
    }
}

impl Default for LogConfig {
    fn default() -> Self {
        #[cfg(debug_assertions)]
        return Self::debug();

        #[cfg(not(debug_assertions))]
        return Self::release();
    }
}

/// Inicializa el sistema de logging con la configuración especificada
pub fn init(config: LogConfig) {
    let mut filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("{}={}", env!("CARGO_PKG_NAME"), config.level)));

    // Filtrar crates ruidosos de GTK/WebKit
    if config.filter_noisy_crates {
        filter = filter
            .add_directive("gtk=warn".parse().unwrap())
            .add_directive("webkit6=warn".parse().unwrap())
            .add_directive("glib=warn".parse().unwrap())
            .add_directive("gdk=warn".parse().unwrap())
            .add_directive("gio=warn".parse().unwrap());
    }

    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(config.show_target)
        .with_thread_ids(config.show_thread_ids)
        .with_thread_names(config.show_thread_names)
        .with_line_number(config.show_line_number)
        .with_file(config.show_file)
        .with_level(true)
        .with_ansi(config.use_colors);

    if config.compact {
        subscriber.compact().init();
    } else {
        subscriber.init();
    }
}

/// Inicializa logging con configuración de debug
pub fn init_debug() {
    init(LogConfig::debug());
}

/// Inicializa logging con configuración de release
pub fn init_release() {
    init(LogConfig::release());
}

/// Inicializa logging con configuración por defecto (debug en dev, release en prod)
pub fn init_default() {
    init(LogConfig::default());
}
