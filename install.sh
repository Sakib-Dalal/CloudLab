#!/bin/bash
# CloudLab CLI Installer
# Author: Sakib Dalal
# GitHub: https://github.com/Sakib-Dalal
# Supports: Linux (x86_64, ARM64), macOS (Intel, Apple Silicon), Git Bash on Windows

set -e

# Configuration
VERSION="1.2.0"
AUTHOR="Sakib Dalal"
GITHUB="https://github.com/Sakib-Dalal"
GO_VERSION="1.21.6"

# Detect environment
detect_environment() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"
    IS_GITBASH=false
    IS_WSL=false
    
    case "$OS" in
        Linux*)
            if grep -qi microsoft /proc/version 2>/dev/null; then
                IS_WSL=true
                OS="wsl"
            else
                OS="linux"
            fi
            ;;
        Darwin*)
            OS="darwin"
            ;;
        MINGW*|MSYS*|CYGWIN*)
            IS_GITBASH=true
            OS="windows"
            ;;
        *)
            echo "Unsupported OS: $OS"
            exit 1
            ;;
    esac
    
    case "$ARCH" in
        x86_64|amd64)
            ARCH="amd64"
            ;;
        aarch64|arm64)
            ARCH="arm64"
            ;;
        *)
            echo "Unsupported architecture: $ARCH"
            exit 1
            ;;
    esac
    
    # Set paths based on environment
    if [ "$IS_GITBASH" = true ]; then
        HOME_DIR="$USERPROFILE"
        INSTALL_DIR="$LOCALAPPDATA/CloudLab/bin"
        CLOUDLAB_DIR="$USERPROFILE/.cloudlab"
        BINARY_NAME="cloudlab.exe"
    else
        HOME_DIR="$HOME"
        INSTALL_DIR="/usr/local/bin"
        CLOUDLAB_DIR="$HOME/.cloudlab"
        BINARY_NAME="cloudlab"
    fi
}

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
WHITE='\033[0;37m'
BOLD='\033[1m'
DIM='\033[2m'
NC='\033[0m'

# Print functions
print_banner() {
    echo -e "${CYAN}${BOLD}"
    cat << "EOF"
   _____ _                 _ _           _     
  / ____| |               | | |         | |    
 | |    | | ___  _   _  __| | |     __ _| |__  
 | |    | |/ _ \| | | |/ _` | |    / _` | '_ \ 
 | |____| | (_) | |_| | (_| | |___| (_| | |_) |
  \_____|_|\___/ \__,_|\__,_|______\__,_|_.__/ 
EOF
    echo -e "${NC}"
    echo -e "${WHITE}${BOLD}  ☁️  CloudLab CLI Installer ${GREEN}v${VERSION}${NC}"
    echo -e "${YELLOW}  👤 Author: ${BOLD}${AUTHOR}${NC}"
    echo -e "${BLUE}  🔗 GitHub: ${GITHUB}${NC}"
    echo ""
}

print_step() {
    echo -e "  ${BLUE}▶${NC} $1"
}

print_success() {
    echo -e "  ${GREEN}✓${NC} $1"
}

print_error() {
    echo -e "  ${RED}✗${NC} $1"
}

print_warning() {
    echo -e "  ${YELLOW}⚠${NC} $1"
}

print_info() {
    echo -e "  ${CYAN}💡${NC} $1"
}

print_header() {
    echo ""
    echo -e "${BOLD}${WHITE}$1${NC}"
    echo -e "${DIM}$(printf '─%.0s' {1..50})${NC}"
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Install Go
install_go() {
    print_header "📦 INSTALLING GO"
    
    if command_exists go; then
        GO_VER=$(go version | awk '{print $3}')
        print_success "Go already installed: $GO_VER"
        return 0
    fi
    
    print_step "Downloading Go $GO_VERSION..."
    
    case "$OS-$ARCH" in
        linux-amd64|wsl-amd64)
            GO_URL="https://go.dev/dl/go${GO_VERSION}.linux-amd64.tar.gz"
            ;;
        linux-arm64|wsl-arm64)
            GO_URL="https://go.dev/dl/go${GO_VERSION}.linux-arm64.tar.gz"
            ;;
        darwin-amd64)
            GO_URL="https://go.dev/dl/go${GO_VERSION}.darwin-amd64.tar.gz"
            ;;
        darwin-arm64)
            GO_URL="https://go.dev/dl/go${GO_VERSION}.darwin-arm64.tar.gz"
            ;;
        windows-amd64)
            GO_URL="https://go.dev/dl/go${GO_VERSION}.windows-amd64.zip"
            ;;
        *)
            print_error "Unsupported platform: $OS-$ARCH"
            return 1
            ;;
    esac
    
    if [ "$IS_GITBASH" = true ]; then
        # Windows Git Bash - download and extract
        print_step "Downloading Go for Windows..."
        curl -sL "$GO_URL" -o /tmp/go.zip
        
        print_step "Extracting Go..."
        mkdir -p "$LOCALAPPDATA/Go"
        unzip -q /tmp/go.zip -d "$LOCALAPPDATA"
        rm /tmp/go.zip
        
        export PATH="$LOCALAPPDATA/Go/go/bin:$PATH"
        print_success "Go $GO_VERSION installed to $LOCALAPPDATA/Go"
        print_warning "Add to PATH: export PATH=\"\$LOCALAPPDATA/Go/go/bin:\$PATH\""
    else
        # Unix systems
        curl -sL "$GO_URL" -o /tmp/go.tar.gz
        
        print_step "Installing Go..."
        sudo rm -rf /usr/local/go
        sudo tar -C /usr/local -xzf /tmp/go.tar.gz
        rm /tmp/go.tar.gz
        
        export PATH=$PATH:/usr/local/go/bin
        
        # Add to shell config
        for rc in ~/.bashrc ~/.zshrc ~/.profile; do
            if [ -f "$rc" ]; then
                grep -q '/usr/local/go/bin' "$rc" || echo 'export PATH=$PATH:/usr/local/go/bin' >> "$rc"
            fi
        done
        
        print_success "Go $GO_VERSION installed"
    fi
    
    return 0
}

