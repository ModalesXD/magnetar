use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;

mod commands;

#[derive(Parser)]
#[command(name = "magnetar")]
#[command(author, version, about = "Wayland shell compositor widget manager", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Nivel de logging (error, warn, info, debug, trace)
    #[arg(short, long, global = true)]
    pub log_level: Option<String>,

    /// Desactivar colores en la salida
    #[arg(long, global = true)]
    pub no_color: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Inicia la shell de Magnetar (modo por defecto)
    Run {
        /// Directorio de configuración personalizado
        #[arg(short, long)]
        config_dir: Option<PathBuf>,
    },

    /// Gestión de widgets
    #[command(subcommand)]
    Widget(WidgetCommands),

    /// Comandos del compositor (Hyprland)
    #[command(subcommand)]
    Compositor(CompositorCommands),

    /// Información del sistema y diagnósticos
    Info {
        /// Mostrar información detallada
        #[arg(short, long)]
        verbose: bool,
    },

    /// Crear un nuevo widget desde una plantilla
    New {
        /// Nombre del widget
        name: String,

        /// Tipo de plantilla (topbar, sidebar, overlay, notification)
        #[arg(short, long, default_value = "topbar")]
        template: String,

        /// Directorio de salida
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Validar configuración de widgets
    Validate {
        /// Archivo HTML del widget a validar
        file: PathBuf,
    },
}

#[derive(Subcommand)]
pub enum WidgetCommands {
    /// Lista todos los widgets disponibles
    List {
        /// Mostrar información detallada
        #[arg(short, long)]
        verbose: bool,
    },

    /// Recargar widgets sin reiniciar
    Reload {
        /// Nombre del widget específico a recargar
        name: Option<String>,
    },

    /// Inspeccionar configuración de un widget
    Inspect {
        /// Nombre o ruta del widget
        widget: String,
    },
}

#[derive(Subcommand)]
pub enum CompositorCommands {
    /// Información del compositor actual
    Info,

    /// Lista todos los workspaces
    Workspaces {
        /// Formato de salida (table, json)
        #[arg(short, long, default_value = "table")]
        format: String,
    },

    /// Lista todas las ventanas/clientes
    Clients {
        /// Formato de salida (table, json)
        #[arg(short, long, default_value = "table")]
        format: String,

        /// Filtrar por workspace
        #[arg(short, long)]
        workspace: Option<i32>,
    },

    /// Workspace activo
    Active,

    /// Ejecutar comando del compositor
    Exec {
        /// Comando a ejecutar (ej: "workspace 1")
        command: Vec<String>,
    },
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Configurar colores
    if cli.no_color {
        colored::control::set_override(false);
    }

    match cli.command {
        Some(Commands::Widget(cmd)) => {
            commands::widget::execute(cmd).await?;
            std::process::exit(0);
        }
        Some(Commands::Compositor(cmd)) => {
            commands::compositor::execute(cmd).await?;
            std::process::exit(0);
        }
        Some(Commands::Info { verbose }) => {
            commands::info::execute(verbose).await?;
            std::process::exit(0);
        }
        Some(Commands::New {
            name,
            template,
            output,
        }) => {
            commands::new::execute(name, template, output).await?;
            std::process::exit(0);
        }
        Some(Commands::Validate { file }) => {
            commands::validate::execute(file).await?;
            std::process::exit(0);
        }
        Some(Commands::Run { config_dir: _ }) | None => {
            // Sin comando o comando 'run' = ejecutar la GUI
            Ok(())
        }
    }
}

/// Utilidad para imprimir errores con formato
pub fn print_error(msg: &str) {
    eprintln!("{} {}", "✗".red().bold(), msg.red());
}

/// Utilidad para imprimir éxito con formato
pub fn print_success(msg: &str) {
    println!("{} {}", "✓".green().bold(), msg.green());
}

/// Utilidad para imprimir información con formato
pub fn print_info(msg: &str) {
    println!("{} {}", "ℹ".blue().bold(), msg);
}

/// Utilidad para imprimir advertencias con formato
pub fn print_warning(msg: &str) {
    println!("{} {}", "⚠".yellow().bold(), msg.yellow());
}
