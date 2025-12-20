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
[![Docs](https://img.shields.io/badge/Docs-CloudLab-green?style=for-the-badge)](https://cloudlab-alpha.vercel.app/)


## ✨ Features

- 📓 **Jupyter Lab & Notebook** - Full Python notebook environment
- 💻 **VS Code Server** - Browser-based code editor
- 🔒 **SSH Terminal** - Web-based terminal access (ttyd)
- 📊 **Web Dashboard** - Manage everything from browser
- 🌐 **Cloudflare Tunnels** - Free public URLs (no account needed!)
- 📧 **Email Notifications** - Receive all URLs via email
- 🐍 **Kernel Management** - Add/remove Jupyter kernels
- 📦 **Environment Management** - Create Python environments
- ⚡ **MPS/CUDA Support** - GPU acceleration for Apple/NVIDIA
- 🔋 **Low Power Mode** - Energy efficient

## 🚀 Quick Start

```bash
# 1. Clone
git clone https://github.com/Sakib-Dalal/cloudlab.git
cd cloudlab

# 2. Build
chmod +x build.sh
./build.sh

# 3. Initialize
cloudlab init

# 4. Install components
cloudlab install all

# 5. Start everything
cloudlab start all

# 6. Check status
cloudlab status
```

## 📖 Commands

### Services
```bash
cloudlab start all          # Start all services + tunnels
cloudlab start jupyter      # Start Jupyter Lab
cloudlab start notebook     # Start Jupyter Notebook
cloudlab start vscode       # Start VS Code
cloudlab start ssh          # Start SSH Terminal
cloudlab start dashboard    # Start Web Dashboard
cloudlab stop all           # Stop everything
cloudlab restart all        # Restart everything
cloudlab status             # Show status and URLs
```

### Tunnels
```bash
cloudlab tunnel start       # Start all tunnels, get public URLs
cloudlab tunnel stop        # Stop all tunnels
cloudlab tunnel restart     # Get new URLs
cloudlab tunnel status      # Show current URLs
```

### SSH Terminal
```bash
cloudlab ssh start          # Start SSH terminal
cloudlab ssh stop           # Stop SSH terminal
cloudlab ssh config         # Configure SSH settings
cloudlab ssh status         # Show SSH status
```

### Dashboard
```bash
cloudlab dashboard start    # Start web dashboard
cloudlab dashboard stop     # Stop dashboard
cloudlab dashboard status   # Show dashboard status
```

### Email
```bash
cloudlab email setup        # Configure email (Gmail, Outlook, etc.)
cloudlab email test         # Send test email
cloudlab email send         # Send all tunnel URLs via email
```

### Kernels
```bash
cloudlab kernel list                  # List Jupyter kernels
cloudlab kernel add mykernel 3.10     # Add kernel with Python 3.10
cloudlab kernel remove mykernel       # Remove kernel
```

### Environments
```bash
cloudlab env list                     # List Python environments
cloudlab env create myenv 3.11        # Create Python 3.11 environment
cloudlab env remove myenv             # Remove environment
cloudlab env install numpy            # Install package
```

### Configuration
```bash
cloudlab config                             # Show config
cloudlab config set jupyter_mode notebook   # Change Jupyter mode
cloudlab config set working_directory /path # Set project directory
cloudlab config reset                       # Reset to defaults
```

## 🌐 How Tunnels Work

CloudLab uses **Cloudflare Quick Tunnels** (TryCloudflare):

- ✅ **No Cloudflare account required**
- ✅ Creates URLs like `https://random-words.trycloudflare.com`
- ✅ All 4 services get their own tunnel (Jupyter, VS Code, SSH, Dashboard)
- ⚠️ URLs change when you restart tunnels

```bash
# Get new URLs
cloudlab tunnel restart

# Send URLs to email
cloudlab email send
```

## 📧 Email Setup

### Gmail
1. Go to https://myaccount.google.com/apppasswords
2. Create App Password for "Mail"
3. Use during `cloudlab init` or `cloudlab email setup`

### Outlook
1. Go to https://account.microsoft.com/security
2. Enable 2FA
3. Create App Password

## 📊 Web Dashboard

Access the dashboard at `http://localhost:3000`:

- View all service status
- Start/stop services with one click
- Copy tunnel URLs
- View credentials
- Run terminal commands
- Manage kernels and environments

## 📁 File Locations

```
~/.cloudlab/
├── config.json          # Configuration
├── venv/                # Main Python environment
├── envs/                # Additional environments
├── logs/                # Service logs
│   ├── jupyter.log
│   ├── vscode.log
│   ├── ssh.log
│   ├── dashboard.log
│   └── tunnel_*.log
├── pids/                # Process IDs
├── dashboard.html       # Web dashboard
└── server.py            # Dashboard server
```

## ⚙️ Configuration

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
| `email_address` | Notification email | - |

## 🔧 Troubleshooting

### Service not starting
```bash
# Check logs
cloudlab logs jupyter
cloudlab logs vscode
cloudlab logs ssh

# Reinstall
cloudlab install jupyter
cloudlab install vscode
cloudlab install ssh
```

### Tunnel URLs not working
```bash
# Restart tunnels
cloudlab tunnel restart

# Check tunnel logs
cloudlab logs tunnel_jupyter
cloudlab logs tunnel_vscode
cloudlab logs tunnel_ssh
cloudlab logs tunnel_dashboard
```

### Email not sending
```bash
# Test email config
cloudlab email test

# Re-setup email
cloudlab email setup
```

## 👤 Author

**Sakib Dalal**

- GitHub: [@Sakib-Dalal](https://github.com/Sakib-Dalal)

## 📄 License

MIT License

---

Made with ❤️ by [Sakib Dalal](https://github.com/Sakib-Dalal)
