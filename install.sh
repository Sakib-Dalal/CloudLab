#!/bin/bash

# CloudLab CLI Installer
# Supports: Linux (x86_64, ARM64), macOS (Intel, Apple Silicon)

set -e

VERSION="1.0.0"
INSTALL_DIR="/usr/local/bin"
BINARY_NAME="cloudlab"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
YELLOW='\033[0;33m'
NC='\033[0m'

print_banner() {
    echo -e "${CYAN}"
    cat << "EOF"
   _____ _                 _ _           _     
  / ____| |               | | |         | |    
 | |    | | ___  _   _  __| | |     __ _| |__  
 | |    | |/ _ \| | | |/ _` | |    / _` | '_ \ 
 | |____| | (_) | |_| | (_| | |___| (_| | |_) |
  \_____|_|\___/ \__,_|\__,_|______\__,_|_.__/ 
                                               
  CloudLab CLI Installer v1.0.0
  Using UV Package Manager
EOF
    echo -e "${NC}"
}

print_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
print_success() { echo -e "${GREEN}[OK]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }
print_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }

detect_os() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"
    
    case "$OS" in
        Linux*)     OS=linux;;
        Darwin*)    OS=darwin;;
        *)          print_error "Unsupported OS: $OS"; exit 1;;
    esac
    
    case "$ARCH" in
        x86_64)     ARCH=amd64;;
        aarch64)    ARCH=arm64;;
        arm64)      ARCH=arm64;;
        *)          print_error "Unsupported architecture: $ARCH"; exit 1;;
    esac
    
    print_info "Detected: $OS/$ARCH"
}

install_go() {
    if command -v go &> /dev/null; then
        print_success "Go is already installed: $(go version)"
        return 0
    fi

    print_info "Installing Go..."
    GO_VERSION="1.21.5"
    
    case "$OS-$ARCH" in
        linux-amd64)  GO_URL="https://go.dev/dl/go${GO_VERSION}.linux-amd64.tar.gz" ;;
        linux-arm64)  GO_URL="https://go.dev/dl/go${GO_VERSION}.linux-arm64.tar.gz" ;;
        darwin-amd64) GO_URL="https://go.dev/dl/go${GO_VERSION}.darwin-amd64.tar.gz" ;;
        darwin-arm64) GO_URL="https://go.dev/dl/go${GO_VERSION}.darwin-arm64.tar.gz" ;;
    esac
    
    curl -sL "$GO_URL" -o /tmp/go.tar.gz
    sudo rm -rf /usr/local/go
    sudo tar -C /usr/local -xzf /tmp/go.tar.gz
    rm /tmp/go.tar.gz
    
    export PATH=$PATH:/usr/local/go/bin
    
    # Add to shell config
    for rc in ~/.bashrc ~/.zshrc; do
        if [ -f "$rc" ]; then
            grep -q '/usr/local/go/bin' "$rc" || echo 'export PATH=$PATH:/usr/local/go/bin' >> "$rc"
        fi
    done
    
    print_success "Go ${GO_VERSION} installed!"
}

build_cloudlab() {
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    
    if [ ! -f "$SCRIPT_DIR/cloudlab.go" ]; then
        print_error "cloudlab.go not found in $SCRIPT_DIR"
        exit 1
    fi
    
    print_info "Building CloudLab CLI..."
    cd "$SCRIPT_DIR"
    
    # Initialize module if needed
    if [ ! -f "go.mod" ]; then
        /usr/local/go/bin/go mod init cloudlab 2>/dev/null || go mod init cloudlab
    fi
    
    # Build with optimizations
    CGO_ENABLED=0 go build -ldflags="-s -w" -o "$BINARY_NAME" cloudlab.go || \
    CGO_ENABLED=0 /usr/local/go/bin/go build -ldflags="-s -w" -o "$BINARY_NAME" cloudlab.go
    
    if [ ! -f "$BINARY_NAME" ]; then
        print_error "Build failed!"
        exit 1
    fi
    
    # Install
    print_info "Installing to ${INSTALL_DIR}..."
    sudo mv "$BINARY_NAME" "${INSTALL_DIR}/${BINARY_NAME}"
    sudo chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
    
    print_success "CloudLab CLI installed!"
}

