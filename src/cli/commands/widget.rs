use crate::cli::{print_error, print_info, print_success, WidgetCommands};
use crate::config::{WidgetLoader, WindowConfig};
use colored::Colorize;

pub async fn execute(cmd: WidgetCommands) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        WidgetCommands::List { verbose } => list_widgets(verbose).await,
        WidgetCommands::Reload { name } => reload_widgets(name).await,
        WidgetCommands::Inspect { widget } => inspect_widget(widget).await,
    }
}

async fn list_widgets(verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    let loader = WidgetLoader::new();
    let widgets = loader.load_all()?;

    if widgets.is_empty() {
        print_info("No se encontraron widgets");
        return Ok(());
    }

    println!("\n{}", "Widgets disponibles:".bold().underline());
    println!();

    for widget in &widgets {
        let name = widget.config.name.cyan().bold();
        let layer = format!("{:?}", widget.config.layer).yellow();
        let size = format!("{}x{}", widget.config.width, widget.config.height).green();

        println!(
            "  {} {} {}",
            "•".blue(),
            name,
            format!("[{}]", layer).dimmed()
        );

        if verbose {
            println!("    {} {}", "Tamaño:".dimmed(), size);
            println!(
                "    {} top={} bottom={} left={} right={}",
                "Anclajes:".dimmed(),
                widget.config.anchor_top,
                widget.config.anchor_bottom,
                widget.config.anchor_left,
                widget.config.anchor_right
            );
            println!(
                "    {} {}",
                "Zona exclusiva:".dimmed(),
                widget.config.exclusive_zone
            );
            println!();
        }
    }

    println!(
        "\n{} {} widgets encontrados\n",
        "Total:".bold(),
        widgets.len()
    );

    Ok(())
}

async fn reload_widgets(_name: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    print_error("La recarga de widgets requiere que Magnetar esté ejecutándose");
    print_info("Esta funcionalidad estará disponible en una versión futura con IPC");
    Ok(())
}

async fn inspect_widget(widget_path: String) -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::Path::new(&widget_path);

    if !path.exists() {
        print_error(&format!("Widget no encontrado: {}", widget_path));
        return Ok(());
    }

    let html = std::fs::read_to_string(path)?;
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    let config = WindowConfig::from_html(&html, name);

    println!("\n{}", "Configuración del widget:".bold().underline());
    println!();
    println!("  {} {}", "Nombre:".cyan(), config.name.bold());
    println!("  {} {:?}", "Capa:".cyan(), config.layer);
    println!("  {} {}x{}", "Tamaño:".cyan(), config.width, config.height);
    println!(
        "  {} top={} bottom={} left={} right={}",
        "Anclajes:".cyan(),
        config.anchor_top,
        config.anchor_bottom,
        config.anchor_left,
        config.anchor_right
    );
    println!(
        "  {} top={} bottom={} left={} right={}",
        "Márgenes:".cyan(),
        config.margin_top,
        config.margin_bottom,
        config.margin_left,
        config.margin_right
    );
    println!("  {} {}", "Zona exclusiva:".cyan(), config.exclusive_zone);
    println!();

    print_success("Widget válido");

    Ok(())
}
