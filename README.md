# ☁️ CloudLab CLI

**Self-hosted web development environment with Jupyter Lab/Notebook, VS Code Server, Web SSH Terminal, and Cloudflare Tunnels.**

Turn any computer, laptop, or cloud instance into a remote development environment accessible from anywhere.

Documentation: https://cloudlab-alpha.vercel.app/

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
[![License](https://img.shields.io/badge/License-MIT-green?style=for-the-badge)](LICENSE)
[![Version](https://img.shields.io/badge/Version-1.1.0-blue?style=for-the-badge)](https://github.com/Sakib-Dalal/CloudLab)
[![WebSite](https://img.shields.io/badge/WebSite-CloudLab-red?style=for-the-badge)](https://cloudlab-alpha.vercel.app/)

## ✨ Features

- 🚀 **One-command setup** - `cloudlab init` configures everything
- 🌐 **Free Cloudflare Tunnels** - Public URLs without account or token
- 📧 **Email notifications** - Receive tunnel URLs automatically
- 📓 **Jupyter Lab & Notebook** - Full-featured notebook environment
- 💻 **VS Code Server** - Browser-based code editor
- 🔒 **Web SSH Terminal** - Browser-based SSH access via ttyd *(NEW!)*
- 🐍 **UV Package Manager** - Fast Python environment management
- ⚡ **Apple MPS** and **NVIDIA CUDA** support
- 🔋 **Low power mode** - Optimized for energy efficiency
- 🎨 **Colorful CLI** - Beautiful terminal output

## 🆕 What's New in v1.1.0

### 🔒 Web-Based SSH Terminal
Access your server's terminal directly from your browser! No SSH client needed.

- **ttyd Integration** - Lightweight web-based terminal
- **Cloudflare Tunnel Support** - Secure public SSH access
- **Email Notifications** - SSH URL included in emails
- **Cross-Platform** - Works on Linux, macOS, and Windows

```bash
# Start SSH terminal
cloudlab start ssh

# Configure SSH settings
cloudlab ssh config

# Get public SSH URL
cloudlab tunnel start
```

## 🚀 Quick Start

```bash
# 1. Clone the repository
git clone https://github.com/Sakib-Dalal/cloudlab.git
cd cloudlab

# 2. Build
chmod +x build.sh
./build.sh

# 3. Initialize (interactive setup)
cloudlab init

# 4. Install all components (including SSH terminal)
cloudlab install all

# 5. Start everything (URLs sent to your email!)
cloudlab start all

# 6. Check status
cloudlab status
```

## 📦 Installation

### Prerequisites

**macOS:**
```bash
brew install go
```

**Linux (Ubuntu/Debian):**
```bash
# 1. Remove any existing Go installation (safe if none exists)
sudo rm -rf /usr/local/go

# 2. Download Go 1.21.5 (amd64)
wget https://go.dev/dl/go1.21.5.linux-amd64.tar.gz

# 3. Extract Go to /usr/local
sudo tar -C /usr/local -xzf go1.21.5.linux-amd64.tar.gz

# 4. Add Go to PATH only if not already added
grep -qxF 'export PATH=$PATH:/usr/local/go/bin' ~/.bashrc || \
echo 'export PATH=$PATH:/usr/local/go/bin' >> ~/.bashrc

# 5. Reload shell
source ~/.bashrc
```

**Windows (PowerShell as Admin):**
```powershell
winget install GoLang.Go
```

### Build & Install

```bash
# Clone repository
git clone https://github.com/Sakib-Dalal/cloudlab.git
cd cloudlab

# Build
chmod +x build.sh
./build.sh

# Or build manually
go mod init cloudlab
go build -ldflags="-s -w" -o cloudlab cloudlab.go
sudo mv cloudlab /usr/local/bin/
```

## 📖 Commands

### Services

| Command | Description |
|---------|-------------|
| `cloudlab start all` | Start all services + tunnels |
| `cloudlab start lab` | Start Jupyter Lab |
| `cloudlab start notebook` | Start Jupyter Notebook |
| `cloudlab start vscode` | Start VS Code Server |
| `cloudlab start ssh` | Start Web SSH Terminal |
| `cloudlab start tunnel` | Start Cloudflare tunnels |
| `cloudlab stop all` | Stop all services |
| `cloudlab stop ssh` | Stop SSH terminal |
| `cloudlab restart all` | Restart all services |
| `cloudlab status` | Show status and URLs |
| `cloudlab logs <service>` | View service logs |

### Configuration

| Command | Description |
|---------|-------------|
| `cloudlab init` | Interactive setup |
| `cloudlab config` | Show current configuration |
| `cloudlab config set <key> <value>` | Set configuration value |
| `cloudlab config reset` | Reset to defaults |

### Tunnels

| Command | Description |
|---------|-------------|
| `cloudlab tunnel start` | Start tunnels and get URLs |
| `cloudlab tunnel stop` | Stop tunnels |
| `cloudlab tunnel restart` | Get new URLs |
| `cloudlab tunnel status` | Show current URLs |

### SSH Terminal

| Command | Description |
|---------|-------------|
| `cloudlab ssh` | Show SSH terminal status |
| `cloudlab ssh config` | Configure SSH settings |
| `cloudlab ssh start` | Start SSH terminal |
| `cloudlab ssh stop` | Stop SSH terminal |
| `cloudlab ssh restart` | Restart SSH terminal |
| `cloudlab start ssh` | Start SSH terminal (alternative) |
| `cloudlab stop ssh` | Stop SSH terminal (alternative) |
| `cloudlab logs ssh` | View SSH terminal logs |

### Email

| Command | Description |
|---------|-------------|
| `cloudlab email setup` | Configure email notifications |
| `cloudlab email test` | Send test email |
| `cloudlab email send` | Send tunnel URLs via email (includes SSH) |

### Kernels

| Command | Description |
|---------|-------------|
| `cloudlab kernel list` | List Jupyter kernels |
| `cloudlab kernel add <name>` | Add kernel with default Python |
| `cloudlab kernel add <name> <version>` | Add kernel with specific Python |
| `cloudlab kernel remove <name>` | Remove kernel |

### Environments

| Command | Description |
|---------|-------------|
| `cloudlab env list` | List Python environments |
| `cloudlab env create <name> <version>` | Create new environment |
| `cloudlab env remove <name>` | Remove environment |
| `cloudlab env activate <name>` | Show activation command |
| `cloudlab env install <package>` | Install package |

## ⚙️ Configuration Options

### General Settings

| Key | Description | Default |
|-----|-------------|---------|
| `working_directory` | Project directory | `~` |
| `jupyter_mode` | `lab` or `notebook` | `lab` |
| `jupyter_port` | Jupyter port | `8888` |
| `vscode_port` | VS Code port | `8080` |
| `python_version` | Python version | `3.11` |
| `jupyter_password` | Jupyter password | Auto-generated |
| `vscode_password` | VS Code password | Auto-generated |
| `email_address` | Notification email | - |
| `enable_mps` | Apple MPS acceleration | Auto-detected |
| `enable_cuda` | NVIDIA CUDA acceleration | Auto-detected |
| `low_power_mode` | Use less CPU/memory | `true` |
| `notify_on_start` | Email URLs on tunnel start | `true` |

### SSH Terminal Settings

| Key | Description | Default |
|-----|-------------|---------|
| `ssh_port` | Web SSH terminal port | `2222` |
| `ssh_user` | SSH username | Current user |
| `ssh_password` | SSH password (optional, for web auth) | System auth |
| `ssh_host` | SSH target host | `localhost` |
| `ssh_target_port` | SSH target port | `22` |
| `ssh_enabled` | Enable SSH terminal | `true` |

### Examples

```bash
# Change Jupyter mode to notebook
cloudlab config set jupyter_mode notebook

# Change working directory
cloudlab config set working_directory /path/to/projects

# Change port
cloudlab config set jupyter_port 9999

# Change password
cloudlab config set jupyter_password mysecretpassword

# Configure SSH terminal port
cloudlab config set ssh_port 3000

# Set SSH username
cloudlab config set ssh_user myuser

# Disable SSH terminal
cloudlab config set ssh_enabled false
```

## 🔒 SSH Terminal Setup

The web-based SSH terminal uses [ttyd](https://github.com/tsl0922/ttyd) to provide browser-based terminal access.

### Installation

ttyd is automatically installed when you run:
```bash
cloudlab install all
# or
cloudlab install ssh
```

### Manual Installation

**macOS:**
```bash
brew install ttyd
```

**Ubuntu/Debian:**
```bash
sudo apt-get install ttyd
```

**Other Linux (binary):**
```bash
# x86_64
wget https://github.com/tsl0922/ttyd/releases/latest/download/ttyd.x86_64
chmod +x ttyd.x86_64
sudo mv ttyd.x86_64 /usr/local/bin/ttyd

# ARM64
wget https://github.com/tsl0922/ttyd/releases/latest/download/ttyd.aarch64
chmod +x ttyd.aarch64
sudo mv ttyd.aarch64 /usr/local/bin/ttyd
```

### Usage

```bash
# Start SSH terminal
cloudlab start ssh

# Access locally
# Open http://localhost:2222 in your browser

# Get public URL via Cloudflare tunnel
cloudlab tunnel start

# Check status
cloudlab ssh status
```

### Security

- **Password Protection**: Set a password for web authentication:
  ```bash
  cloudlab config set ssh_password mysecretpassword
  ```
- **HTTPS**: When using Cloudflare tunnels, all traffic is encrypted
- **Authentication**: Uses system credentials or custom password

## 📧 Email Setup

Email notifications now include SSH terminal URL!

### Gmail
1. Go to https://myaccount.google.com/apppasswords
2. Create an App Password for "Mail"
3. Use this password with `cloudlab email setup`

### Outlook/Hotmail
1. Go to https://account.microsoft.com/security
2. Enable 2FA
3. Create an App Password

### Email Contents
When you run `cloudlab email send`, you'll receive:
- 🐍 Jupyter Lab/Notebook URL + Password
- 💻 VS Code Server URL + Password
- 🔒 SSH Terminal URL + Username *(NEW!)*

## 📁 File Locations

```
~/.cloudlab/
├── config.json          # Configuration
├── venv/                # Main Python environment
├── envs/                # Additional environments
│   └── <name>/          # Named environments
├── logs/
│   ├── jupyter.log
│   ├── vscode.log
│   ├── ssh.log          # SSH terminal logs (NEW!)
│   ├── tunnel_jupyter.log
│   ├── tunnel_vscode.log
│   └── tunnel_ssh.log   # SSH tunnel logs (NEW!)
├── pids/
│   ├── jupyter.pid
│   ├── vscode.pid
│   ├── ssh.pid          # SSH terminal PID (NEW!)
│   ├── tunnel_jupyter.pid
│   ├── tunnel_vscode.pid
│   └── tunnel_ssh.pid   # SSH tunnel PID (NEW!)
└── vscode-data/         # VS Code user data
```

## ☁️ Cloud Deployment

### AWS EC2 / Google Cloud / Azure

```bash
# SSH into your instance
ssh user@your-instance-ip

# Install Go
wget https://go.dev/dl/go1.21.5.linux-amd64.tar.gz
sudo tar -C /usr/local -xzf go1.21.5.linux-amd64.tar.gz
export PATH=$PATH:/usr/local/go/bin

# Clone and build
git clone https://github.com/Sakib-Dalal/cloudlab.git
cd cloudlab
./build.sh

# Initialize and start
cloudlab init
cloudlab install all
cloudlab start all

# Your SSH terminal is now accessible via Cloudflare tunnel!
```

### Run on Startup (systemd)

```bash
sudo tee /etc/systemd/system/cloudlab.service << EOF
[Unit]
Description=CloudLab Services
After=network.target

[Service]
Type=forking
User=$USER
ExecStart=/usr/local/bin/cloudlab start all
ExecStop=/usr/local/bin/cloudlab stop all
Restart=on-failure

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl daemon-reload
sudo systemctl enable cloudlab
sudo systemctl start cloudlab
```

## 🌐 How Tunnels Work

CloudLab uses **Cloudflare Quick Tunnels** (TryCloudflare):

- ✅ No Cloudflare account required
- ✅ Creates temporary public URLs like `https://random-words.trycloudflare.com`
- ✅ Separate tunnels for Jupyter, VS Code, and SSH
- ⚠️ URLs change when you restart tunnels

To get new URLs:
```bash
cloudlab tunnel restart
cloudlab email send  # Send new URLs to your email
```

### Tunnel URLs
After starting tunnels, you'll get three public URLs:
1. **Jupyter**: `https://jupyter-xyz.trycloudflare.com`
2. **VS Code**: `https://vscode-abc.trycloudflare.com`
3. **SSH Terminal**: `https://ssh-def.trycloudflare.com` *(NEW!)*

## 🔧 Troubleshooting

### "Jupyter not found"
```bash
cloudlab install jupyter
```

### "UV not found"
```bash
cloudlab install uv
```

### "ttyd not found" (SSH terminal)
```bash
cloudlab install ssh
# or manually
brew install ttyd  # macOS
sudo apt-get install ttyd  # Ubuntu/Debian
```

### Port already in use
```bash
cloudlab config set jupyter_port 9999
cloudlab config set ssh_port 3000
cloudlab restart all
```

### View logs
```bash
cloudlab logs jupyter
cloudlab logs vscode
cloudlab logs ssh
cloudlab logs tunnel_jupyter
cloudlab logs tunnel_vscode
cloudlab logs tunnel_ssh
```

### Reset everything
```bash
cloudlab stop all
cloudlab config reset
cloudlab init
```

### SSH terminal not starting
```bash
# Check if ttyd is installed
which ttyd

# View SSH logs
cloudlab logs ssh

# Try manual start
ttyd --port 2222 bash
```

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        CloudLab CLI                          │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐          │
│  │   Jupyter   │  │   VS Code   │  │     SSH     │          │
│  │  Lab/NB     │  │   Server    │  │   Terminal  │          │
│  │  :8888      │  │   :8080     │  │   :2222     │          │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘          │
│         │                │                │                  │
│  ┌──────┴──────┐  ┌──────┴──────┐  ┌──────┴──────┐          │
│  │  Cloudflare │  │  Cloudflare │  │  Cloudflare │          │
│  │   Tunnel    │  │   Tunnel    │  │   Tunnel    │          │
│  └──────┬──────┘  └──────┴──────┘  └──────┬──────┘          │
│         │                │                │                  │
│         └────────────────┼────────────────┘                  │
│                          │                                   │
│                   ┌──────┴──────┐                           │
│                   │    Email    │                           │
│                   │ Notification│                           │
│                   └─────────────┘                           │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## 👤 Author

**Sakib Dalal**

- GitHub: [@Sakib-Dalal](https://github.com/Sakib-Dalal)

## 📄 License

MIT License - see the [LICENSE](LICENSE) file for details.

## 🤝 Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 🙏 Acknowledgments

- [ttyd](https://github.com/tsl0922/ttyd) - Web-based terminal
- [code-server](https://github.com/coder/code-server) - VS Code in the browser
- [Cloudflare](https://www.cloudflare.com/) - Free tunneling service
- [UV](https://github.com/astral-sh/uv) - Fast Python package manager

---

Made with ❤️ by [Sakib Dalal](https://github.com/Sakib-Dalal)