# Install UV
install_uv() {
    print_header "📦 INSTALLING UV PACKAGE MANAGER"
    
    if command_exists uv; then
        print_success "UV already installed"
        return 0
    fi
    
    print_step "Installing UV..."
    
    if [ "$IS_GITBASH" = true ]; then
        powershell.exe -Command "irm https://astral.sh/uv/install.ps1 | iex"
    else
        curl -LsSf https://astral.sh/uv/install.sh | sh
    fi
    
    # Update PATH
    export PATH="$HOME/.cargo/bin:$HOME/.local/bin:$PATH"
    
    print_success "UV installed"
    return 0
}

# Install cloudflared
install_cloudflared() {
    print_header "📦 INSTALLING CLOUDFLARED"
    
    if command_exists cloudflared; then
        print_success "cloudflared already installed"
        return 0
    fi
    
    print_step "Installing cloudflared..."
    
    case "$OS" in
        darwin)
            if command_exists brew; then
                brew install cloudflared
            else
                curl -L "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-darwin-${ARCH}.tgz" | tar xz
                sudo mv cloudflared /usr/local/bin/
            fi
            ;;
        linux|wsl)
            if [ "$ARCH" = "amd64" ]; then
                CF_URL="https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64"
            else
                CF_URL="https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-arm64"
            fi
            curl -L "$CF_URL" -o /tmp/cloudflared
            chmod +x /tmp/cloudflared
            sudo mv /tmp/cloudflared /usr/local/bin/cloudflared
            ;;
        windows)
            curl -L "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-windows-amd64.exe" -o "$INSTALL_DIR/cloudflared.exe"
            ;;
    esac
    
    print_success "cloudflared installed"
    return 0
}

# Install code-server
install_code_server() {
    print_header "📦 INSTALLING VS CODE SERVER"
    
    if command_exists code-server; then
        print_success "code-server already installed"
        return 0
    fi
    
    print_step "Installing code-server..."
    
    if [ "$IS_GITBASH" = true ]; then
        if command_exists npm; then
            npm install -g code-server
        else
            print_warning "npm not found. Please install code-server manually."
            return 1
        fi
    else
        curl -fsSL https://code-server.dev/install.sh | sh
    fi
    
    print_success "code-server installed"
    return 0
}

