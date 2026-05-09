#!/bin/bash

set -e

echo "🚀 Instalando Magnetar..."
echo ""

# Colores
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Verificar que estamos en Hyprland
if [ -z "$HYPRLAND_INSTANCE_SIGNATURE" ]; then
    echo -e "${YELLOW}⚠ Advertencia: No se detectó Hyprland${NC}"
    echo "Magnetar está diseñado para Hyprland, pero puedes continuar de todos modos."
    read -p "¿Continuar? (s/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Ss]$ ]]; then
        exit 1
    fi
fi

# Compilar
echo -e "${BLUE}📦 Compilando Magnetar...${NC}"
cargo build --release

# Instalar binario
echo -e "${BLUE}📥 Instalando binario...${NC}"
if [ "$EUID" -eq 0 ]; then
    cp target/release/magnetar /usr/local/bin/
else
    sudo cp target/release/magnetar /usr/local/bin/
fi

# Crear directorio de configuración
echo -e "${BLUE}📁 Creando directorio de configuración...${NC}"
mkdir -p ~/.config/magnetar

# Copiar ejemplos
echo -e "${BLUE}📋 Copiando widgets de ejemplo...${NC}"
cp topbar.html ~/.config/magnetar/
if [ -d "examples" ]; then
    cp examples/*.html ~/.config/magnetar/ 2>/dev/null || true
fi

echo ""
echo -e "${GREEN}✓ Instalación completada!${NC}"
echo ""
echo "Para iniciar Magnetar:"
echo "  $ magnetar"
echo ""
echo "Para ver comandos disponibles:"
echo "  $ magnetar --help"
echo ""
echo "Para listar widgets:"
echo "  $ magnetar widget list"
echo ""
echo "Para crear un nuevo widget:"
echo "  $ magnetar new mi-widget"
echo ""
echo "Widgets instalados en: ~/.config/magnetar/"
echo ""
