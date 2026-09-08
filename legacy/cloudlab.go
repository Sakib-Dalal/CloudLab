// CloudLab CLI - Self-hosted Web Editor Setup Tool
// Author: Sakib Dalal
// GitHub: https://github.com/Sakib-Dalal
// Version: 1.2.0
// Features: Jupyter Lab/Notebook, VS Code Server, SSH Terminal, Web Dashboard
// All with Cloudflare Tunneling and Email Notifications

package main

import (
	"bufio"
	"crypto/rand"
	"crypto/tls"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/smtp"
	"os"
	"os/exec"
	"path/filepath"
	"regexp"
	"runtime"
	"strconv"
	"strings"
	"syscall"
	"time"
)

const (
	VERSION = "1.2.0"
	AUTHOR  = "Sakib Dalal"
	GITHUB  = "https://github.com/Sakib-Dalal"
)

// ANSI Colors
const (
	Reset         = "\033[0m"
	Bold          = "\033[1m"
	Dim           = "\033[2m"
	Underline     = "\033[4m"
	Red           = "\033[31m"
	Green         = "\033[32m"
	Yellow        = "\033[33m"
	Blue          = "\033[34m"
	Magenta       = "\033[35m"
	Cyan          = "\033[36m"
	White         = "\033[37m"
	BrightRed     = "\033[91m"
	BrightGreen   = "\033[92m"
	BrightYellow  = "\033[93m"
	BrightBlue    = "\033[94m"
	BrightMagenta = "\033[95m"
	BrightCyan    = "\033[96m"
	BrightWhite   = "\033[97m"
)

// Configuration
type Config struct {
	JupyterPort     int        `json:"jupyter_port"`
	VSCodePort      int        `json:"vscode_port"`
	SSHPort         int        `json:"ssh_port"`
	DashboardPort   int        `json:"dashboard_port"`
	PythonVersion   string     `json:"python_version"`
	JupyterPassword string     `json:"jupyter_password"`
	VSCodePassword  string     `json:"vscode_password"`
	SSHUser         string     `json:"ssh_user"`
	SSHPassword     string     `json:"ssh_password"`
	JupyterMode     string     `json:"jupyter_mode"`
	WorkDir         string     `json:"working_directory"`
	Email           string     `json:"email_address"`
	EmailPassword   string     `json:"email_app_password"`
	SMTPServer      string     `json:"smtp_server"`
	SMTPPort        int        `json:"smtp_port"`
	EnableMPS       bool       `json:"enable_mps"`
	EnableCUDA      bool       `json:"enable_cuda"`
	LowPowerMode    bool       `json:"low_power_mode"`
	NotifyOnStart   bool       `json:"notify_on_start"`
	TunnelURLs      TunnelURLs `json:"tunnel_urls"`
}

type TunnelURLs struct {
	Jupyter   string `json:"jupyter"`
	VSCode    string `json:"vscode"`
	SSH       string `json:"ssh"`
	Dashboard string `json:"dashboard"`
}

var (
	config      Config
	homeDir     string
	cloudlabDir string
	configPath  string
)

func main() {
	runtime.GOMAXPROCS(1)

	homeDir, _ = os.UserHomeDir()
	cloudlabDir = filepath.Join(homeDir, ".cloudlab")
	configPath = filepath.Join(cloudlabDir, "config.json")

	os.MkdirAll(cloudlabDir, 0755)
	os.MkdirAll(filepath.Join(cloudlabDir, "logs"), 0755)
	os.MkdirAll(filepath.Join(cloudlabDir, "pids"), 0755)
	os.MkdirAll(filepath.Join(cloudlabDir, "envs"), 0755)

	loadConfig()

	if len(os.Args) < 2 {
		showHelp()
		return
	}

	cmd := os.Args[1]
	args := os.Args[2:]

	switch cmd {
	case "init":
		initSetup()
	case "install":
		if len(args) > 0 {
			installComponent(args[0])
		} else {
			installAll()
		}
	case "start":
		if len(args) > 0 {
			startService(args[0])
		} else {
			startAll()
		}
	case "stop":
		if len(args) > 0 {
			stopService(args[0])
		} else {
			stopAll()
		}
	case "restart":
		if len(args) > 0 {
			stopService(args[0])
			time.Sleep(2 * time.Second)
			startService(args[0])
		} else {
			stopAll()
			time.Sleep(2 * time.Second)
			startAll()
		}
	case "status":
		showStatus()
	case "logs":
		if len(args) > 0 {
			showLogs(args[0])
		} else {
			fmt.Println("Usage: cloudlab logs <service>")
		}
	case "config":
		if len(args) > 0 {
			handleConfig(args)
		} else {
			showConfig()
		}
	case "tunnel":
		if len(args) > 0 {
			handleTunnel(args[0])
		} else {
			showTunnelStatus()
		}
	case "kernel":
		if len(args) > 0 {
			handleKernel(args)
		} else {
			listKernels()
		}
	case "env":
		if len(args) > 0 {
			handleEnv(args)
		} else {
			listEnvs()
		}
	case "email":
		if len(args) > 0 {
			handleEmail(args[0])
		} else {
			showEmailConfig()
		}
	case "ssh":
		if len(args) > 0 {
			handleSSH(args[0])
		} else {
			showSSHStatus()
		}
	case "dashboard":
		if len(args) > 0 {
			handleDashboard(args[0])
		} else {
			showDashboardStatus()
		}
	case "update":
		updateAll()
	case "uninstall":
		uninstallAll()
	case "help", "-h", "--help":
		showHelp()
	case "version", "-v", "--version":
		showVersion()
	default:
		printError("Unknown command: " + cmd)
		showHelp()
	}
}

func getLogo() string {
	return fmt.Sprintf(`
%s%s   _____ _                 _ _           _     %s
%s%s  / ____| |               | | |         | |    %s
%s%s | |    | | ___  _   _  __| | |     __ _| |__  %s
%s%s | |    | |/ _ \| | | |/ _' | |    / _' | '_ \ %s
%s%s | |____| | (_) | |_| | (_| | |___| (_| | |_) |%s
%s%s  \_____|_|\___/ \__,_|\__,_|______\__,_|_.__/ %s

%s  ☁️  Self-Hosted Web Editor CLI %sv%s%s
%s  👤 Author: %s%s%s
%s  🔗 GitHub: %s%s%s
`,
		Bold, BrightCyan, Reset,
		Bold, BrightCyan, Reset,
		Bold, BrightBlue, Reset,
		Bold, BrightBlue, Reset,
		Bold, BrightMagenta, Reset,
		Bold, BrightMagenta, Reset,
		BrightWhite, Bold, VERSION, Reset,
		BrightYellow, Bold, AUTHOR, Reset,
		BrightBlue, Underline, GITHUB, Reset)
}

func showVersion() {
	fmt.Printf("%s☁️  CloudLab CLI v%s%s\n", BrightCyan, VERSION, Reset)
	fmt.Printf("%sAuthor: %s%s\n", Dim, AUTHOR, Reset)
	fmt.Printf("%sGitHub: %s%s\n", Dim, GITHUB, Reset)
}

