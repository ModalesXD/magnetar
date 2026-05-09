# Magnetar

Un gestor de widgets para Wayland shell compositor, diseñado específicamente para Hyprland.

## Características

- 🎨 **Widgets HTML/CSS/JS**: Crea interfaces personalizadas con tecnologías web
- 🚀 **Alto rendimiento**: Runtime optimizado con Tokio y GTK4
- 🔧 **CLI completo**: Gestiona widgets y compositor desde la terminal
- 📊 **Integración con Hyprland**: Acceso completo a workspaces, ventanas y eventos
- 🎯 **Sistema modular**: Arquitectura limpia y extensible

## Inicio Rápido

```bash
# 1. Compilar
cargo build --release

# 2. Instalar
sudo cp target/release/magnetar /usr/local/bin/

# 3. El proyecto incluye una topbar de ejemplo
# Simplemente ejecuta:
magnetar

# 4. Para crear widgets personalizados:
magnetar new mi-widget --template topbar
```

## Ejemplos Incluidos

El proyecto incluye varios widgets de ejemplo:

- **topbar.html** - Barra superior completa con workspaces, ventana activa, y reloj
- **examples/sidebar.html** - Panel lateral con información detallada
- **examples/notification.html** - Widget de notificaciones

Para usar los ejemplos:

```bash
# Copiar al directorio de configuración
mkdir -p ~/.config/magnetar
cp topbar.html ~/.config/magnetar/
cp examples/*.html ~/.config/magnetar/

# Listar widgets disponibles
magnetar widget list -v

# Validar un widget
magnetar validate ~/.config/magnetar/topbar.html
```

## Uso

### Iniciar Magnetar

```bash
# Modo GUI (por defecto)
magnetar

# O explícitamente
magnetar run
```

### CLI - Gestión de Widgets

#### Listar widgets disponibles

```bash
# Lista simple
magnetar widget list

# Lista detallada
magnetar widget list -v
```

#### Inspeccionar un widget

```bash
magnetar widget inspect ~/.config/magnetar/topbar.html
```

#### Crear un nuevo widget

```bash
# Crear desde plantilla
magnetar new mi-topbar

# Especificar tipo de plantilla
magnetar new mi-sidebar --template sidebar

# Especificar directorio de salida
magnetar new mi-widget --output ./widgets
```

Plantillas disponibles:

- `topbar` - Barra superior
- `sidebar` - Barra lateral
- `overlay` - Widget flotante
- `notification` - Notificación

#### Validar configuración

```bash
magnetar validate ~/.config/magnetar/topbar.html
```

### CLI - Compositor (Hyprland)

#### Información del compositor

```bash
magnetar compositor info
```

#### Listar workspaces

```bash
# Formato tabla
magnetar compositor workspaces

# Formato JSON
magnetar compositor workspaces --format json
```

#### Listar ventanas/clientes

```bash
# Todas las ventanas
magnetar compositor clients

# Filtrar por workspace
magnetar compositor clients --workspace 1

# Formato JSON
magnetar compositor clients --format json
```

#### Workspace activo

```bash
magnetar compositor active
```

#### Ejecutar comandos

```bash
# Cambiar a workspace
magnetar compositor exec workspace 2

# Mover ventana
magnetar compositor exec movetoworkspace 3

# Cualquier comando de Hyprland
magnetar compositor exec "fullscreen, 1"
```

### CLI - Información del Sistema

```bash
# Información básica
magnetar info

# Información detallada
magnetar info -v
```

## Configuración de Widgets

Los widgets se configuran mediante comentarios HTML:

```html
<!-- magnetar: layer=top, height=40, anchor=top|left|right, exclusive_zone=-1 -->
<!DOCTYPE html>
<html>
  <head>
    <style>
      body {
        background: rgba(30, 30, 46, 0.95);
        color: #cdd6f4;
      }
    </style>
  </head>
  <body>
    <h1>Mi Widget</h1>
    <script>
      // API de Magnetar
      if (window.magnetar) {
        // Invocar comandos IPC
        magnetar.invoke("hyprland.workspaces").then((ws) => {
          console.log("Workspaces:", ws);
        });

        // Escuchar eventos
        magnetar.on("workspace:changed", (data) => {
          console.log("Workspace cambió:", data);
        });
      }
    </script>
  </body>
</html>
```

### Opciones de Configuración

