use crate::cli::{print_info, print_success};
use colored::Colorize;

pub async fn execute(verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "\n{}",
        "Magnetar - Información del Sistema".bold().underline()
    );
    println!();

    // Información básica
    println!(
        "  {} {}",
        "Versión:".cyan(),
        env!("CARGO_PKG_VERSION").green()
    );
    println!("  {} {}", "Autor:".cyan(), env!("CARGO_PKG_AUTHORS"));
    println!();

    // Compositor
    print_info("Compositor:");
    if let Ok(signature) = std::env::var("HYPRLAND_INSTANCE_SIGNATURE") {
        println!("    {} Hyprland", "✓".green());
        if verbose {
            println!("    {} {}", "Instancia:".dimmed(), signature.dimmed());
        }
    } else {
        println!("    {} No detectado", "✗".red());
    }
    println!();

    // Directorios
    print_info("Directorios:");
    if let Ok(home) = std::env::var("HOME") {
        let config_dir = format!("{}/.config/magnetar", home);
        println!("    {} {}", "Config:".dimmed(), config_dir.yellow());

        if verbose {
            let exists = std::path::Path::new(&config_dir).exists();
            let status = if exists { "✓".green() } else { "✗".red() };
            println!("    {} {}", "Existe:".dimmed(), status);
        }
    }
    println!();

    // Variables de entorno relevantes
    if verbose {
        print_info("Variables de entorno:");

        let env_vars = ["WAYLAND_DISPLAY", "XDG_RUNTIME_DIR", "XDG_SESSION_TYPE"];

        for var in env_vars {
            if let Ok(value) = std::env::var(var) {
                println!("    {} {}", format!("{}:", var).dimmed(), value.yellow());
            }
        }
        println!();
    }

    // Widgets
    let loader = crate::config::WidgetLoader::new();
    match loader.load_all() {
        Ok(widgets) => {
            print_success(&format!("Widgets encontrados: {}", widgets.len()));
            if verbose && !widgets.is_empty() {
                for widget in widgets {
                    println!("    {} {}", "•".blue(), widget.config.name.cyan());
                }
            }
        }
        Err(e) => {
            println!("    {} Error al cargar widgets: {}", "✗".red(), e);
        }
    }
    println!();

    Ok(())
}