func showHelp() {
	fmt.Println(getLogo())
	fmt.Printf(`
%sUSAGE:%s
  cloudlab <command> [options]

%sSERVICES:%s
  init                    Initialize CloudLab
  install [component]     Install (all|jupyter|vscode|ssh|dashboard|cloudflare|uv)
  start [service]         Start (all|jupyter|lab|notebook|vscode|ssh|dashboard|tunnel)
  stop [service]          Stop services
  restart [service]       Restart services
  status                  Show all status

%sTUNNELS:%s
  tunnel start            Start all Cloudflare tunnels
  tunnel stop             Stop all tunnels
  tunnel restart          Get new URLs
  tunnel status           Show tunnel URLs

%sSSH TERMINAL:%s
  ssh start               Start web SSH terminal
  ssh stop                Stop SSH terminal
  ssh config              Configure SSH settings
  ssh status              Show SSH status

%sDASHBOARD:%s
  dashboard start         Start web dashboard
  dashboard stop          Stop dashboard
  dashboard status        Show dashboard status

%sKERNELS:%s
  kernel list             List Jupyter kernels
  kernel add <name> [ver] Add kernel with Python version
  kernel remove <name>    Remove kernel

%sENVIRONMENTS:%s
  env list                List Python environments
  env create <name> <ver> Create new environment
  env remove <name>       Remove environment
  env install <pkg>       Install package

%sEMAIL:%s
  email setup             Setup email notifications
  email test              Send test email
  email send              Send all tunnel URLs

%sCONFIG:%s
  config                  Show configuration
  config set <key> <val>  Set config value
  config reset            Reset to defaults

%sOTHER:%s
  update                  Update components
  uninstall               Uninstall CloudLab
  help                    Show this help
  version                 Show version

%sEXAMPLES:%s
  cloudlab init
  cloudlab install all
  cloudlab start all
  cloudlab tunnel start
  cloudlab email send
  cloudlab kernel add mykernel 3.10
`, Bold, Reset, Bold, Reset, Bold, Reset, Bold, Reset, Bold, Reset, Bold, Reset, Bold, Reset, Bold, Reset, Bold, Reset, Bold, Reset, Bold, Reset)
}

// ==================== Config ====================

func loadConfig() {
	config = Config{
		JupyterPort:   8888,
		VSCodePort:    8080,
		SSHPort:       7681,
		DashboardPort: 3000,
		PythonVersion: "3.11",
		JupyterMode:   "lab",
		WorkDir:       homeDir,
		SMTPPort:      587,
		LowPowerMode:  true,
		NotifyOnStart: true,
	}

	if u := os.Getenv("USER"); u != "" {
		config.SSHUser = u
	} else if u := os.Getenv("USERNAME"); u != "" {
		config.SSHUser = u
	} else {
		config.SSHUser = "user"
	}

	if runtime.GOOS == "darwin" && runtime.GOARCH == "arm64" {
		config.EnableMPS = true
	}
	if _, err := exec.LookPath("nvidia-smi"); err == nil {
		config.EnableCUDA = true
	}

	if data, err := os.ReadFile(configPath); err == nil {
		json.Unmarshal(data, &config)
	}
}

func saveConfig() {
	data, _ := json.MarshalIndent(config, "", "  ")
	os.WriteFile(configPath, data, 0600)
}

func showConfig() {
	fmt.Println(getLogo())
	printHeader("📋 CONFIGURATION")
	fmt.Printf("  %-20s : %s%d%s\n", "jupyter_port", BrightCyan, config.JupyterPort, Reset)
	fmt.Printf("  %-20s : %s%d%s\n", "vscode_port", BrightCyan, config.VSCodePort, Reset)
	fmt.Printf("  %-20s : %s%d%s\n", "ssh_port", BrightCyan, config.SSHPort, Reset)
	fmt.Printf("  %-20s : %s%d%s\n", "dashboard_port", BrightCyan, config.DashboardPort, Reset)
	fmt.Printf("  %-20s : %s%s%s\n", "jupyter_mode", BrightGreen, config.JupyterMode, Reset)
	fmt.Printf("  %-20s : %s%s%s\n", "python_version", BrightYellow, config.PythonVersion, Reset)
	fmt.Printf("  %-20s : %s%s%s\n", "working_directory", BrightBlue, config.WorkDir, Reset)
	fmt.Printf("  %-20s : %s%s%s\n", "ssh_user", BrightMagenta, config.SSHUser, Reset)
	if config.Email != "" {
		fmt.Printf("  %-20s : %s%s%s\n", "email", BrightMagenta, config.Email, Reset)
	}
	fmt.Printf("  %-20s : %s%v%s\n", "enable_mps", boolColor(config.EnableMPS), config.EnableMPS, Reset)
	fmt.Printf("  %-20s : %s%v%s\n", "enable_cuda", boolColor(config.EnableCUDA), config.EnableCUDA, Reset)
	fmt.Println()
}

func handleConfig(args []string) {
	if args[0] == "reset" {
		os.Remove(configPath)
		loadConfig()
		saveConfig()
		printSuccess("Configuration reset!")
		return
	}
	if args[0] == "set" && len(args) >= 3 {
		key, val := args[1], strings.Join(args[2:], " ")
		switch key {
		case "jupyter_port":
			config.JupyterPort, _ = strconv.Atoi(val)
		case "vscode_port":
			config.VSCodePort, _ = strconv.Atoi(val)
		case "ssh_port":
			config.SSHPort, _ = strconv.Atoi(val)
		case "dashboard_port":
			config.DashboardPort, _ = strconv.Atoi(val)
		case "jupyter_mode":
			config.JupyterMode = val
		case "python_version":
			config.PythonVersion = val
		case "working_directory":
			config.WorkDir = val
		case "jupyter_password":
			config.JupyterPassword = val
		case "vscode_password":
			config.VSCodePassword = val
		case "ssh_user":
			config.SSHUser = val
		case "ssh_password":
			config.SSHPassword = val
		case "email_address":
			config.Email = val
		case "email_app_password":
			config.EmailPassword = val
		case "smtp_server":
			config.SMTPServer = val
		case "notify_on_start":
			config.NotifyOnStart = val == "true"
		default:
			printError("Unknown key: " + key)
			return
		}
		saveConfig()
		printSuccess(fmt.Sprintf("Set %s = %s", key, val))
	}
}

func boolColor(b bool) string {
	if b {
		return BrightGreen
	}
	return BrightRed
}

// ==================== Init ====================

func initSetup() {
	fmt.Println(getLogo())
	printHeader("🚀 INITIALIZATION")
	reader := bufio.NewReader(os.Stdin)

	// Working directory
	fmt.Printf("\n%s[1/9]%s Working directory [%s]: ", BrightCyan, Reset, config.WorkDir)
	if input := readLine(reader); input != "" {
		os.MkdirAll(input, 0755)
		config.WorkDir = input
	}

	// Jupyter mode
	fmt.Printf("%s[2/9]%s Jupyter mode (lab/notebook) [%s]: ", BrightCyan, Reset, config.JupyterMode)
	if input := readLine(reader); input == "lab" || input == "notebook" {
		config.JupyterMode = input
	}

	// Ports
	fmt.Printf("%s[3/9]%s Jupyter port [%d]: ", BrightCyan, Reset, config.JupyterPort)
	if input := readLine(reader); input != "" {
		config.JupyterPort, _ = strconv.Atoi(input)
	}

	fmt.Printf("%s[4/9]%s VS Code port [%d]: ", BrightCyan, Reset, config.VSCodePort)
	if input := readLine(reader); input != "" {
		config.VSCodePort, _ = strconv.Atoi(input)
	}

	fmt.Printf("%s[5/9]%s SSH Terminal port [%d]: ", BrightCyan, Reset, config.SSHPort)
	if input := readLine(reader); input != "" {
		config.SSHPort, _ = strconv.Atoi(input)
	}

	fmt.Printf("%s[6/9]%s Dashboard port [%d]: ", BrightCyan, Reset, config.DashboardPort)
	if input := readLine(reader); input != "" {
		config.DashboardPort, _ = strconv.Atoi(input)
	}

	// Passwords
	fmt.Printf("%s[7/9]%s Jupyter password (Enter=auto): ", BrightCyan, Reset)
	if input := readLine(reader); input != "" {
		config.JupyterPassword = input
	} else {
		config.JupyterPassword = genToken(16)
		fmt.Printf("       Generated: %s%s%s\n", BrightGreen, config.JupyterPassword, Reset)
	}

	fmt.Printf("%s[8/9]%s VS Code password (Enter=auto): ", BrightCyan, Reset)
	if input := readLine(reader); input != "" {
		config.VSCodePassword = input
	} else {
		config.VSCodePassword = genToken(16)
		fmt.Printf("       Generated: %s%s%s\n", BrightGreen, config.VSCodePassword, Reset)
	}

	// Email
	fmt.Printf("%s[9/9]%s Email for notifications (optional): ", BrightCyan, Reset)
	if input := readLine(reader); input != "" {
		config.Email = input
		detectSMTP(input)
		fmt.Printf("       App password: ")
		config.EmailPassword = readLine(reader)
	}

	// Hardware
	printHeader("🔧 HARDWARE")
	if config.EnableMPS {
		printSuccess("Apple Silicon (MPS) detected")
	}
	if config.EnableCUDA {
		printSuccess("NVIDIA GPU (CUDA) detected")
	}
	if !config.EnableMPS && !config.EnableCUDA {
		printInfo("CPU mode")
	}

	saveConfig()
	printSuccess("Configuration saved!")

	fmt.Printf("\n%sInstall components now?%s [Y/n]: ", Bold, Reset)
	if ans := strings.ToLower(readLine(reader)); ans == "" || ans == "y" {
		installAll()
	}
}

