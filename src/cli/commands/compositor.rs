use crate::cli::{print_error, print_info, print_success, CompositorCommands};
use crate::compositor::CompositorManager;
use colored::Colorize;

pub async fn execute(cmd: CompositorCommands) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        CompositorCommands::Info => compositor_info().await,
        CompositorCommands::Workspaces { format } => list_workspaces(format).await,
        CompositorCommands::Clients { format, workspace } => list_clients(format, workspace).await,
        CompositorCommands::Active => active_workspace().await,
        CompositorCommands::Exec { command } => exec_command(command).await,
    }
}

async fn compositor_info() -> Result<(), Box<dyn std::error::Error>> {
    let manager = CompositorManager::new();

    match manager.detect().await {
        Ok(_) => {
            print_success("Compositor detectado: Hyprland");

            if let Ok(signature) = std::env::var("HYPRLAND_INSTANCE_SIGNATURE") {
                println!("  {} {}", "Instancia:".cyan(), signature.dimmed());
            }

            Ok(())
        }
        Err(e) => {
            print_error(&format!("No se detectó compositor: {}", e));
            Ok(())
        }
    }
}

async fn list_workspaces(format: String) -> Result<(), Box<dyn std::error::Error>> {
    let manager = CompositorManager::new();
    manager.detect().await?;

    let Some(compositor) = manager.get().await else {
        print_error("No se pudo obtener el compositor");
        return Ok(());
    };

    let workspaces = compositor.workspaces().await?;

    match format.as_str() {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&workspaces)?);
        }
        _ => {
            println!("\n{}", "Workspaces:".bold().underline());
            println!();
            println!(
                "  {} {} {} {}",
                "ID".cyan().bold(),
                "Nombre".cyan().bold(),
                "Monitor".cyan().bold(),
                "Ventanas".cyan().bold()
            );
            println!("  {}", "─".repeat(60).dimmed());

            for ws in workspaces {
                let urgent = if ws.urgent {
                    "⚠".yellow()
                } else {
                    " ".normal()
                };
                println!(
                    "  {} {:2} {:15} {:10} {:3}",
                    urgent,
                    ws.id.to_string().green(),
                    ws.name,
                    ws.monitor.dimmed(),
                    ws.windows
                );
            }
            println!();
        }
    }

    Ok(())
}

async fn list_clients(
    format: String,
    workspace_filter: Option<i32>,
) -> Result<(), Box<dyn std::error::Error>> {
    let manager = CompositorManager::new();
    manager.detect().await?;

    let Some(compositor) = manager.get().await else {
        print_error("No se pudo obtener el compositor");
        return Ok(());
    };

    let mut clients = compositor.clients().await?;

    if let Some(ws) = workspace_filter {
        clients.retain(|c| c.workspace == ws);
    }

    match format.as_str() {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&clients)?);
        }
        _ => {
            println!("\n{}", "Clientes:".bold().underline());
            println!();
            println!(
                "  {} {} {} {}",
                "WS".cyan().bold(),
                "Título".cyan().bold(),
                "Clase".cyan().bold(),
                "Estado".cyan().bold()
            );
            println!("  {}", "─".repeat(80).dimmed());

            for client in &clients {
                let focused = if client.focused {
                    "●".green()
                } else {
                    "○".dimmed()
                };
                let floating = if client.floating { "F" } else { " " };
                let fullscreen = if client.fullscreen { "⛶" } else { " " };

                println!(
                    "  {} {:2} {:30} {:20} {}{}",
                    focused,
                    client.workspace.to_string().yellow(),
                    truncate(&client.title, 30),
                    truncate(&client.class, 20).cyan(),
                    floating,
                    fullscreen
                );
            }
            println!();
            println!("  {} {} clientes", "Total:".bold(), clients.len());
            println!();
        }
    }

    Ok(())
}

async fn active_workspace() -> Result<(), Box<dyn std::error::Error>> {
    let manager = CompositorManager::new();
    manager.detect().await?;

    let Some(compositor) = manager.get().await else {
        print_error("No se pudo obtener el compositor");
        return Ok(());
    };

    let ws = compositor.activeworkspace().await?;

    println!("\n{}", "Workspace activo:".bold().underline());
    println!();
    println!("  {} {}", "ID:".cyan(), ws.id.to_string().green().bold());
    println!("  {} {}", "Nombre:".cyan(), ws.name);
    println!("  {} {}", "Monitor:".cyan(), ws.monitor.dimmed());
    println!("  {} {}", "Ventanas:".cyan(), ws.windows);
    println!();

    Ok(())
}

async fn exec_command(command: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    if command.is_empty() {
        print_error("Debes especificar un comando");
        return Ok(());
    }

    let manager = CompositorManager::new();
    manager.detect().await?;

    let Some(compositor) = manager.get().await else {
        print_error("No se pudo obtener el compositor");
        return Ok(());
    };

    let cmd = command.join(" ");
    print_info(&format!("Ejecutando: {}", cmd.yellow()));

    match compositor.dispatch(&cmd).await {
        Ok(_) => {
            print_success("Comando ejecutado correctamente");
        }
        Err(e) => {
            print_error(&format!("Error al ejecutar comando: {}", e));
        }
    }

    Ok(())
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len - 1])
    }
}