- `layer`: `background`, `bottom`, `top`, `overlay`
- `height`: Altura en píxeles
- `width`: Ancho en píxeles (0 = ancho completo)
- `anchor`: Combinación de `top`, `bottom`, `left`, `right` separados por `|`
- `margin_top`, `margin_bottom`, `margin_left`, `margin_right`: Márgenes en píxeles
- `exclusive_zone`: Zona exclusiva (-1 = automático)

## API JavaScript

### IPC - Invocar comandos

```javascript
// Obtener workspaces
const workspaces = await magnetar.invoke("hyprland.workspaces");

// Workspace activo
const active = await magnetar.invoke("hyprland.activeworkspace");

// Ventana activa
const window = await magnetar.invoke("hyprland.activewindow");

// Todos los clientes
const clients = await magnetar.invoke("hyprland.clients");

// Ejecutar comando
await magnetar.invoke("hyprland.exec", "workspace 2");

// Control de ventana
await magnetar.invoke("window.resize", { width: 400, height: 300 });
await magnetar.invoke("window.hide");
await magnetar.invoke("window.show");

// Broadcast a otros widgets
await magnetar.invoke("broadcast", {
  event: "mi-evento",
  data: { foo: "bar" },
});
```

### Eventos

```javascript
// Cambio de workspace
magnetar.on("workspace:changed", (data) => {
  console.log("Nuevo workspace:", data.id);
});

// Cambio de ventana activa
magnetar.on("activewindow:changed", (data) => {
  console.log("Ventana:", data.title, data.class);
});

// Eventos personalizados (via broadcast)
magnetar.on("mi-evento", (data) => {
  console.log("Evento recibido:", data);
});
```

## Estructura del Proyecto

```
magnetar/
├── src/
│   ├── cli/              # CLI y comandos
│   │   └── commands/     # Implementación de comandos
│   ├── compositor/       # Integración con compositor
│   │   ├── handlers.rs   # Handlers IPC del compositor
│   │   ├── hyprland.rs   # Implementación Hyprland
│   │   └── mod.rs        # Trait Compositor
│   ├── config/           # Configuración y carga de widgets
│   ├── error.rs          # Tipos de error centralizados
│   ├── ipc/              # Sistema IPC WebView ↔ Rust
│   ├── logging/          # Sistema de logging
│   ├── runtime.rs        # Runtime compartido de Tokio
│   ├── window/           # Gestión de ventanas GTK
│   └── main.rs           # Punto de entrada
├── Cargo.toml
└── README.md
```

## Mejoras de Rendimiento

- ✅ Runtime de Tokio optimizado (2 worker threads)
- ✅ Manejo eficiente de eventos del compositor
- ✅ Sistema de logging configurable con filtros
- ✅ Arquitectura modular para reducir acoplamiento
- ✅ Uso de `Arc` y clonación inteligente para compartir datos
- ✅ Compilación optimizada en release (LTO, strip)

## Logging

Configurar nivel de logging mediante variable de entorno:

```bash
# Debug completo
RUST_LOG=magnetar=debug magnetar

# Solo info
RUST_LOG=magnetar=info magnetar

# Solo errores
RUST_LOG=magnetar=error magnetar

# Filtrar módulos específicos
RUST_LOG=magnetar::compositor=debug,magnetar=info magnetar
```

## Desarrollo

```bash
# Compilar
cargo build

# Ejecutar en modo debug
cargo run

# Ejecutar con logging debug
RUST_LOG=magnetar=debug cargo run

# Ejecutar comando CLI
cargo run -- widget list
cargo run -- compositor workspaces
cargo run -- new test-widget

# Tests
cargo test

# Formato
cargo fmt

# Linting
cargo clippy
```

## Licencia

MIT

## Contribuir

Las contribuciones son bienvenidas. Por favor:

1. Fork el proyecto
2. Crea una rama para tu feature (`git checkout -b feature/amazing`)
3. Commit tus cambios (`git commit -am 'Add amazing feature'`)
4. Push a la rama (`git push origin feature/amazing`)
5. Abre un Pull Request

## Roadmap

- [ ] Soporte para más compositores (Sway, River)
- [ ] Hot reload de widgets
- [ ] Sistema de plugins
- [ ] Temas predefinidos
- [ ] Documentación interactiva
- [ ] Gestor de configuración visual