func detectSMTP(email string) {
	email = strings.ToLower(email)
	if strings.Contains(email, "gmail") {
		config.SMTPServer = "smtp.gmail.com"
		printInfo("Gmail detected")
	} else if strings.Contains(email, "outlook") || strings.Contains(email, "hotmail") {
		config.SMTPServer = "smtp-mail.outlook.com"
		printInfo("Outlook detected")
	} else if strings.Contains(email, "yahoo") {
		config.SMTPServer = "smtp.mail.yahoo.com"
		printInfo("Yahoo detected")
	}
}

func readLine(r *bufio.Reader) string {
	s, _ := r.ReadString('\n')
	return strings.TrimSpace(s)
}

// ==================== Install ====================

func installAll() {
	printHeader("📦 INSTALLING")
	installUV()
	installJupyter()
	installVSCode()
	installTTYD()
	installCloudflared()
	createDashboardFiles()
	printSuccess("All components installed!")
}

func installComponent(c string) {
	switch c {
	case "all":
		installAll()
	case "uv":
		installUV()
	case "jupyter":
		installJupyter()
	case "vscode":
		installVSCode()
	case "ssh", "ttyd":
		installTTYD()
	case "cloudflare", "cloudflared":
		installCloudflared()
	case "dashboard":
		createDashboardFiles()
	default:
		printError("Unknown: " + c)
	}
}

func installUV() {
	printStep("Installing UV...")
	if _, err := exec.LookPath("uv"); err == nil {
		printSuccess("UV already installed")
		return
	}
	var cmd *exec.Cmd
	if runtime.GOOS == "windows" {
		cmd = exec.Command("powershell", "-c", "irm https://astral.sh/uv/install.ps1 | iex")
	} else {
		cmd = exec.Command("bash", "-c", "curl -LsSf https://astral.sh/uv/install.sh | sh")
	}
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	cmd.Run()
	printSuccess("UV installed")
}

func getUVPath() string {
	paths := []string{
		filepath.Join(homeDir, ".cargo", "bin", "uv"),
		filepath.Join(homeDir, ".local", "bin", "uv"),
		"/usr/local/bin/uv",
	}
	for _, p := range paths {
		if _, err := os.Stat(p); err == nil {
			return p
		}
	}
	if p, err := exec.LookPath("uv"); err == nil {
		return p
	}
	return ""
}

func getPythonPath() string {
	venv := filepath.Join(cloudlabDir, "venv")
	if runtime.GOOS == "windows" {
		return filepath.Join(venv, "Scripts", "python.exe")
	}
	return filepath.Join(venv, "bin", "python")
}

func getJupyterPath() string {
	venv := filepath.Join(cloudlabDir, "venv")
	if runtime.GOOS == "windows" {
		return filepath.Join(venv, "Scripts", "jupyter.exe")
	}
	return filepath.Join(venv, "bin", "jupyter")
}

func installJupyter() {
	printStep("Installing Jupyter...")
	uv := getUVPath()
	if uv == "" {
		installUV()
		uv = getUVPath()
	}
	if uv == "" {
		printError("UV not found")
		return
	}

	venv := filepath.Join(cloudlabDir, "venv")
	exec.Command(uv, "venv", venv, "--python", config.PythonVersion).Run()

	py := getPythonPath()
	pkgs := []string{"jupyterlab", "notebook", "ipykernel", "ipywidgets"}
	for _, pkg := range pkgs {
		exec.Command(uv, "pip", "install", pkg, "--python", py).Run()
	}

	// PyTorch
	if config.EnableMPS {
		exec.Command(uv, "pip", "install", "torch", "torchvision", "--python", py).Run()
	} else if config.EnableCUDA {
		exec.Command(uv, "pip", "install", "torch", "torchvision", "--index-url", "https://download.pytorch.org/whl/cu121", "--python", py).Run()
	}

	// Register kernel
	exec.Command(py, "-m", "ipykernel", "install", "--user", "--name", "cloudlab", "--display-name", "Python "+config.PythonVersion+" (CloudLab)").Run()

	configureJupyter()
	printSuccess("Jupyter installed")
}

func configureJupyter() {
	jupyterDir := filepath.Join(homeDir, ".jupyter")
	os.MkdirAll(jupyterDir, 0755)

	py := getPythonPath()
	hashCmd := fmt.Sprintf(`from jupyter_server.auth import passwd; print(passwd('%s'))`, config.JupyterPassword)
	out, _ := exec.Command(py, "-c", hashCmd).Output()
	hash := strings.TrimSpace(string(out))
	if hash == "" {
		hash = "''"
	}

	cfg := fmt.Sprintf(`c = get_config()
c.ServerApp.ip = '0.0.0.0'
c.ServerApp.port = %d
c.ServerApp.open_browser = False
c.ServerApp.allow_root = True
c.ServerApp.allow_origin = '*'
c.ServerApp.root_dir = '%s'
c.ServerApp.password = '%s'
c.ServerApp.token = ''
c.NotebookApp.ip = '0.0.0.0'
c.NotebookApp.port = %d
c.NotebookApp.open_browser = False
c.NotebookApp.allow_root = True
c.NotebookApp.notebook_dir = '%s'
c.NotebookApp.password = '%s'
c.NotebookApp.token = ''
`, config.JupyterPort, config.WorkDir, hash, config.JupyterPort, config.WorkDir, hash)

	os.WriteFile(filepath.Join(jupyterDir, "jupyter_lab_config.py"), []byte(cfg), 0644)
	os.WriteFile(filepath.Join(jupyterDir, "jupyter_server_config.py"), []byte(cfg), 0644)
}

func installVSCode() {
	printStep("Installing VS Code Server...")
	if _, err := exec.LookPath("code-server"); err == nil {
		printSuccess("code-server already installed")
		configureVSCode()
		return
	}
	cmd := exec.Command("bash", "-c", "curl -fsSL https://code-server.dev/install.sh | sh")
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	cmd.Run()
	configureVSCode()
	printSuccess("VS Code installed")
}

func configureVSCode() {
	cfgDir := filepath.Join(homeDir, ".config", "code-server")
	os.MkdirAll(cfgDir, 0755)
	cfg := fmt.Sprintf(`bind-addr: 0.0.0.0:%d
auth: password
password: %s
cert: false
`, config.VSCodePort, config.VSCodePassword)
	os.WriteFile(filepath.Join(cfgDir, "config.yaml"), []byte(cfg), 0644)
}