# Install ttyd
install_ttyd() {
    print_header "📦 INSTALLING WEB TERMINAL (TTYD)"
    
    if command_exists ttyd; then
        print_success "ttyd already installed"
        return 0
    fi
    
    print_step "Installing ttyd..."
    
    case "$OS" in
        darwin)
            if command_exists brew; then
                brew install ttyd
            else
                print_warning "Please install Homebrew first, then run: brew install ttyd"
                return 1
            fi
            ;;
        linux|wsl)
            # Try apt
            if command_exists apt-get; then
                sudo apt-get update && sudo apt-get install -y ttyd 2>/dev/null || {
                    # Download binary
                    if [ "$ARCH" = "amd64" ]; then
                        TTYD_URL="https://github.com/tsl0922/ttyd/releases/latest/download/ttyd.x86_64"
                    else
                        TTYD_URL="https://github.com/tsl0922/ttyd/releases/latest/download/ttyd.aarch64"
                    fi
                    curl -L "$TTYD_URL" -o /tmp/ttyd
                    chmod +x /tmp/ttyd
                    sudo mv /tmp/ttyd /usr/local/bin/ttyd
                }
            else
                if [ "$ARCH" = "amd64" ]; then
                    TTYD_URL="https://github.com/tsl0922/ttyd/releases/latest/download/ttyd.x86_64"
                else
                    TTYD_URL="https://github.com/tsl0922/ttyd/releases/latest/download/ttyd.aarch64"
                fi
                curl -L "$TTYD_URL" -o /tmp/ttyd
                chmod +x /tmp/ttyd
                sudo mv /tmp/ttyd /usr/local/bin/ttyd
            fi
            ;;
        windows)
            curl -L "https://github.com/tsl0922/ttyd/releases/latest/download/ttyd.win32.exe" -o "$INSTALL_DIR/ttyd.exe"
            ;;
    esac
    
    print_success "ttyd installed"
    return 0
}

# Build CloudLab
build_cloudlab() {
    print_header "🔨 BUILDING CLOUDLAB CLI"
    
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    
    if [ ! -f "$SCRIPT_DIR/cloudlab.go" ]; then
        print_error "cloudlab.go not found in $SCRIPT_DIR"
        return 1
    fi
    
    cd "$SCRIPT_DIR"
    
    print_step "Initializing Go module..."
    
    # Initialize module
    if [ ! -f "go.mod" ]; then
        go mod init cloudlab 2>/dev/null || true
    fi
    
    # Get dependencies
    go get golang.org/x/text/cases 2>/dev/null || true
    go get golang.org/x/text/language 2>/dev/null || true
    go mod tidy 2>/dev/null || true
    
    print_step "Compiling with optimizations..."
    
    mkdir -p build
    
    export CGO_ENABLED=0
    go build -ldflags="-s -w" -o "build/$BINARY_NAME" cloudlab.go
    
    if [ ! -f "build/$BINARY_NAME" ]; then
        print_error "Build failed!"
        return 1
    fi
    
    SIZE=$(ls -lh "build/$BINARY_NAME" | awk '{print $5}')
    print_success "Built: build/$BINARY_NAME ($SIZE)"
    
    return 0
}

# Install CloudLab binary
install_cloudlab() {
    print_header "📥 INSTALLING CLOUDLAB"
    
    if [ "$IS_GITBASH" = true ]; then
        mkdir -p "$INSTALL_DIR"
        cp "build/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
        print_success "Installed to $INSTALL_DIR/$BINARY_NAME"
        print_warning "Add to PATH: export PATH=\"\$LOCALAPPDATA/CloudLab/bin:\$PATH\""
    else
        print_step "Installing to $INSTALL_DIR..."
        sudo cp "build/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
        sudo chmod +x "$INSTALL_DIR/$BINARY_NAME"
        print_success "Installed to $INSTALL_DIR/$BINARY_NAME"
    fi
    
    return 0
}

# Copy dashboard files
copy_dashboard_files() {
    print_header "📁 COPYING DASHBOARD FILES"
    
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    
    mkdir -p "$CLOUDLAB_DIR"
    mkdir -p "$CLOUDLAB_DIR/logs"
    mkdir -p "$CLOUDLAB_DIR/pids"
    mkdir -p "$CLOUDLAB_DIR/envs"
    
    if [ -f "$SCRIPT_DIR/index.html" ]; then
        cp "$SCRIPT_DIR/index.html" "$CLOUDLAB_DIR/dashboard.html"
        print_success "Copied dashboard.html"
    fi
    
    if [ -f "$SCRIPT_DIR/server.py" ]; then
        cp "$SCRIPT_DIR/server.py" "$CLOUDLAB_DIR/server.py"
        chmod +x "$CLOUDLAB_DIR/server.py"
        print_success "Copied server.py"
    fi
    
    return 0
}