setup_completion() {
    print_info "Setting up shell completion..."
    
    # Bash completion
    if [ -d "/etc/bash_completion.d" ]; then
        sudo tee /etc/bash_completion.d/cloudlab > /dev/null << 'EOF'
_cloudlab() {
    local cur prev opts
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    
    opts="init install start stop status config tunnel kernel env email dashboard update uninstall help version"
    
    case "${prev}" in
        install)
            COMPREPLY=( $(compgen -W "all jupyter vscode cloudflare uv" -- ${cur}) )
            return 0;;
        start|stop)
            COMPREPLY=( $(compgen -W "all jupyter lab notebook vscode tunnel dashboard" -- ${cur}) )
            return 0;;
        tunnel)
            COMPREPLY=( $(compgen -W "start stop restart status" -- ${cur}) )
            return 0;;
        kernel)
            COMPREPLY=( $(compgen -W "list add remove" -- ${cur}) )
            return 0;;
        env)
            COMPREPLY=( $(compgen -W "list create remove install" -- ${cur}) )
            return 0;;
        email)
            COMPREPLY=( $(compgen -W "setup test send" -- ${cur}) )
            return 0;;
        config)
            COMPREPLY=( $(compgen -W "set reset" -- ${cur}) )
            return 0;;
    esac
    
    COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
}
complete -F _cloudlab cloudlab
EOF
        print_success "Bash completion installed"
    fi
    
    # Zsh completion
    ZSH_COMP_DIR="${HOME}/.zsh/completion"
    mkdir -p "$ZSH_COMP_DIR"
    
    cat > "$ZSH_COMP_DIR/_cloudlab" << 'EOF'
#compdef cloudlab
_cloudlab() {
    local -a commands
    commands=(
        'init:Initialize CloudLab'
        'install:Install components'
        'start:Start services'
        'stop:Stop services'
        'status:Show status'
        'config:Configuration'
        'tunnel:Tunnel management'
        'kernel:Kernel management'
        'env:Environment management'
        'email:Email notifications'
        'dashboard:Web dashboard'
        'update:Update CloudLab'
        'uninstall:Uninstall'
        'help:Show help'
        'version:Show version'
    )
    _describe 'command' commands
}
_cloudlab
EOF
    
    if [ -f ~/.zshrc ]; then
        grep -q '\.zsh/completion' ~/.zshrc || {
            echo 'fpath=(~/.zsh/completion $fpath)' >> ~/.zshrc
            echo 'autoload -Uz compinit && compinit' >> ~/.zshrc
        }
    fi
}

create_services() {
    # Create systemd services for Linux
    if [ "$OS" = "linux" ] && command -v systemctl &> /dev/null; then
        print_info "Creating systemd services..."
        
        sudo tee /etc/systemd/system/cloudlab.service > /dev/null << EOF
[Unit]
Description=CloudLab Services
After=network.target

[Service]
Type=forking
User=$USER
ExecStart=${INSTALL_DIR}/cloudlab start all
ExecStop=${INSTALL_DIR}/cloudlab stop all
RemainAfterExit=yes

[Install]
WantedBy=multi-user.target
EOF
        
        sudo systemctl daemon-reload
        print_success "Systemd service created. Enable with: sudo systemctl enable cloudlab"
    fi
    
    # Create launchd service for macOS
    if [ "$OS" = "darwin" ]; then
        print_info "Creating launchd service..."
        
        PLIST_DIR="${HOME}/Library/LaunchAgents"
        mkdir -p "$PLIST_DIR"
        
        cat > "${PLIST_DIR}/com.cloudlab.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.cloudlab</string>
    <key>ProgramArguments</key>
    <array>
        <string>${INSTALL_DIR}/cloudlab</string>
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
        print_success "Launchd service created. Enable with: launchctl load ~/Library/LaunchAgents/com.cloudlab.plist"
    fi
}

main() {
    print_banner
    
    if [ "$EUID" -eq 0 ]; then
        print_error "Please do not run as root. Use a regular user."
        exit 1
    fi
    
    detect_os
    install_go
    build_cloudlab
    setup_completion
    create_services
    
    echo ""
    print_success "Installation complete!"
    echo ""
    echo -e "${CYAN}Next Steps:${NC}"
    echo "  1. cloudlab init          # Configure (email, ports, etc.)"
    echo "  2. cloudlab install all   # Install components"
    echo "  3. cloudlab start all     # Start services"
    echo ""
    echo -e "${CYAN}Quick Commands:${NC}"
    echo "  cloudlab status           # Check status"
    echo "  cloudlab tunnel status    # View tunnel URLs"
    echo "  cloudlab help             # Full help"
    echo ""
}

main "$@"