func installTTYD() {
	printStep("Installing SSH Terminal (ttyd)...")
	if _, err := exec.LookPath("ttyd"); err == nil {
		printSuccess("ttyd already installed")
		return
	}

	switch runtime.GOOS {
	case "darwin":
		exec.Command("brew", "install", "ttyd").Run()
	case "linux":
		// Try apt first
		if _, err := exec.LookPath("apt-get"); err == nil {
			exec.Command("sudo", "apt-get", "update").Run()
			exec.Command("sudo", "apt-get", "install", "-y", "ttyd").Run()
		} else {
			// Download binary
			url := "https://github.com/tsl0922/ttyd/releases/latest/download/ttyd.x86_64"
			if runtime.GOARCH == "arm64" {
				url = "https://github.com/tsl0922/ttyd/releases/latest/download/ttyd.aarch64"
			}
			downloadFile("/tmp/ttyd", url)
			os.Chmod("/tmp/ttyd", 0755)
			exec.Command("sudo", "mv", "/tmp/ttyd", "/usr/local/bin/ttyd").Run()
		}
	}
	printSuccess("ttyd installed")
}

func installCloudflared() {
	printStep("Installing Cloudflared...")
	if _, err := exec.LookPath("cloudflared"); err == nil {
		printSuccess("cloudflared already installed")
		return
	}

	switch runtime.GOOS {
	case "darwin":
		exec.Command("brew", "install", "cloudflared").Run()
	case "linux":
		url := "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64"
		if runtime.GOARCH == "arm64" {
			url = "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-arm64"
		}
		downloadFile("/tmp/cloudflared", url)
		os.Chmod("/tmp/cloudflared", 0755)
		exec.Command("sudo", "mv", "/tmp/cloudflared", "/usr/local/bin/cloudflared").Run()
	}
	printSuccess("cloudflared installed")
}

func createDashboardFiles() {
	printStep("Creating dashboard files...")

	// Create server.py
	serverPy := `#!/usr/bin/env python3
"""CloudLab Dashboard Server"""
import http.server
import json
import subprocess
import os
import socketserver
import urllib.parse
import socket
import psutil

PORT = int(os.environ.get('CLOUDLAB_PORT', 3000))
DIR = os.path.expanduser('~/.cloudlab')

def check_port(port):
    """Check if a port is in use"""
    try:
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
            s.settimeout(1)
            result = s.connect_ex(('127.0.0.1', port))
            return result == 0
    except:
        return False

def check_process(name):
    """Check if process is running"""
    pid_file = os.path.join(DIR, 'pids', f'{name}.pid')
    try:
        if not os.path.exists(pid_file):
            return False
        with open(pid_file, 'r') as f:
            pid = int(f.read().strip())
        # Check if process exists
        try:
            os.kill(pid, 0)
            return True
        except OSError:
            return False
    except:
        return False

def get_config():
    try:
        with open(os.path.join(DIR, 'config.json'), 'r') as f:
            return json.load(f)
    except:
        return {}

def get_system_info():
    try:
        import psutil
        return {
            'cpu_percent': psutil.cpu_percent(interval=0.1),
            'memory_percent': psutil.virtual_memory().percent,
            'disk_percent': psutil.disk_usage('/').percent,
            'cpu_count': psutil.cpu_count(),
            'memory_total': round(psutil.virtual_memory().total / (1024**3), 1),
            'disk_total': round(psutil.disk_usage('/').total / (1024**3), 1)
        }
    except:
        return {}

class Handler(http.server.BaseHTTPRequestHandler):
    def log_message(self, *args): pass
    
    def send_json(self, data, code=200):
        body = json.dumps(data).encode()
        self.send_response(code)
        self.send_header('Content-Type', 'application/json')
        self.send_header('Content-Length', len(body))
        self.send_header('Access-Control-Allow-Origin', '*')
        self.end_headers()
        self.wfile.write(body)
    
    def do_OPTIONS(self):
        self.send_response(200)
        self.send_header('Access-Control-Allow-Origin', '*')
        self.send_header('Access-Control-Allow-Methods', 'GET, POST, OPTIONS')
        self.send_header('Access-Control-Allow-Headers', 'Content-Type')
        self.end_headers()
    
    def do_GET(self):
        path = urllib.parse.urlparse(self.path).path
        
        if path in ['/', '/index.html']:
            html_path = os.path.join(DIR, 'dashboard.html')
            if os.path.exists(html_path):
                with open(html_path, 'rb') as f:
                    content = f.read()
                self.send_response(200)
                self.send_header('Content-Type', 'text/html')
                self.send_header('Content-Length', len(content))
                self.end_headers()
                self.wfile.write(content)
            else:
                self.send_json({'error': 'Dashboard not found'}, 404)
            return
        
        if path == '/api/status':
            cfg = get_config()
            # Check both PID and port for more accurate status
            jupyter_running = check_process('jupyter') or check_port(cfg.get('jupyter_port', 8888))
            vscode_running = check_process('vscode') or check_port(cfg.get('vscode_port', 8080))
            ssh_running = check_process('ssh') or check_port(cfg.get('ssh_port', 7681))
            
            self.send_json({
                'jupyter': jupyter_running,
                'vscode': vscode_running,
                'ssh': ssh_running,
                'dashboard': True,
                'tunnel_jupyter': check_process('tunnel_jupyter'),
                'tunnel_vscode': check_process('tunnel_vscode'),
                'tunnel_ssh': check_process('tunnel_ssh'),
                'tunnel_dashboard': check_process('tunnel_dashboard'),
                'config': cfg,
                'system': get_system_info()
            })
            return
        
        if path == '/api/logs':
            query = urllib.parse.parse_qs(urllib.parse.urlparse(self.path).query)
            service = query.get('service', ['jupyter'])[0]
            lines = int(query.get('lines', [100])[0])
            log_path = os.path.join(DIR, 'logs', f'{service}.log')
            try:
                with open(log_path, 'r') as f:
                    content = f.readlines()[-lines:]
                self.send_json({'log': ''.join(content)})
            except:
                self.send_json({'log': 'No logs available'})
            return
        
        if path.startswith('/api/command/'):
            parts = path[13:].split('/')
            parts = [urllib.parse.unquote(p) for p in parts if p]
            if parts:
                try:
                    result = subprocess.run(['cloudlab'] + parts, capture_output=True, text=True, timeout=120)
                    self.send_json({'success': result.returncode == 0, 'stdout': result.stdout, 'stderr': result.stderr})
                except Exception as e:
                    self.send_json({'success': False, 'error': str(e)})
            else:
                self.send_json({'error': 'No command'}, 400)
            return
        
        self.send_json({'error': 'Not found'}, 404)

class Server(socketserver.TCPServer):
    allow_reuse_address = True

if __name__ == '__main__':
    # Install psutil if not available
    try:
        import psutil
    except ImportError:
        os.system('pip install psutil')
        import psutil
    
    print(f'Dashboard: http://localhost:{PORT}')
    with Server(('0.0.0.0', PORT), Handler) as server:
        server.serve_forever()
`
	os.WriteFile(filepath.Join(cloudlabDir, "server.py"), []byte(serverPy), 0755)

	// Copy index.html to dashboard.html
	if _, err := os.Stat("index.html"); err == nil {
		data, _ := os.ReadFile("index.html")
		os.WriteFile(filepath.Join(cloudlabDir, "dashboard.html"), data, 0644)
	}

	printSuccess("Dashboard files created")
}

// ==================== Start/Stop ====================

func startService(s string) {
	switch s {
	case "all":
		startAll()
	case "jupyter", "lab":
		startJupyter("lab")
	case "notebook":
		startJupyter("notebook")
	case "vscode":
		startVSCode()
	case "ssh":
		startSSH()
	case "dashboard":
		startDashboard()
	case "tunnel", "tunnels":
		startAllTunnels()
	default:
		printError("Unknown: " + s)
	}
}

func startAll() {
	printHeader("🚀 STARTING ALL SERVICES")
	startJupyter(config.JupyterMode)
	startVSCode()
	startSSH()
	startDashboard()
	time.Sleep(2 * time.Second)
	startAllTunnels()
	printSuccess("All services started!")
}