# Create shell completion
setup_completion() {
    print_header "🔧 SETTING UP SHELL COMPLETION"
    
    if [ "$IS_GITBASH" = true ]; then
        print_info "Shell completion not supported in Git Bash"
        return 0
    fi
    
    # Bash completion
    if [ -d "/etc/bash_completion.d" ] && [ -w "/etc/bash_completion.d" ]; then
        sudo tee /etc/bash_completion.d/cloudlab > /dev/null << 'EOF'
_cloudlab() {
    local cur prev opts
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    
    opts="init install start stop restart status logs config tunnel kernel env email ssh dashboard update uninstall help version"
    
    case "${prev}" in
        install)
            COMPREPLY=( $(compgen -W "all jupyter vscode ssh cloudflare uv dashboard" -- ${cur}) )
            return 0;;
        start|stop|restart)
            COMPREPLY=( $(compgen -W "all jupyter lab notebook vscode ssh dashboard tunnel" -- ${cur}) )
            return 0;;
        tunnel|ssh|dashboard)
            COMPREPLY=( $(compgen -W "start stop restart status config" -- ${cur}) )
            return 0;;
        kernel)
            COMPREPLY=( $(compgen -W "list add remove" -- ${cur}) )
            return 0;;
        env)
            COMPREPLY=( $(compgen -W "list create remove install activate" -- ${cur}) )
            return 0;;
        email)
            COMPREPLY=( $(compgen -W "setup test send" -- ${cur}) )
            return 0;;
        config)
            COMPREPLY=( $(compgen -W "set reset" -- ${cur}) )
            return 0;;
        logs)
            COMPREPLY=( $(compgen -W "jupyter vscode ssh dashboard tunnel_jupyter tunnel_vscode tunnel_ssh tunnel_dashboard" -- ${cur}) )
            return 0;;
    esac
    
    COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
}
complete -F _cloudlab cloudlab
EOF
        print_success "Bash completion installed"
    fi
    
    # Zsh completion
    ZSH_COMP_DIR="$HOME/.zsh/completion"
    mkdir -p "$ZSH_COMP_DIR"
    
    cat > "$ZSH_COMP_DIR/_cloudlab" << 'EOF'
#compdef cloudlab

_cloudlab() {
    local -a commands
    commands=(
        'init:Initialize CloudLab configuration'
        'install:Install components'
        'start:Start services'
        'stop:Stop services'
        'restart:Restart services'
        'status:Show status of all services'
        'logs:Show service logs'
        'config:Configuration management'
        'tunnel:Tunnel management'
        'kernel:Jupyter kernel management'
        'env:Python environment management'
        'email:Email notifications'
        'ssh:SSH terminal management'
        'dashboard:Web dashboard management'
        'update:Update components'
        'uninstall:Uninstall CloudLab'
        'help:Show help'
        'version:Show version'
    )
    _describe 'command' commands
}

_cloudlab
EOF
    
    # Add to zshrc if exists
    if [ -f "$HOME/.zshrc" ]; then
        grep -q '\.zsh/completion' "$HOME/.zshrc" || {
            echo 'fpath=(~/.zsh/completion $fpath)' >> "$HOME/.zshrc"
            echo 'autoload -Uz compinit && compinit' >> "$HOME/.zshrc"
        }
    fi
    
    print_success "Zsh completion installed"
    return 0
}

# Create systemd/launchd service
create_service() {
    print_header "⚙️  CREATING SYSTEM SERVICE"
    
    case "$OS" in
        linux|wsl)
            if command_exists systemctl; then
                sudo tee /etc/systemd/system/cloudlab.service > /dev/null << EOF
[Unit]
Description=CloudLab Services
After=network.target

[Service]
Type=forking
User=$USER
ExecStart=/usr/local/bin/cloudlab start all
ExecStop=/usr/local/bin/cloudlab stop all
RemainAfterExit=yes
Restart=on-failure

