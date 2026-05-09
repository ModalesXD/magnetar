use crate::cli::{print_error, print_success, print_warning};
use crate::config::WindowConfig;
use colored::Colorize;
use std::path::PathBuf;

pub async fn execute(file: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    if !file.exists() {
        print_error(&format!("Archivo no encontrado: {}", file.display()));
        return Ok(());
    }

    let html = std::fs::read_to_string(&file)?;
    let name = file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    println!(
        "\n{} {}\n",
        "Validando:".bold(),
        file.display().to_string().cyan()
    );

    // Validar que sea HTML
    if !html.contains("<html") && !html.contains("<HTML") {
        print_warning("El archivo no parece contener HTML válido");
    }

    // Parsear configuración
    let config = WindowConfig::from_html(&html, name);

    // Validaciones
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    // Validar tamaño
    if config.height == 0 && config.width == 0 {
        warnings.push("Tanto height como width son 0, el widget podría no ser visible");
    }

    // Validar anclajes
    let anchor_count = [
        config.anchor_top,
        config.anchor_bottom,
        config.anchor_left,
        config.anchor_right,
    ]
    .iter()
    .filter(|&&x| x)
    .count();

    if anchor_count == 0 {
        errors.push("No hay anclajes configurados, el widget no se posicionará correctamente");
    }

    // Validar zona exclusiva
    if config.exclusive_zone > 0 && config.exclusive_zone < config.height {
        warnings.push("La zona exclusiva es menor que la altura del widget");
    }

    // Validar que tenga contenido
    if html.len() < 100 {
        warnings.push("El archivo HTML es muy pequeño, podría estar incompleto");
    }

    // Mostrar configuración
    println!("  {} {}", "Nombre:".cyan(), config.name.bold());
    println!("  {} {:?}", "Capa:".cyan(), config.layer);
    println!("  {} {}x{}", "Tamaño:".cyan(), config.width, config.height);
    println!("  {} {} anclajes activos", "Anclajes:".cyan(), anchor_count);
    println!();

    // Mostrar errores
    if !errors.is_empty() {
        println!("{}", "Errores:".red().bold());
        for error in errors {
            println!("  {} {}", "✗".red(), error.red());
        }
        println!();
        return Ok(());
    }

    // Mostrar advertencias
    if !warnings.is_empty() {
        println!("{}", "Advertencias:".yellow().bold());
        for warning in warnings {
            println!("  {} {}", "⚠".yellow(), warning.yellow());
        }
        println!();
    }

    print_success("Widget válido");
    println!();

    Ok(())
}