func startJupyter(mode string) {
	printStep("Starting Jupyter " + mode + "...")
	jp := getJupyterPath()
	if _, err := os.Stat(jp); err != nil {
		printError("Jupyter not found. Run: cloudlab install jupyter")
		return
	}

	stopPID("jupyter")
	time.Sleep(500 * time.Millisecond)

	var cmd *exec.Cmd
	if mode == "lab" {
		cmd = exec.Command(jp, "lab", "--no-browser", "--ip=0.0.0.0",
			fmt.Sprintf("--port=%d", config.JupyterPort),
			fmt.Sprintf("--notebook-dir=%s", config.WorkDir),
			"--ServerApp.token=''", "--ServerApp.allow_origin='*'")
	} else {
		cmd = exec.Command(jp, "notebook", "--no-browser", "--ip=0.0.0.0",
			fmt.Sprintf("--port=%d", config.JupyterPort),
			fmt.Sprintf("--notebook-dir=%s", config.WorkDir),
			"--NotebookApp.token=''", "--NotebookApp.allow_origin='*'")
	}
	cmd.Dir = config.WorkDir

	logFile, _ := os.Create(filepath.Join(cloudlabDir, "logs", "jupyter.log"))
	cmd.Stdout = logFile
	cmd.Stderr = logFile

	if err := cmd.Start(); err != nil {
		printError("Failed: " + err.Error())
		return
	}
	savePID("jupyter", cmd.Process.Pid)
	fmt.Printf("  %s✓%s Jupyter %s on port %s%d%s\n", BrightGreen, Reset, mode, BrightCyan, config.JupyterPort, Reset)
}

func startVSCode() {
	printStep("Starting VS Code...")
	cs, err := exec.LookPath("code-server")
	if err != nil {
		printError("code-server not found. Run: cloudlab install vscode")
		return
	}

	stopPID("vscode")
	time.Sleep(500 * time.Millisecond)

	cmd := exec.Command(cs, fmt.Sprintf("--bind-addr=0.0.0.0:%d", config.VSCodePort), config.WorkDir)
	cmd.Dir = config.WorkDir

	logFile, _ := os.Create(filepath.Join(cloudlabDir, "logs", "vscode.log"))
	cmd.Stdout = logFile
	cmd.Stderr = logFile

	if err := cmd.Start(); err != nil {
		printError("Failed: " + err.Error())
		return
	}
	savePID("vscode", cmd.Process.Pid)
	fmt.Printf("  %s✓%s VS Code on port %s%d%s\n", BrightGreen, Reset, BrightCyan, config.VSCodePort, Reset)
}

func startSSH() {
	printStep("Starting SSH Terminal...")
	ttyd, err := exec.LookPath("ttyd")
	if err != nil {
		printError("ttyd not found. Run: cloudlab install ssh")
		return
	}

	stopPID("ssh")
	time.Sleep(500 * time.Millisecond)

	args := []string{"--port", strconv.Itoa(config.SSHPort), "--writable"}
	if config.SSHPassword != "" {
		args = append(args, "--credential", fmt.Sprintf("%s:%s", config.SSHUser, config.SSHPassword))
	}

	shell := "bash"
	if runtime.GOOS == "windows" {
		shell = "cmd.exe"
	}
	args = append(args, shell, "-l")

	cmd := exec.Command(ttyd, args...)
	cmd.Dir = config.WorkDir

	logFile, _ := os.Create(filepath.Join(cloudlabDir, "logs", "ssh.log"))
	cmd.Stdout = logFile
	cmd.Stderr = logFile

	if err := cmd.Start(); err != nil {
		printError("Failed: " + err.Error())
		return
	}
	savePID("ssh", cmd.Process.Pid)
	fmt.Printf("  %s✓%s SSH Terminal on port %s%d%s\n", BrightGreen, Reset, BrightCyan, config.SSHPort, Reset)
}

func startDashboard() {
	printStep("Starting Dashboard...")

	py := getPythonPath()
	if _, err := os.Stat(py); err != nil {
		py = "python3"
		if _, err := exec.LookPath(py); err != nil {
			py = "python"
		}
	}

	stopPID("dashboard")
	time.Sleep(500 * time.Millisecond)

	// Copy latest dashboard.html
	if _, err := os.Stat("index.html"); err == nil {
		data, _ := os.ReadFile("index.html")
		os.WriteFile(filepath.Join(cloudlabDir, "dashboard.html"), data, 0644)
	}

	serverPath := filepath.Join(cloudlabDir, "server.py")
	if _, err := os.Stat(serverPath); err != nil {
		createDashboardFiles()
	}

	cmd := exec.Command(py, serverPath)
	cmd.Dir = cloudlabDir
	cmd.Env = append(os.Environ(), fmt.Sprintf("CLOUDLAB_PORT=%d", config.DashboardPort))

	logFile, _ := os.Create(filepath.Join(cloudlabDir, "logs", "dashboard.log"))
	cmd.Stdout = logFile
	cmd.Stderr = logFile

	if err := cmd.Start(); err != nil {
		printError("Failed: " + err.Error())
		return
	}
	savePID("dashboard", cmd.Process.Pid)
	fmt.Printf("  %s✓%s Dashboard on port %s%d%s\n", BrightGreen, Reset, BrightCyan, config.DashboardPort, Reset)
}

func stopService(s string) {
	switch s {
	case "all":
		stopAll()
	case "jupyter", "lab", "notebook":
		stopPID("jupyter")
		printSuccess("Jupyter stopped")
	case "vscode":
		stopPID("vscode")
		printSuccess("VS Code stopped")
	case "ssh":
		stopPID("ssh")
		printSuccess("SSH stopped")
	case "dashboard":
		stopPID("dashboard")
		printSuccess("Dashboard stopped")
	case "tunnel", "tunnels":
		stopAllTunnels()
	default:
		printError("Unknown: " + s)
	}
}

func stopAll() {
	printHeader("🛑 STOPPING ALL")
	stopAllTunnels()
	stopPID("jupyter")
	stopPID("vscode")
	stopPID("ssh")
	stopPID("dashboard")
	printSuccess("All stopped")
}

// ==================== Tunnels ====================

func handleTunnel(action string) {
	switch action {
	case "start":
		startAllTunnels()
	case "stop":
		stopAllTunnels()
	case "restart":
		stopAllTunnels()
		time.Sleep(2 * time.Second)
		startAllTunnels()
	case "status":
		showTunnelStatus()
	default:
		printError("Unknown: " + action)
	}
}

func startAllTunnels() {
	printStep("Starting Cloudflare tunnels...")

	cf, err := exec.LookPath("cloudflared")
	if err != nil {
		printError("cloudflared not found. Run: cloudlab install cloudflare")
		return
	}

	// Stop existing
	stopPID("tunnel_jupyter")
	stopPID("tunnel_vscode")
	stopPID("tunnel_ssh")
	stopPID("tunnel_dashboard")
	time.Sleep(1 * time.Second)

	// Start tunnels
	services := []struct {
		name string
		port int
	}{
		{"jupyter", config.JupyterPort},
		{"vscode", config.VSCodePort},
		{"ssh", config.SSHPort},
		{"dashboard", config.DashboardPort},
	}

	for _, svc := range services {
		if !isRunning(svc.name) && svc.name != "dashboard" {
			continue
		}
		go func(name string, port int) {
			logPath := filepath.Join(cloudlabDir, "logs", "tunnel_"+name+".log")
			logFile, _ := os.Create(logPath)
			cmd := exec.Command(cf, "tunnel", "--url", fmt.Sprintf("http://localhost:%d", port))
			cmd.Stdout = logFile
			cmd.Stderr = logFile
			if err := cmd.Start(); err == nil && cmd.Process != nil {
				savePID("tunnel_"+name, cmd.Process.Pid)
			}
			time.Sleep(8 * time.Second)
			extractURL(name, logPath)
		}(svc.name, svc.port)
	}

	fmt.Printf("  %s⏳%s Waiting for tunnel URLs...\n", BrightYellow, Reset)
	time.Sleep(15 * time.Second)

	loadConfig()
	showTunnelStatus()

	if config.NotifyOnStart && config.Email != "" && config.EmailPassword != "" {
		sendTunnelEmail()
	}
}

