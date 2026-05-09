# Ejemplos de Widgets Magnetar

Esta carpeta contiene widgets de ejemplo para Magnetar.

## Widgets Disponibles

### 1. Topbar (../topbar.html)

Barra superior completa con:

- Workspaces interactivos
- Ventana activa
- Contador de ventanas
- Reloj y fecha

**Uso:**

```bash
# Copiar al directorio de configuración
cp topbar.html ~/.config/magnetar/

# O ejecutar directamente
magnetar
```

### 2. Sidebar (sidebar.html)

Panel lateral con información detallada:

- Lista de workspaces con contador de ventanas
- Lista de todas las ventanas abiertas
- Indicador de ventana activa
- Información de estado (flotante, fullscreen)

**Uso:**

```bash
cp examples/sidebar.html ~/.config/magnetar/
```

### 3. Notification (notification.html)

Widget de notificaciones que muestra:

- Cambios de workspace
- Cambios de ventana activa
- Auto-oculta después de 3 segundos

**Uso:**

```bash
cp examples/notification.html ~/.config/magnetar/
```

## Personalización

Cada widget puede personalizarse editando:

1. **Configuración** (comentario HTML):

```html
<!-- magnetar: layer=top, height=40, anchor=top|left|right -->
```

2. **Estilos** (CSS):

```css
body {
  background: rgba(30, 30, 46, 0.95);
  color: #cdd6f4;
}
```

3. **Funcionalidad** (JavaScript):

```javascript
magnetar.on("workspace:changed", (data) => {
  console.log("Workspace:", data.id);
});
```

## Crear tu Propio Widget

```bash
# Usar el CLI para generar desde plantilla
magnetar new mi-widget --template topbar

# Editar el archivo generado
nano ~/.config/magnetar/mi-widget.html

# Validar configuración
magnetar validate ~/.config/magnetar/mi-widget.html

# Reiniciar Magnetar para cargar el widget
```

## API Disponible

### Comandos IPC

```javascript
// Workspaces
const workspaces = await magnetar.invoke("hyprland.workspaces");
const active = await magnetar.invoke("hyprland.activeworkspace");

// Ventanas
const window = await magnetar.invoke("hyprland.activewindow");
const clients = await magnetar.invoke("hyprland.clients");

// Ejecutar comandos
await magnetar.invoke("hyprland.exec", "workspace 2");

// Control de ventana
await magnetar.invoke("window.resize", { width: 400 });
await magnetar.invoke("window.hide");
await magnetar.invoke("window.show");
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
```

## Tips

1. **Debugging**: Abre las DevTools con F12 en cualquier widget
2. **Hot Reload**: Edita el HTML y recarga con Ctrl+R
3. **Logging**: Usa `console.log()` para debug
4. **Estilos**: Usa variables CSS para temas consistentes
5. **Performance**: Evita actualizaciones muy frecuentes (< 100ms)

## Temas

Los widgets usan el esquema de colores Catppuccin Mocha por defecto:

```css
:root {
  --bg: rgba(30, 30, 46, 0.95);
  --fg: #cdd6f4;
  --blue: #89b4fa;
  --pink: #f38ba8;
  --purple: #cba6f7;
  --green: #a6e3a1;
  --yellow: #f9e2af;
}
```

Puedes cambiar estos colores para personalizar el tema.