[Install]
WantedBy=multi-user.target
EOF
                sudo systemctl daemon-reload
                print_success "Systemd service created"
                print_info "Enable with: sudo systemctl enable cloudlab"
            fi
            ;;
        darwin)
            PLIST_DIR="$HOME/Library/LaunchAgents"
            mkdir -p "$PLIST_DIR"
            
            cat > "$PLIST_DIR/com.cloudlab.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.cloudlab</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/cloudlab</string>
        <string>start</string>
        <string>all</string>
    </array>
    <key>RunAtLoad</key>
    <false/>
    <key>KeepAlive</key>
    <false/>
</dict>
</plist>
EOF
            print_success "LaunchAgent created"
            print_info "Enable with: launchctl load ~/Library/LaunchAgents/com.cloudlab.plist"
            ;;
    esac
    
    return 0
}

# Show post-install message
show_post_install() {
    echo ""
    echo -e "${DIM}══════════════════════════════════════════════════${NC}"
    echo ""
    echo -e "${GREEN}${BOLD}✅ INSTALLATION COMPLETE!${NC}"
    echo ""
    echo -e "${BOLD}Quick Start:${NC}"
    echo -e "  ${MAGENTA}\$${NC} cloudlab init         ${DIM}# Configure settings${NC}"
    echo -e "  ${MAGENTA}\$${NC} cloudlab install all  ${DIM}# Install Jupyter, VS Code, etc.${NC}"
    echo -e "  ${MAGENTA}\$${NC} cloudlab start all    ${DIM}# Start all services${NC}"
    echo -e "  ${MAGENTA}\$${NC} cloudlab status       ${DIM}# Check status${NC}"
    echo ""
    echo -e "${BOLD}Services:${NC}"
    echo -e "  🐍 Jupyter Lab/Notebook  ${DIM}(port 8888)${NC}"
    echo -e "  💻 VS Code Server        ${DIM}(port 8080)${NC}"
    echo -e "  🔒 SSH Terminal          ${DIM}(port 7681)${NC}"
    echo -e "  📊 Web Dashboard         ${DIM}(port 3000)${NC}"
    echo ""
    echo -e "${BOLD}Tunnel URLs:${NC}"
    echo -e "  ${MAGENTA}\$${NC} cloudlab tunnel start ${DIM}# Get public URLs${NC}"
    echo -e "  ${MAGENTA}\$${NC} cloudlab email send   ${DIM}# Email all URLs${NC}"
    echo ""
    echo -e "${BOLD}Help:${NC}"
    echo -e "  ${MAGENTA}\$${NC} cloudlab help"
    echo ""
}

# Main installation
main() {
    print_banner
    
    # Detect environment
    detect_environment
    
    print_info "Detected: $OS / $ARCH"
    [ "$IS_GITBASH" = true ] && print_info "Running in Git Bash"
    [ "$IS_WSL" = true ] && print_info "Running in WSL"
    echo ""
    
    # Check if root (not needed on Git Bash)
    if [ "$IS_GITBASH" = false ] && [ "$EUID" -eq 0 ]; then
        print_error "Please do not run as root. Use a regular user."
        exit 1
    fi
    
    # Install components
    install_go || print_warning "Go installation failed"
    
    # Refresh PATH
    export PATH="$PATH:/usr/local/go/bin:$HOME/.cargo/bin:$HOME/.local/bin"
    
    # Check Go is available
    if ! command_exists go; then
        print_error "Go not found in PATH. Please restart your terminal and try again."
        exit 1
    fi
    
    # Build CloudLab
    build_cloudlab || {
        print_error "Build failed!"
        exit 1
    }
    
    # Install binary
    install_cloudlab
    
    # Copy dashboard files
    copy_dashboard_files
    
    # Install dependencies (optional)
    echo ""
    echo -e "${BOLD}Install additional components?${NC} [Y/n]: \c"
    read -r answer
    if [ "$answer" != "n" ] && [ "$answer" != "N" ]; then
        install_uv
        install_cloudflared
        install_code_server
        install_ttyd
    fi
    
    # Setup completion
    setup_completion
    
    # Create service (optional for Unix)
    if [ "$IS_GITBASH" = false ]; then
        create_service
    fi
    
    # Show post-install message
    show_post_install
}

# Run main
main "$@"