func extractURL(name, logPath string) {
	for i := 0; i < 30; i++ {
		data, err := os.ReadFile(logPath)
		if err == nil {
			re := regexp.MustCompile(`https://[a-zA-Z0-9-]+\.trycloudflare\.com`)
			matches := re.FindAllString(string(data), -1)
			if len(matches) > 0 {
				url := matches[len(matches)-1]
				switch name {
				case "jupyter":
					config.TunnelURLs.Jupyter = url
				case "vscode":
					config.TunnelURLs.VSCode = url
				case "ssh":
					config.TunnelURLs.SSH = url
				case "dashboard":
					config.TunnelURLs.Dashboard = url
				}
				saveConfig()
				return
			}
		}
		time.Sleep(1 * time.Second)
	}
}

func stopAllTunnels() {
	stopPID("tunnel_jupyter")
	stopPID("tunnel_vscode")
	stopPID("tunnel_ssh")
	stopPID("tunnel_dashboard")
	config.TunnelURLs = TunnelURLs{}
	saveConfig()
	printSuccess("Tunnels stopped")
}

func showTunnelStatus() {
	loadConfig()
	printHeader("🌐 TUNNEL URLS")

	printTunnelLine("🐍 Jupyter", config.TunnelURLs.Jupyter, isRunning("tunnel_jupyter"))
	printTunnelLine("💻 VS Code", config.TunnelURLs.VSCode, isRunning("tunnel_vscode"))
	printTunnelLine("🔒 SSH", config.TunnelURLs.SSH, isRunning("tunnel_ssh"))
	printTunnelLine("📊 Dashboard", config.TunnelURLs.Dashboard, isRunning("tunnel_dashboard"))
	fmt.Println()
}

func printTunnelLine(name, url string, running bool) {
	status := fmt.Sprintf("%s[Stopped]%s", BrightRed, Reset)
	if running {
		status = fmt.Sprintf("%s[Running]%s", BrightGreen, Reset)
	}
	if url != "" {
		fmt.Printf("  %-12s %s\n", name, status)
		fmt.Printf("    └─ %s%s%s\n", BrightMagenta+Underline, url, Reset)
	} else {
		fmt.Printf("  %-12s %s %s(no tunnel)%s\n", name, status, Dim, Reset)
	}
}

// ==================== SSH ====================

func handleSSH(action string) {
	switch action {
	case "start":
		startSSH()
	case "stop":
		stopPID("ssh")
		printSuccess("SSH stopped")
	case "config":
		configureSSH()
	case "status":
		showSSHStatus()
	default:
		printError("Unknown: " + action)
	}
}

func configureSSH() {
	printHeader("🔒 SSH CONFIG")
	reader := bufio.NewReader(os.Stdin)

	fmt.Printf("  SSH port [%d]: ", config.SSHPort)
	if input := readLine(reader); input != "" {
		config.SSHPort, _ = strconv.Atoi(input)
	}

	fmt.Printf("  SSH username [%s]: ", config.SSHUser)
	if input := readLine(reader); input != "" {
		config.SSHUser = input
	}

	fmt.Printf("  SSH password (optional): ")
	if input := readLine(reader); input != "" {
		config.SSHPassword = input
	}

	saveConfig()
	printSuccess("SSH configured")
}

func showSSHStatus() {
	printHeader("🔒 SSH STATUS")
	if isRunning("ssh") {
		fmt.Printf("  %s●%s SSH Terminal %s[Running]%s port %s%d%s\n", BrightGreen, Reset, BrightGreen, Reset, BrightCyan, config.SSHPort, Reset)
		fmt.Printf("    └─ http://localhost:%d\n", config.SSHPort)
		if config.TunnelURLs.SSH != "" {
			fmt.Printf("    └─ %s%s%s\n", BrightMagenta, config.TunnelURLs.SSH, Reset)
		}
	} else {
		fmt.Printf("  %s○%s SSH Terminal %s[Stopped]%s\n", BrightRed, Reset, BrightRed, Reset)
	}
	fmt.Println()
}

// ==================== Dashboard ====================

func handleDashboard(action string) {
	switch action {
	case "start":
		startDashboard()
	case "stop":
		stopPID("dashboard")
		printSuccess("Dashboard stopped")
	case "status":
		showDashboardStatus()
	default:
		printError("Unknown: " + action)
	}
}

func showDashboardStatus() {
	printHeader("📊 DASHBOARD STATUS")
	if isRunning("dashboard") {
		fmt.Printf("  %s●%s Dashboard %s[Running]%s port %s%d%s\n", BrightGreen, Reset, BrightGreen, Reset, BrightCyan, config.DashboardPort, Reset)
		fmt.Printf("    └─ http://localhost:%d\n", config.DashboardPort)
		if config.TunnelURLs.Dashboard != "" {
			fmt.Printf("    └─ %s%s%s\n", BrightMagenta, config.TunnelURLs.Dashboard, Reset)
		}
	} else {
		fmt.Printf("  %s○%s Dashboard %s[Stopped]%s\n", BrightRed, Reset, BrightRed, Reset)
	}
	fmt.Println()
}

// ==================== Status ====================

func showStatus() {
	fmt.Println(getLogo())
	printHeader("📊 SERVICE STATUS")

	// Jupyter
	if isRunning("jupyter") {
		fmt.Printf("  %s●%s Jupyter %s %s[Running]%s port %s%d%s\n", BrightGreen, Reset, config.JupyterMode, BrightGreen, Reset, BrightCyan, config.JupyterPort, Reset)
	} else {
		fmt.Printf("  %s○%s Jupyter %s[Stopped]%s\n", BrightRed, Reset, BrightRed, Reset)
	}

	// VS Code
	if isRunning("vscode") {
		fmt.Printf("  %s●%s VS Code %s[Running]%s port %s%d%s\n", BrightGreen, Reset, BrightGreen, Reset, BrightCyan, config.VSCodePort, Reset)
	} else {
		fmt.Printf("  %s○%s VS Code %s[Stopped]%s\n", BrightRed, Reset, BrightRed, Reset)
	}

	// SSH
	if isRunning("ssh") {
		fmt.Printf("  %s●%s SSH Terminal %s[Running]%s port %s%d%s\n", BrightGreen, Reset, BrightGreen, Reset, BrightCyan, config.SSHPort, Reset)
	} else {
		fmt.Printf("  %s○%s SSH Terminal %s[Stopped]%s\n", BrightRed, Reset, BrightRed, Reset)
	}

	// Dashboard
	if isRunning("dashboard") {
		fmt.Printf("  %s●%s Dashboard %s[Running]%s port %s%d%s\n", BrightGreen, Reset, BrightGreen, Reset, BrightCyan, config.DashboardPort, Reset)
	} else {
		fmt.Printf("  %s○%s Dashboard %s[Stopped]%s\n", BrightRed, Reset, BrightRed, Reset)
	}

	showTunnelStatus()

	printHeader("🔐 CREDENTIALS")
	fmt.Printf("  Jupyter:   %s%s%s\n", BrightYellow, config.JupyterPassword, Reset)
	fmt.Printf("  VS Code:   %s%s%s\n", BrightYellow, config.VSCodePassword, Reset)
	fmt.Printf("  SSH User:  %s%s%s\n", BrightYellow, config.SSHUser, Reset)
	if config.SSHPassword != "" {
		fmt.Printf("  SSH Pass:  %s%s%s\n", BrightYellow, config.SSHPassword, Reset)
	}
	fmt.Println()
}

