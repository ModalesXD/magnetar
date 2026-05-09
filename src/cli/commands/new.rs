use crate::cli::{print_error, print_success};
use colored::Colorize;
use std::path::PathBuf;

pub async fn execute(
    name: String,
    template: String,
    output: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let template_content = match template.as_str() {
        "topbar" => TOPBAR_TEMPLATE,
        "sidebar" => SIDEBAR_TEMPLATE,
        "overlay" => OVERLAY_TEMPLATE,
        "notification" => NOTIFICATION_TEMPLATE,
        _ => {
            print_error(&format!("Plantilla desconocida: {}", template));
            println!("\nPlantillas disponibles:");
            println!("  {} topbar", "•".blue());
            println!("  {} sidebar", "•".blue());
            println!("  {} overlay", "•".blue());
            println!("  {} notification", "•".blue());
            return Ok(());
        }
    };

    let output_dir = output.unwrap_or_else(|| {
        std::env::var("HOME")
            .map(|home| PathBuf::from(format!("{}/.config/magnetar", home)))
            .unwrap_or_else(|_| PathBuf::from("."))
    });

    std::fs::create_dir_all(&output_dir)?;

    let filename = format!("{}.html", name);
    let filepath = output_dir.join(&filename);

    if filepath.exists() {
        print_error(&format!("El archivo ya existe: {}", filepath.display()));
        return Ok(());
    }

    let content = template_content.replace("{{NAME}}", &name);
    std::fs::write(&filepath, content)?;

    print_success(&format!(
        "Widget creado: {}",
        filepath.display().to_string().cyan()
    ));
    println!("\nPara usarlo:");
    println!("  {} magnetar", "1.".yellow(),);
    println!("  {} El widget se cargará automáticamente", "2.".yellow());

    Ok(())
}

const TOPBAR_TEMPLATE: &str = r#"<!-- magnetar: layer=top, height=40, anchor=top|left|right, exclusive_zone=-1 -->
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }

        body {
            background: rgba(30, 30, 46, 0.95);
            color: #cdd6f4;
            font-family: 'JetBrains Mono', monospace;
            display: flex;
            align-items: center;
            justify-content: space-between;
            padding: 0 20px;
            height: 40px;
        }

        .left, .center, .right {
            display: flex;
            align-items: center;
            gap: 15px;
        }

        .workspace {
            padding: 5px 12px;
            background: rgba(137, 180, 250, 0.2);
            border-radius: 5px;
            cursor: pointer;
            transition: all 0.2s;
        }

        .workspace:hover {
            background: rgba(137, 180, 250, 0.4);
        }

        .workspace.active {
            background: #89b4fa;
            color: #1e1e2e;
        }

        .time {
            font-weight: bold;
            color: #89b4fa;
        }
    </style>
</head>
<body>
    <div class="left">
        <div class="workspace active">1</div>
        <div class="workspace">2</div>
        <div class="workspace">3</div>
    </div>
    
    <div class="center">
        <span>{{NAME}}</span>
    </div>
    
    <div class="right">
        <span class="time" id="time">00:00</span>
    </div>

    <script>
        // Actualizar reloj
        function updateTime() {
            const now = new Date();
            const hours = String(now.getHours()).padStart(2, '0');
            const minutes = String(now.getMinutes()).padStart(2, '0');
            document.getElementById('time').textContent = `${hours}:${minutes}`;
        }
        
        updateTime();
        setInterval(updateTime, 1000);

        // Escuchar eventos del compositor
        if (window.magnetar) {
            magnetar.on('workspace:changed', (data) => {
                console.log('Workspace changed:', data);
            });
        }
    </script>
</body>
</html>
"#;

const SIDEBAR_TEMPLATE: &str = r#"<!-- magnetar: layer=top, width=300, anchor=top|bottom|left, exclusive_zone=-1 -->
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }

        body {
            background: rgba(30, 30, 46, 0.95);
            color: #cdd6f4;
            font-family: 'JetBrains Mono', monospace;
            padding: 20px;
            height: 100vh;
            overflow-y: auto;
        }

        h1 {
            color: #89b4fa;
            margin-bottom: 20px;
            font-size: 24px;
        }

        .section {
            margin-bottom: 30px;
        }

        .section h2 {
            color: #f38ba8;
            font-size: 16px;
            margin-bottom: 10px;
        }
    </style>
</head>
<body>
    <h1>{{NAME}}</h1>
    
    <div class="section">
        <h2>Sección 1</h2>
        <p>Contenido del sidebar</p>
    </div>
</body>
</html>
"#;

const OVERLAY_TEMPLATE: &str = r#"<!-- magnetar: layer=overlay, height=200, width=400, anchor=top|right, margin_top=50, margin_right=50 -->
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }

        body {
            background: rgba(30, 30, 46, 0.98);
            color: #cdd6f4;
            font-family: 'JetBrains Mono', monospace;
            padding: 20px;
            border-radius: 10px;
            border: 2px solid #89b4fa;
        }

        h1 {
            color: #89b4fa;
            margin-bottom: 15px;
        }
    </style>
</head>
<body>
    <h1>{{NAME}}</h1>
    <p>Overlay widget</p>
</body>
</html>
"#;

const NOTIFICATION_TEMPLATE: &str = r#"<!-- magnetar: layer=overlay, height=100, width=350, anchor=top|right, margin_top=10, margin_right=10 -->
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }

        body {
            background: rgba(30, 30, 46, 0.98);
            color: #cdd6f4;
            font-family: 'JetBrains Mono', monospace;
            padding: 15px;
            border-radius: 8px;
            border-left: 4px solid #89b4fa;
        }

        .title {
            color: #89b4fa;
            font-weight: bold;
            margin-bottom: 5px;
        }

        .message {
            font-size: 14px;
            color: #bac2de;
        }
    </style>
</head>
<body>
    <div class="title">{{NAME}}</div>
    <div class="message">Notificación de ejemplo</div>
</body>
</html>
"#;
