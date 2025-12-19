# ☁️ CloudLab CLI

**Self-hosted web development environment with Jupyter Lab/Notebook, VS Code Server, and Cloudflare Tunnels.**

Turn any computer, laptop, or cloud instance into a remote development environment accessible from anywhere.

Documentation: https://cloud-lab-gilt.vercel.app/

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
[![WebSite](https://img.shields.io/badge/WebSite-CloudLab-red?style=for-the-badge)](https://cloud-lab-gilt.vercel.app/)

## ✨ Features

- 🚀 **One-command setup** - `cloudlab init` configures everything
- 🌐 **Free Cloudflare Tunnels** - Public URLs without account or token
- 📧 **Email notifications** - Receive tunnel URLs automatically
- 📓 **Jupyter Lab & Notebook** - Full-featured notebook environment
- 💻 **VS Code Server** - Browser-based code editor
- 🐍 **UV Package Manager** - Fast Python environment management
- ⚡ **Apple MPS** and **NVIDIA CUDA** support
- 🔋 **Low power mode** - Optimized for energy efficiency
- 🎨 **Colorful CLI** - Beautiful terminal output

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

# 4. Install all components
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
wget https://go.dev/dl/go1.21.5.linux-amd64.tar.gz
sudo tar -C /usr/local -xzf go1.21.5.linux-amd64.tar.gz
echo 'export PATH=$PATH:/usr/local/go/bin' >> ~/.bashrc
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
| `cloudlab start tunnel` | Start Cloudflare tunnels |
| `cloudlab stop all` | Stop all services |
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

### Email

| Command | Description |
|---------|-------------|
| `cloudlab email setup` | Configure email notifications |
| `cloudlab email test` | Send test email |
| `cloudlab email send` | Send tunnel URLs via email |

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
```

## 📧 Email Setup

### Gmail
1. Go to https://myaccount.google.com/apppasswords
2. Create an App Password for "Mail"
3. Use this password with `cloudlab email setup`

### Outlook/Hotmail
1. Go to https://account.microsoft.com/security
2. Enable 2FA
3. Create an App Password

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
│   ├── tunnel_jupyter.log
│   └── tunnel_vscode.log
├── pids/
│   ├── jupyter.pid
│   ├── vscode.pid
│   └── tunnel_*.pid
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
- ⚠️ URLs change when you restart tunnels

To get new URLs:
```bash
cloudlab tunnel restart
cloudlab email send  # Send new URLs to your email
```

## 🔧 Troubleshooting

### "Jupyter not found"
```bash
cloudlab install jupyter
```

### "UV not found"
```bash
cloudlab install uv
```

### Port already in use
```bash
cloudlab config set jupyter_port 9999
cloudlab restart jupyter
```

### View logs
```bash
cloudlab logs jupyter
cloudlab logs vscode
cloudlab logs tunnel_jupyter
```

### Reset everything
```bash
cloudlab stop all
cloudlab config reset
cloudlab init
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

---

Made with ❤️ by [Sakib Dalal](https://github.com/Sakib-Dalal)