func showLogs(service string) {
	logPath := filepath.Join(cloudlabDir, "logs", service+".log")
	data, err := os.ReadFile(logPath)
	if err != nil {
		printError("Log not found: " + logPath)
		return
	}
	fmt.Printf("\n%s=== %s logs ===%s\n\n", BrightCyan, service, Reset)
	fmt.Println(string(data))
}

// ==================== Kernels ====================

func handleKernel(args []string) {
	switch args[0] {
	case "list":
		listKernels()
	case "add":
		if len(args) < 2 {
			printError("Usage: cloudlab kernel add <name> [version]")
			return
		}
		ver := config.PythonVersion
		if len(args) > 2 {
			ver = args[2]
		}
		addKernel(args[1], ver)
	case "remove", "rm":
		if len(args) < 2 {
			printError("Usage: cloudlab kernel remove <name>")
			return
		}
		removeKernel(args[1])
	default:
		printError("Unknown: " + args[0])
	}
}

func listKernels() {
	printHeader("📓 JUPYTER KERNELS")
	jp := getJupyterPath()
	if _, err := os.Stat(jp); err != nil {
		printError("Jupyter not installed")
		return
	}
	cmd := exec.Command(jp, "kernelspec", "list")
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	cmd.Run()
}

func addKernel(name, ver string) {
	printStep(fmt.Sprintf("Creating kernel %s with Python %s...", name, ver))
	uv := getUVPath()
	if uv == "" {
		printError("UV not found")
		return
	}

	envPath := filepath.Join(cloudlabDir, "envs", name)
	exec.Command(uv, "venv", envPath, "--python", ver).Run()

	var py string
	if runtime.GOOS == "windows" {
		py = filepath.Join(envPath, "Scripts", "python.exe")
	} else {
		py = filepath.Join(envPath, "bin", "python")
	}

	exec.Command(uv, "pip", "install", "ipykernel", "--python", py).Run()
	exec.Command(py, "-m", "ipykernel", "install", "--user", "--name", name, "--display-name", fmt.Sprintf("Python %s (%s)", ver, name)).Run()

	printSuccess(fmt.Sprintf("Kernel %s created", name))
}

func removeKernel(name string) {
	printStep("Removing kernel " + name + "...")
	jp := getJupyterPath()
	if jp != "" {
		exec.Command(jp, "kernelspec", "uninstall", name, "-f").Run()
	}
	os.RemoveAll(filepath.Join(cloudlabDir, "envs", name))
	printSuccess("Kernel removed")
}

// ==================== Environments ====================

func handleEnv(args []string) {
	switch args[0] {
	case "list":
		listEnvs()
	case "create":
		if len(args) < 3 {
			printError("Usage: cloudlab env create <name> <version>")
			return
		}
		createEnv(args[1], args[2])
	case "remove", "rm":
		if len(args) < 2 {
			printError("Usage: cloudlab env remove <name>")
			return
		}
		os.RemoveAll(filepath.Join(cloudlabDir, "envs", args[1]))
		printSuccess("Environment removed")
	case "install":
		if len(args) < 2 {
			printError("Usage: cloudlab env install <package>")
			return
		}
		installPkg(strings.Join(args[1:], " "))
	default:
		printError("Unknown: " + args[0])
	}
}

func listEnvs() {
	printHeader("🐍 ENVIRONMENTS")
	venv := filepath.Join(cloudlabDir, "venv")
	if _, err := os.Stat(venv); err == nil {
		fmt.Printf("  %s★%s cloudlab (default)\n", BrightYellow, Reset)
	}
	entries, _ := os.ReadDir(filepath.Join(cloudlabDir, "envs"))
	for _, e := range entries {
		if e.IsDir() {
			fmt.Printf("  %s○%s %s\n", Dim, Reset, e.Name())
		}
	}
	fmt.Println()
}

func createEnv(name, ver string) {
	printStep(fmt.Sprintf("Creating %s with Python %s...", name, ver))
	uv := getUVPath()
	if uv == "" {
		printError("UV not found")
		return
	}
	envPath := filepath.Join(cloudlabDir, "envs", name)
	exec.Command(uv, "venv", envPath, "--python", ver).Run()
	printSuccess("Environment created")
}

func installPkg(pkg string) {
	printStep("Installing " + pkg + "...")
	uv := getUVPath()
	if uv == "" {
		printError("UV not found")
		return
	}
	py := getPythonPath()
	cmd := exec.Command(uv, "pip", "install", pkg, "--python", py)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	cmd.Run()
	printSuccess("Installed " + pkg)
}

// ==================== Email ====================

func handleEmail(action string) {
	switch action {
	case "setup":
		setupEmail()
	case "test":
		sendTestEmail()
	case "send":
		sendTunnelEmail()
	default:
		printError("Unknown: " + action)
	}
}

func showEmailConfig() {
	printHeader("📧 EMAIL CONFIG")
	if config.Email != "" {
		fmt.Printf("  Email: %s%s%s\n", BrightMagenta, config.Email, Reset)
		fmt.Printf("  SMTP:  %s%s:%d%s\n", Dim, config.SMTPServer, config.SMTPPort, Reset)
	} else {
		printWarning("Email not configured. Run: cloudlab email setup")
	}
	fmt.Println()
}

func setupEmail() {
	printHeader("📧 EMAIL SETUP")
	reader := bufio.NewReader(os.Stdin)

	fmt.Printf("  Email address: ")
	config.Email = readLine(reader)
	if config.Email == "" {
		return
	}

	detectSMTP(config.Email)

	fmt.Printf("  App password: ")
	config.EmailPassword = readLine(reader)

	saveConfig()
	printSuccess("Email configured")

	fmt.Printf("\nSend test email? [Y/n]: ")
	if ans := strings.ToLower(readLine(reader)); ans != "n" {
		sendTestEmail()
	}
}

func sendTestEmail() {
	if config.Email == "" {
		printError("Email not configured")
		return
	}
	printStep("Sending test email...")

	body := fmt.Sprintf(`<html><body style="font-family:sans-serif;padding:40px;background:#f5f5f5;">
<div style="max-width:500px;margin:0 auto;background:white;padding:40px;border-radius:16px;">
<h1 style="color:#7c3aed;">☁️ CloudLab</h1>
<div style="background:#dcfce7;color:#166534;padding:20px;border-radius:12px;">
<h2>✅ Email Working!</h2>
</div>
<p>Your email notifications are configured correctly.</p>
<hr style="border:none;border-top:1px solid #e5e7eb;margin:30px 0;">
<p style="color:#999;font-size:12px;">CloudLab v%s | %s | <a href="%s">GitHub</a></p>
</div></body></html>`, VERSION, AUTHOR, GITHUB)

	if err := sendEmail("CloudLab - Test ✓", body); err != nil {
		printError("Failed: " + err.Error())
		return
	}
	printSuccess("Test email sent!")
}

