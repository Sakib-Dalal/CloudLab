# ☁️ CloudLab CLI v1.2.0

**Self-hosted web development environment with Jupyter Lab/Notebook, VS Code Server, SSH Terminal, and Web Dashboard - all with Cloudflare Tunnels and Email Notifications.**

```
   _____ _                 _ _           _     
  / ____| |               | | |         | |    
 | |    | | ___  _   _  __| | |     __ _| |__  
 | |    | |/ _ \| | | |/ _` | |    / _` | '_ \ 
 | |____| | (_) | |_| | (_| | |___| (_| | |_) |
  \_____|_|\___/ \__,_|\__,_|______\__,_|_.__/ 
```

[![Author](https://img.shields.io/badge/Author-Sakib%20Dalal-7c3aed?style=for-the-badge)](https://github.com/Sakib-Dalal)
[![Go](https://img.shields.io/badge/Go-1.21+-00ADD8?style=for-the-badge&logo=go)](https://go.dev)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux%20%7C%20Docker-blue?style=for-the-badge)](https://github.com/Sakib-Dalal/CloudLab)
[![License](https://img.shields.io/badge/License-MIT-green?style=for-the-badge)](LICENSE)

---

## 📋 Table of Contents

- [Features](#-features)
- [Services & Ports](#-services--ports)
- [Installation](#-installation)
  - [Windows Installation](#-windows-installation)
  - [macOS Installation](#-macos-installation)
  - [Linux Installation](#-linux-installation)
  - [Docker Installation](#-docker-installation)
- [Quick Start](#-quick-start)
- [Commands Reference](#-commands-reference)
- [Configuration](#-configuration)
- [Email Setup](#-email-setup)
- [Troubleshooting](#-troubleshooting)
- [Author](#-author)

---

## ✨ Features

| Feature | Description |
|---------|-------------|
| 🐍 **Jupyter Lab/Notebook** | Full Python notebook environment |
| 💻 **VS Code Server** | Browser-based code editor |
| 🔒 **SSH Terminal** | Web-based terminal access (ttyd) |
| 📊 **Web Dashboard** | Manage everything from browser |
| 🌐 **Cloudflare Tunnels** | Free public URLs (no account needed!) |
| 📧 **Email Notifications** | Receive all 4 URLs via email |
| 🐍 **Kernel Management** | Add/remove Jupyter kernels |
| 📦 **Environment Management** | Create Python environments with UV |
| ⚡ **MPS/CUDA Support** | GPU acceleration for Apple/NVIDIA |
| 🔋 **Low Power Mode** | Energy efficient operation |

---

## 🌐 Services & Ports

| Service | Default Port | Description |
|---------|--------------|-------------|
| 🐍 Jupyter | 8888 | Lab or Notebook |
| 💻 VS Code | 8080 | Browser-based IDE |
| 🔒 SSH Terminal | 7681 | Web terminal (ttyd) |
| 📊 Dashboard | 3000 | Management UI |

All 4 services get their own Cloudflare tunnel URL and are sent via email!

---

## 📦 Installation

### 🪟 Windows Installation

#### Prerequisites: Install Go

**Option 1: Using Winget (Recommended)**
```powershell
# Open PowerShell as Administrator
winget install --id GoLang.Go -e
```

**Option 2: Using Chocolatey**
```powershell
# Install Chocolatey first (if not installed)
Set-ExecutionPolicy Bypass -Scope Process -Force
[System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072
iex ((New-Object System.Net.WebClient).DownloadString('https://chocolatey.org/install.ps1'))

# Install Go
choco install golang -y
```

**Option 3: Manual Installation**
1. Download Go from https://go.dev/dl/
2. Download `go1.21.x.windows-amd64.msi`
3. Run the installer
4. Restart PowerShell/Terminal

**Verify Go Installation:**
```powershell
go version
# Should show: go version go1.21.x windows/amd64
```

#### Install CloudLab

**Method 1: PowerShell Installer (Recommended)**
```powershell
# Open PowerShell as Administrator
# Clone repository
git clone https://github.com/Sakib-Dalal/CloudLab.git
cd CloudLab

# Run installer
.\install.ps1

# Follow the prompts
```

**Method 2: Git Bash**
```bash
# Open Git Bash
git clone https://github.com/Sakib-Dalal/CloudLab.git
cd CloudLab

# Make executable and run
chmod +x install.sh
./install.sh
```

**Method 3: Manual Build**
```powershell
# Clone repository
git clone https://github.com/Sakib-Dalal/CloudLab.git
cd CloudLab

# Initialize Go module
go mod init cloudlab
go get golang.org/x/text/cases
go get golang.org/x/text/language
go mod tidy

# Build
$env:CGO_ENABLED=0
go build -ldflags="-s -w" -o cloudlab.exe cloudlab.go

# Create install directory
New-Item -ItemType Directory -Force -Path "$env:LOCALAPPDATA\CloudLab\bin"

# Copy binary
Copy-Item cloudlab.exe "$env:LOCALAPPDATA\CloudLab\bin\"

# Add to PATH (User)
$path = [Environment]::GetEnvironmentVariable("Path", "User")
if ($path -notlike "*CloudLab*") {
    [Environment]::SetEnvironmentVariable("Path", "$path;$env:LOCALAPPDATA\CloudLab\bin", "User")
}

# Restart PowerShell and verify
cloudlab version
```

**Method 4: Double-Click Installer**
1. Download/clone the repository
2. Double-click `install.bat`
3. Follow the prompts

---

### 🍎 macOS Installation

#### Prerequisites: Install Go

**Option 1: Using Homebrew (Recommended)**
```bash
# Install Homebrew (if not installed)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install Go
brew install go
```

**Option 2: Using MacPorts**
```bash
sudo port install go
```

**Option 3: Manual Installation**
```bash
# For Intel Mac
curl -LO https://go.dev/dl/go1.21.6.darwin-amd64.tar.gz
sudo tar -C /usr/local -xzf go1.21.6.darwin-amd64.tar.gz

# For Apple Silicon (M1/M2/M3)
curl -LO https://go.dev/dl/go1.21.6.darwin-arm64.tar.gz
sudo tar -C /usr/local -xzf go1.21.6.darwin-arm64.tar.gz

# Add to PATH
echo 'export PATH=$PATH:/usr/local/go/bin' >> ~/.zshrc
source ~/.zshrc
```

**Verify Go Installation:**
```bash
go version
# Should show: go version go1.21.x darwin/amd64 (or darwin/arm64)
```

#### Install CloudLab

**Method 1: Using Build Script (Recommended)**
```bash
# Clone repository
git clone https://github.com/Sakib-Dalal/CloudLab.git
cd CloudLab

# Make executable and run
chmod +x build.sh
./build.sh

# Follow the prompts
```

**Method 2: Using Make**
```bash
git clone https://github.com/Sakib-Dalal/CloudLab.git
cd CloudLab
make build
make install
```

**Method 3: Manual Build**
```bash
# Clone repository
git clone https://github.com/Sakib-Dalal/CloudLab.git
cd CloudLab

# Initialize module
go mod init cloudlab
go get golang.org/x/text/cases
go get golang.org/x/text/language
go mod tidy

# Build
CGO_ENABLED=0 go build -ldflags="-s -w" -o cloudlab cloudlab.go

# Install
sudo mv cloudlab /usr/local/bin/
sudo chmod +x /usr/local/bin/cloudlab

# Verify
cloudlab version
```

---

### 🐧 Linux Installation

#### Prerequisites: Install Go

**Ubuntu / Debian**
```bash
# Update packages
sudo apt update

# Option 1: Using apt (may not be latest version)
sudo apt install -y golang-go

# Option 2: Manual installation (recommended for latest version)
wget https://go.dev/dl/go1.21.6.linux-amd64.tar.gz
sudo rm -rf /usr/local/go
sudo tar -C /usr/local -xzf go1.21.6.linux-amd64.tar.gz

# Add to PATH
echo 'export PATH=$PATH:/usr/local/go/bin' >> ~/.bashrc
source ~/.bashrc
```

**CentOS / RHEL / Fedora**
```bash
# Using dnf
sudo dnf install -y golang

# Or manual installation
wget https://go.dev/dl/go1.21.6.linux-amd64.tar.gz
sudo rm -rf /usr/local/go
sudo tar -C /usr/local -xzf go1.21.6.linux-amd64.tar.gz

# Add to PATH
echo 'export PATH=$PATH:/usr/local/go/bin' >> ~/.bashrc
source ~/.bashrc
```

**Arch Linux**
```bash
sudo pacman -S go
```

**Alpine Linux**
```bash
apk add go
```

**ARM64 / Raspberry Pi**
```bash
wget https://go.dev/dl/go1.21.6.linux-arm64.tar.gz
sudo rm -rf /usr/local/go
sudo tar -C /usr/local -xzf go1.21.6.linux-arm64.tar.gz

echo 'export PATH=$PATH:/usr/local/go/bin' >> ~/.bashrc
source ~/.bashrc
```

**Verify Go Installation:**
```bash
go version
# Should show: go version go1.21.x linux/amd64 (or linux/arm64)
```

#### Install CloudLab

**Method 1: Using Build Script (Recommended)**
```bash
# Clone repository
git clone https://github.com/Sakib-Dalal/CloudLab.git
cd CloudLab

# Make executable and run
chmod +x build.sh
./build.sh

# Follow the prompts
```

**Method 2: Using Make**
```bash
git clone https://github.com/Sakib-Dalal/CloudLab.git
cd CloudLab
make build
sudo make install
```

**Method 3: Manual Build**
```bash
# Clone repository
git clone https://github.com/Sakib-Dalal/CloudLab.git
cd CloudLab

# Install dependencies
sudo apt install -y curl wget git  # Debian/Ubuntu
# or
sudo dnf install -y curl wget git  # Fedora/CentOS

# Initialize module
go mod init cloudlab
go get golang.org/x/text/cases
go get golang.org/x/text/language
go mod tidy

# Build
CGO_ENABLED=0 go build -ldflags="-s -w" -o cloudlab cloudlab.go

# Install
sudo mv cloudlab /usr/local/bin/
sudo chmod +x /usr/local/bin/cloudlab

# Verify
cloudlab version
```

**Method 4: Install as Systemd Service**
```bash
# After installing cloudlab, create service
sudo tee /etc/systemd/system/cloudlab.service > /dev/null << 'EOF'
[Unit]
Description=CloudLab Services
After=network.target

[Service]
Type=forking
User=your-username
ExecStart=/usr/local/bin/cloudlab start all
ExecStop=/usr/local/bin/cloudlab stop all
RemainAfterExit=yes
Restart=on-failure

[Install]
WantedBy=multi-user.target
EOF

# Enable and start
sudo systemctl daemon-reload
sudo systemctl enable cloudlab
sudo systemctl start cloudlab
```

---

### 🐳 Docker Installation

Docker is the easiest way to get CloudLab running without installing Go or any dependencies.

#### Prerequisites

**Windows**
```powershell
# Install Docker Desktop
winget install Docker.DockerDesktop
# Restart your computer
# Open Docker Desktop
```

**macOS**
```bash
# Using Homebrew
brew install --cask docker
# Open Docker Desktop from Applications
```

**Linux**
```bash
# Ubuntu/Debian
sudo apt update
sudo apt install -y docker.io docker-compose
sudo systemctl start docker
sudo systemctl enable docker

# Add user to docker group (to run without sudo)
sudo usermod -aG docker $USER
newgrp docker
```

#### Run CloudLab with Docker

**Method 1: Docker Compose (Recommended)**
```bash
# Clone repository
git clone https://github.com/Sakib-Dalal/CloudLab.git
cd CloudLab

# Start all services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop services
docker-compose down
```

**Method 2: Docker Build & Run**
```bash
# Clone repository
git clone https://github.com/Sakib-Dalal/CloudLab.git
cd CloudLab

# Build image
docker build -t cloudlab:latest .

# Run container
docker run -d \
  --name cloudlab \
  -p 8888:8888 \
  -p 8080:8080 \
  -p 7681:7681 \
  -p 3000:3000 \
  -v $(pwd)/workspace:/home/cloudlab/workspace \
  cloudlab:latest
```

**Method 3: Docker with Environment Variables**
```bash
docker run -d \
  --name cloudlab \
  -p 8888:8888 \
  -p 8080:8080 \
  -p 7681:7681 \
  -p 3000:3000 \
  -e JUPYTER_PASSWORD=mypassword \
  -e VSCODE_PASSWORD=mypassword \
  -e EMAIL_ADDRESS=your@email.com \
  -e EMAIL_PASSWORD=your-app-password \
  -e ENABLE_TUNNELS=true \
  -v $(pwd)/workspace:/home/cloudlab/workspace \
  cloudlab:latest
```

**Method 4: Docker with GPU Support (NVIDIA)**
```bash
# Install NVIDIA Container Toolkit first
# https://docs.nvidia.com/datacenter/cloud-native/container-toolkit/install-guide.html

docker run -d \
  --name cloudlab \
  --gpus all \
  -p 8888:8888 \
  -p 8080:8080 \
  -p 7681:7681 \
  -p 3000:3000 \
  -v $(pwd)/workspace:/home/cloudlab/workspace \
  cloudlab:latest
```

#### Docker Commands

```bash
# View running containers
docker ps

# View logs
docker logs -f cloudlab

# Access container shell
docker exec -it cloudlab bash

# Stop container
docker stop cloudlab

# Remove container
docker rm cloudlab

# Remove image
docker rmi cloudlab:latest
```

#### Docker Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `JUPYTER_PASSWORD` | `cloudlab` | Jupyter password |
| `VSCODE_PASSWORD` | `cloudlab` | VS Code password |
| `EMAIL_ADDRESS` | - | Email for notifications |
| `EMAIL_PASSWORD` | - | Email app password |
| `ENABLE_TUNNELS` | `false` | Enable Cloudflare tunnels |
| `WORKING_DIR` | `/home/cloudlab/workspace` | Working directory |

---

## 🚀 Quick Start

After installation, follow these steps:

```bash
# Step 1: Initialize CloudLab (configure settings)
cloudlab init

# Step 2: Install all components
cloudlab install all

# Step 3: Start all services
cloudlab start all

# Step 4: Check status
cloudlab status

# Step 5: Open dashboard in browser
# http://localhost:3000

# All 4 tunnel URLs will be sent to your email!
```

---

## 📖 Commands Reference

### Service Commands

```bash
cloudlab init                 # Initialize configuration
cloudlab install all          # Install all components
cloudlab install jupyter      # Install Jupyter only
cloudlab install vscode       # Install VS Code only
cloudlab install ssh          # Install SSH terminal (ttyd)
cloudlab install cloudflare   # Install cloudflared
cloudlab install uv           # Install UV package manager

cloudlab start all            # Start all services + tunnels
cloudlab start jupyter        # Start Jupyter Lab
cloudlab start notebook       # Start Jupyter Notebook
cloudlab start vscode         # Start VS Code
cloudlab start ssh            # Start SSH terminal
cloudlab start dashboard      # Start web dashboard
cloudlab start tunnel         # Start all tunnels

cloudlab stop all             # Stop everything
cloudlab stop jupyter         # Stop Jupyter
cloudlab stop vscode          # Stop VS Code
cloudlab stop ssh             # Stop SSH terminal
cloudlab stop dashboard       # Stop dashboard
cloudlab stop tunnel          # Stop all tunnels

cloudlab restart all          # Restart everything
cloudlab status               # Show status of all services
cloudlab logs jupyter         # View Jupyter logs
cloudlab logs vscode          # View VS Code logs
cloudlab logs ssh             # View SSH logs
cloudlab logs dashboard       # View dashboard logs
```

### Tunnel Commands

```bash
cloudlab tunnel start         # Start all Cloudflare tunnels
cloudlab tunnel stop          # Stop all tunnels
cloudlab tunnel restart       # Get new tunnel URLs
cloudlab tunnel status        # Show current URLs
```

### SSH Terminal Commands

```bash
cloudlab ssh start            # Start web SSH terminal
cloudlab ssh stop             # Stop SSH terminal
cloudlab ssh config           # Configure SSH settings
cloudlab ssh status           # Show SSH status
```

### Dashboard Commands

```bash
cloudlab dashboard start      # Start web dashboard
cloudlab dashboard stop       # Stop dashboard
cloudlab dashboard status     # Show dashboard status
```

### Kernel Management

```bash
cloudlab kernel list                  # List Jupyter kernels
cloudlab kernel add mykernel          # Add kernel with default Python
cloudlab kernel add mykernel 3.10     # Add kernel with Python 3.10
cloudlab kernel remove mykernel       # Remove kernel
```

### Environment Management

```bash
cloudlab env list                     # List Python environments
cloudlab env create myenv 3.11        # Create Python 3.11 environment
cloudlab env remove myenv             # Remove environment
cloudlab env install numpy            # Install package in default env
cloudlab env install "numpy pandas"   # Install multiple packages
```

### Email Commands

```bash
cloudlab email setup          # Setup email notifications
cloudlab email test           # Send test email
cloudlab email send           # Send all tunnel URLs via email
```

### Configuration Commands

```bash
cloudlab config                             # Show configuration
cloudlab config set jupyter_port 9999       # Change Jupyter port
cloudlab config set vscode_port 8081        # Change VS Code port
cloudlab config set ssh_port 7682           # Change SSH port
cloudlab config set dashboard_port 3001     # Change dashboard port
cloudlab config set jupyter_mode notebook   # Switch to notebook mode
cloudlab config set working_directory /path # Change working directory
cloudlab config set jupyter_password pass   # Change Jupyter password
cloudlab config set vscode_password pass    # Change VS Code password
cloudlab config reset                       # Reset to defaults
```

### Other Commands

```bash
cloudlab update               # Update all components
cloudlab uninstall            # Uninstall CloudLab
cloudlab version              # Show version
cloudlab help                 # Show help
```

---

## ⚙️ Configuration

Configuration is stored in `~/.cloudlab/config.json`

| Key | Description | Default |
|-----|-------------|---------|
| `jupyter_port` | Jupyter port | `8888` |
| `vscode_port` | VS Code port | `8080` |
| `ssh_port` | SSH Terminal port | `7681` |
| `dashboard_port` | Dashboard port | `3000` |
| `jupyter_mode` | `lab` or `notebook` | `lab` |
| `python_version` | Python version | `3.11` |
| `working_directory` | Project directory | `~` |
| `jupyter_password` | Jupyter password | Auto-generated |
| `vscode_password` | VS Code password | Auto-generated |
| `ssh_user` | SSH username | Current user |
| `email_address` | Email for notifications | - |
| `enable_mps` | Apple Silicon MPS | Auto-detected |
| `enable_cuda` | NVIDIA CUDA | Auto-detected |
| `low_power_mode` | Energy saving mode | `true` |
| `notify_on_start` | Email on start | `true` |

---

## 📧 Email Setup

### Gmail Setup

1. Go to https://myaccount.google.com/apppasswords
2. Sign in to your Google account
3. Select "Mail" as the app
4. Select your device
5. Click "Generate"
6. Copy the 16-character password
7. Use this password in `cloudlab init` or `cloudlab email setup`

### Outlook/Hotmail Setup

1. Go to https://account.microsoft.com/security
2. Sign in to your account
3. Enable Two-Factor Authentication
4. Go to "App passwords"
5. Create a new app password
6. Copy and use this password

### Yahoo Setup

1. Go to https://login.yahoo.com/account/security
2. Enable Two-Step Verification
3. Go to "Generate app password"
4. Select "Other App"
5. Copy and use this password

---

## 🔧 Troubleshooting

### Service Not Starting

```bash
# Check logs
cloudlab logs jupyter
cloudlab logs vscode
cloudlab logs ssh
cloudlab logs dashboard

# Reinstall component
cloudlab install jupyter
cloudlab install vscode
cloudlab install ssh
```

### Tunnel URLs Not Working

```bash
# Restart tunnels
cloudlab tunnel restart

# Check tunnel logs
cloudlab logs tunnel_jupyter
cloudlab logs tunnel_vscode
cloudlab logs tunnel_ssh
cloudlab logs tunnel_dashboard
```

### Email Not Sending

```bash
# Test email configuration
cloudlab email test

# Re-setup email
cloudlab email setup
```

### VS Code Showing Stopped But Running

The dashboard now checks both PID and port. If still having issues:

```bash
# Restart dashboard
cloudlab stop dashboard
cloudlab start dashboard
```

### Port Already in Use

```bash
# Change port
cloudlab config set jupyter_port 9999
cloudlab config set vscode_port 8081

# Restart services
cloudlab restart all
```

### Permission Denied (Linux/macOS)

```bash
# Make sure cloudlab is executable
sudo chmod +x /usr/local/bin/cloudlab

# Or rebuild with proper permissions
sudo make install
```

### Go Not Found After Installation

```bash
# Add Go to PATH
export PATH=$PATH:/usr/local/go/bin

# Make permanent (add to shell config)
echo 'export PATH=$PATH:/usr/local/go/bin' >> ~/.bashrc  # Linux
echo 'export PATH=$PATH:/usr/local/go/bin' >> ~/.zshrc   # macOS
source ~/.bashrc  # or source ~/.zshrc
```

### Docker: Container Exits Immediately

```bash
# Check logs
docker logs cloudlab

# Run in foreground to debug
docker run -it --rm \
  -p 8888:8888 -p 8080:8080 -p 7681:7681 -p 3000:3000 \
  cloudlab:latest
```

---

## 📁 File Locations

```
~/.cloudlab/
├── config.json          # Configuration file
├── venv/                # Main Python environment
├── envs/                # Additional environments
├── logs/                # Service logs
│   ├── jupyter.log
│   ├── vscode.log
│   ├── ssh.log
│   ├── dashboard.log
│   ├── tunnel_jupyter.log
│   ├── tunnel_vscode.log
│   ├── tunnel_ssh.log
│   └── tunnel_dashboard.log
├── pids/                # Process ID files
├── dashboard.html       # Web dashboard
└── server.py            # Dashboard server
```

---

## 🛠️ Building from Source

### Prerequisites

- Go 1.21 or later
- Git

### Build Commands

```bash
# Clone repository
git clone https://github.com/Sakib-Dalal/CloudLab.git
cd CloudLab

# Using Make
make build           # Build for current platform
make build-all       # Build for all platforms
make install         # Install to system
make clean           # Clean build files

# Using build.sh
chmod +x build.sh
./build.sh

# Manual build
go mod init cloudlab
go get golang.org/x/text/cases
go get golang.org/x/text/language
go mod tidy
CGO_ENABLED=0 go build -ldflags="-s -w" -o cloudlab cloudlab.go
```

---

## 👤 Author

**Sakib Dalal**

- GitHub: [@Sakib-Dalal](https://github.com/Sakib-Dalal)
- Repository: [CloudLab](https://github.com/Sakib-Dalal/CloudLab)

---

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

---

## 🙏 Acknowledgments

- [UV Package Manager](https://github.com/astral-sh/uv) - Fast Python package manager
- [code-server](https://github.com/coder/code-server) - VS Code in the browser
- [ttyd](https://github.com/tsl0922/ttyd) - Web terminal
- [cloudflared](https://github.com/cloudflare/cloudflared) - Cloudflare tunnel client
- [JupyterLab](https://github.com/jupyterlab/jupyterlab) - Web-based notebook IDE

---

<p align="center">
  Made with ❤️ by <a href="https://github.com/Sakib-Dalal">Sakib Dalal</a>
</p>