func sendTunnelEmail() {
	if config.Email == "" {
		printWarning("Email not configured")
		return
	}
	loadConfig()

	if config.TunnelURLs.Jupyter == "" && config.TunnelURLs.VSCode == "" && config.TunnelURLs.SSH == "" && config.TunnelURLs.Dashboard == "" {
		printWarning("No tunnel URLs. Run: cloudlab tunnel start")
		return
	}

	printStep("Sending tunnel URLs...")
	hostname, _ := os.Hostname()

	// Build sections
	sections := ""

	if config.TunnelURLs.Jupyter != "" {
		sections += fmt.Sprintf(`
<div style="background:linear-gradient(135deg,#fef3c7,#fde68a);padding:24px;border-radius:12px;margin:20px 0;">
<h2 style="color:#92400e;margin:0 0 12px;">🐍 Jupyter %s</h2>
<p><strong>URL:</strong> <a href="%s">%s</a></p>
<p><strong>Password:</strong> <code style="background:#fef3c7;padding:4px 8px;border-radius:4px;">%s</code></p>
</div>`, config.JupyterMode, config.TunnelURLs.Jupyter, config.TunnelURLs.Jupyter, config.JupyterPassword)
	}

	if config.TunnelURLs.VSCode != "" {
		sections += fmt.Sprintf(`
<div style="background:linear-gradient(135deg,#dbeafe,#bfdbfe);padding:24px;border-radius:12px;margin:20px 0;">
<h2 style="color:#1e40af;margin:0 0 12px;">💻 VS Code</h2>
<p><strong>URL:</strong> <a href="%s">%s</a></p>
<p><strong>Password:</strong> <code style="background:#dbeafe;padding:4px 8px;border-radius:4px;">%s</code></p>
</div>`, config.TunnelURLs.VSCode, config.TunnelURLs.VSCode, config.VSCodePassword)
	}

	if config.TunnelURLs.SSH != "" {
		sshPass := "System credentials"
		if config.SSHPassword != "" {
			sshPass = config.SSHPassword
		}
		sections += fmt.Sprintf(`
<div style="background:linear-gradient(135deg,#d1fae5,#a7f3d0);padding:24px;border-radius:12px;margin:20px 0;">
<h2 style="color:#065f46;margin:0 0 12px;">🔒 SSH Terminal</h2>
<p><strong>URL:</strong> <a href="%s">%s</a></p>
<p><strong>Username:</strong> <code style="background:#d1fae5;padding:4px 8px;border-radius:4px;">%s</code></p>
<p><strong>Password:</strong> <code style="background:#d1fae5;padding:4px 8px;border-radius:4px;">%s</code></p>
</div>`, config.TunnelURLs.SSH, config.TunnelURLs.SSH, config.SSHUser, sshPass)
	}

	if config.TunnelURLs.Dashboard != "" {
		sections += fmt.Sprintf(`
<div style="background:linear-gradient(135deg,#f3e8ff,#e9d5ff);padding:24px;border-radius:12px;margin:20px 0;">
<h2 style="color:#7c3aed;margin:0 0 12px;">📊 Dashboard</h2>
<p><strong>URL:</strong> <a href="%s">%s</a></p>
<p style="font-size:12px;color:#6b21a8;">Manage all services from your browser!</p>
</div>`, config.TunnelURLs.Dashboard, config.TunnelURLs.Dashboard)
	}

	body := fmt.Sprintf(`<html><body style="font-family:sans-serif;padding:40px;background:#f5f5f5;">
<div style="max-width:600px;margin:0 auto;background:white;padding:40px;border-radius:16px;box-shadow:0 4px 6px rgba(0,0,0,0.1);">
<h1 style="color:#7c3aed;margin:0 0 10px;">☁️ CloudLab</h1>
<p style="color:#666;margin:0 0 30px;">Remote Development - %s</p>
%s
<div style="background:#f3e8ff;padding:20px;border-radius:12px;margin:20px 0;">
<h3 style="color:#7c3aed;margin:0 0 8px;">📁 Working Directory</h3>
<code style="color:#6b21a8;">%s</code>
</div>
<hr style="border:none;border-top:1px solid #e5e7eb;margin:30px 0;">
<p style="color:#999;font-size:12px;">CloudLab v%s | %s<br>Author: %s | <a href="%s">GitHub</a></p>
</div></body></html>`, hostname, sections, config.WorkDir, VERSION, time.Now().Format("2006-01-02 15:04:05"), AUTHOR, GITHUB)

	if err := sendEmail(fmt.Sprintf("☁️ CloudLab URLs - %s", hostname), body); err != nil {
		printError("Failed: " + err.Error())
		return
	}
	printSuccess("Tunnel URLs sent to " + config.Email)
}

func sendEmail(subject, body string) error {
	headers := fmt.Sprintf("From: CloudLab <%s>\r\nTo: %s\r\nSubject: %s\r\nMIME-Version: 1.0\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n",
		config.Email, config.Email, subject)

	addr := fmt.Sprintf("%s:%d", config.SMTPServer, config.SMTPPort)

	client, err := smtp.Dial(addr)
	if err != nil {
		return err
	}
	defer client.Close()

	if err := client.StartTLS(&tls.Config{ServerName: config.SMTPServer}); err != nil {
		return err
	}

	auth := smtp.PlainAuth("", config.Email, config.EmailPassword, config.SMTPServer)
	if err := client.Auth(auth); err != nil {
		return err
	}

	if err := client.Mail(config.Email); err != nil {
		return err
	}
	if err := client.Rcpt(config.Email); err != nil {
		return err
	}

	w, err := client.Data()
	if err != nil {
		return err
	}
	w.Write([]byte(headers + body))
	return w.Close()
}

// ==================== Update/Uninstall ====================

func updateAll() {
	printHeader("🔄 UPDATING")
	uv := getUVPath()
	if uv != "" {
		py := getPythonPath()
		exec.Command(uv, "pip", "install", "--upgrade", "jupyterlab", "notebook", "--python", py).Run()
	}
	printSuccess("Updated!")
}

func uninstallAll() {
	reader := bufio.NewReader(os.Stdin)
	fmt.Printf("\n%sUninstall CloudLab?%s [y/N]: ", BrightRed, Reset)
	if strings.ToLower(readLine(reader)) != "y" {
		return
	}
	stopAll()
	os.RemoveAll(cloudlabDir)
	printSuccess("Uninstalled!")
}

// ==================== Helpers ====================

func downloadFile(path, url string) error {
	resp, err := http.Get(url)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	out, err := os.Create(path)
	if err != nil {
		return err
	}
	defer out.Close()

	_, err = io.Copy(out, resp.Body)
	return err
}

func genToken(n int) string {
	b := make([]byte, n)
	rand.Read(b)
	return hex.EncodeToString(b)[:n]
}

func savePID(name string, pid int) {
	path := filepath.Join(cloudlabDir, "pids", name+".pid")
	os.WriteFile(path, []byte(strconv.Itoa(pid)), 0644)
}

func getPID(name string) int {
	data, err := os.ReadFile(filepath.Join(cloudlabDir, "pids", name+".pid"))
	if err != nil {
		return 0
	}
	pid, _ := strconv.Atoi(strings.TrimSpace(string(data)))
	return pid
}

func stopPID(name string) {
	pid := getPID(name)
	if pid == 0 {
		return
	}
	if runtime.GOOS == "windows" {
		exec.Command("taskkill", "/F", "/PID", strconv.Itoa(pid)).Run()
	} else {
		if p, err := os.FindProcess(pid); err == nil {
			p.Signal(syscall.SIGTERM)
			time.Sleep(500 * time.Millisecond)
			p.Kill()
		}
	}
	os.Remove(filepath.Join(cloudlabDir, "pids", name+".pid"))
}

func isRunning(name string) bool {
	pid := getPID(name)
	if pid == 0 {
		return false
	}
	if runtime.GOOS == "windows" {
		out, _ := exec.Command("tasklist", "/FI", fmt.Sprintf("PID eq %d", pid), "/NH").Output()
		return strings.Contains(string(out), strconv.Itoa(pid))
	}
	p, err := os.FindProcess(pid)
	if err != nil {
		return false
	}
	return p.Signal(syscall.Signal(0)) == nil
}

func printHeader(s string) {
	fmt.Printf("\n%s%s%s\n", Bold+BrightWhite, s, Reset)
	fmt.Printf("%s%s%s\n", Dim, strings.Repeat("─", 50), Reset)
}

func printStep(s string) {
	fmt.Printf("  %s▶%s %s\n", BrightBlue, Reset, s)
}

func printSuccess(s string) {
	fmt.Printf("  %s✓%s %s\n", BrightGreen, Reset, s)
}

func printError(s string) {
	fmt.Printf("  %s✗%s %s\n", BrightRed, Reset, s)
}

func printWarning(s string) {
	fmt.Printf("  %s⚠%s %s\n", BrightYellow, Reset, s)
}

func printInfo(s string) {
	fmt.Printf("  %s💡%s %s\n", BrightBlue, Reset, s)
}
