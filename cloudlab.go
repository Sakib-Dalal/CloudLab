// CloudLab CLI - Self-hosted Web Editor Setup Tool
// Author: Sakib Dalal
// GitHub: https://github.com/Sakib-Dalal
// DOCS: https://cloudlab-alpha.vercel.app/
// Uses UV package manager for fast Python management
// Supports Linux, macOS (including Apple Silicon with MPS), Windows

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
	VERSION = "1.0.0"
	AUTHOR  = "Sakib Dalal"
	GITHUB  = "https://github.com/Sakib-Dalal"
	DOCS    = "https://cloudlab-alpha.vercel.app/"
)

// ANSI Color codes
const (
	Reset     = "\033[0m"
	Bold      = "\033[1m"
	Dim       = "\033[2m"
	Italic    = "\033[3m"
	Underline = "\033[4m"

	// Regular colors
	Black   = "\033[30m"
	Red     = "\033[31m"
	Green   = "\033[32m"
	Yellow  = "\033[33m"
	Blue    = "\033[34m"
	Magenta = "\033[35m"
	Cyan    = "\033[36m"
	White   = "\033[37m"

	// Bright colors
	BrightBlack   = "\033[90m"
	BrightRed     = "\033[91m"
	BrightGreen   = "\033[92m"
	BrightYellow  = "\033[93m"
	BrightBlue    = "\033[94m"
	BrightMagenta = "\033[95m"
	BrightCyan    = "\033[96m"
	BrightWhite   = "\033[97m"

	// Background colors
	BgBlack   = "\033[40m"
	BgRed     = "\033[41m"
	BgGreen   = "\033[42m"
	BgYellow  = "\033[43m"
	BgBlue    = "\033[44m"
	BgMagenta = "\033[45m"
	BgCyan    = "\033[46m"
	BgWhite   = "\033[47m"
)

func getLogo() string {
	return fmt.Sprintf(`
%s%s   _____ _                 _ _           _     %s
%s%s  / ____| |               | | |         | |    %s
%s%s | |    | | ___  _   _  __| | |     __ _| |__  %s
%s%s | |    | |/ _ \| | | |/ _' | |    / _' | '_ \ %s
%s%s | |____| | (_) | |_| | (_| | |___| (_| | |_) |%s
%s%s  \_____|_|\___/ \__,_|\__,_|______\__,_|_.__/ %s
                                               
%s  ☁️  Self-Hosted Web Editor CLI %sv%s%s
%s  📦 Using UV Package Manager%s
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
		Dim, Reset,
		BrightYellow, Bold, AUTHOR, Reset,
		BrightBlue, Underline, GITHUB, Reset,
	)
}

// Configuration structure
type Config struct {
	JupyterPort      int        `json:"jupyter_port"`
	VSCodePort       int        `json:"vscode_port"`
	PythonVersion    string     `json:"python_version"`
	JupyterPassword  string     `json:"jupyter_password"`
	VSCodePassword   string     `json:"vscode_password"`
	JupyterMode      string     `json:"jupyter_mode"`
	WorkingDirectory string     `json:"working_directory"`
	EmailAddress     string     `json:"email_address"`
	EmailAppPassword string     `json:"email_app_password"`
	SMTPServer       string     `json:"smtp_server"`
	SMTPPort         int        `json:"smtp_port"`
	EnableMPS        bool       `json:"enable_mps"`
	EnableCUDA       bool       `json:"enable_cuda"`
	LowPowerMode     bool       `json:"low_power_mode"`
	TunnelURLs       TunnelURLs `json:"tunnel_urls"`
	NotifyOnStart    bool       `json:"notify_on_start"`
}

type TunnelURLs struct {
	Jupyter string `json:"jupyter"`
	VSCode  string `json:"vscode"`
}

var config Config
var homeDir string
var configPath string
var cloudlabDir string

func main() {
	// Use single CPU for lower power consumption
	runtime.GOMAXPROCS(1)

	homeDir, _ = os.UserHomeDir()
	cloudlabDir = filepath.Join(homeDir, ".cloudlab")
	configPath = filepath.Join(cloudlabDir, "config.json")

	// Create directories
	os.MkdirAll(cloudlabDir, 0755)
	os.MkdirAll(filepath.Join(cloudlabDir, "logs"), 0755)
	os.MkdirAll(filepath.Join(cloudlabDir, "pids"), 0755)
	os.MkdirAll(filepath.Join(cloudlabDir, "envs"), 0755)

	loadConfig()

	if len(os.Args) < 2 {
		showHelp()
		return
	}

	command := os.Args[1]
	args := os.Args[2:]

	switch command {
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
			printError("Usage: cloudlab logs <service>")
			printInfo("Services: jupyter, vscode, tunnel_jupyter, tunnel_vscode")
		}
	case "config":
		if len(args) > 0 {
			configureOption(args)
		} else {
			showConfig()
		}
	case "tunnel":
		if len(args) > 0 {
			manageTunnel(args[0])
		} else {
			showTunnelStatus()
		}
	case "kernel":
		if len(args) > 0 {
			manageKernel(args)
		} else {
			listKernels()
		}
	case "env":
		if len(args) > 0 {
			manageEnvironment(args)
		} else {
			listEnvironments()
		}
	case "email":
		if len(args) > 0 {
			manageEmail(args)
		} else {
			showEmailConfig()
		}
	case "update":
		updateComponents()
	case "uninstall":
		uninstallAll()
	case "help", "-h", "--help":
		showHelp()
	case "version", "-v", "--version":
		showVersion()
	default:
		printError("Unknown command: " + command)
		showHelp()
	}
}

func showVersion() {
	fmt.Printf("%s%s☁️  CloudLab CLI%s %sv%s%s\n", Bold, BrightCyan, Reset, BrightGreen, VERSION, Reset)
	fmt.Printf("%s👤 Author:%s %s%s%s\n", Dim, Reset, BrightYellow, AUTHOR, Reset)
	fmt.Printf("%s🔗 GitHub:%s %s%s%s\n", Dim, Reset, BrightBlue+Underline, GITHUB, Reset)
	fmt.Printf("%s📄 DOCS:%s %s%s%s\n", Dim, Reset, BrightBlue+Underline, DOCS, Reset)
}

func showHelp() {
	fmt.Println(getLogo())

	fmt.Printf("\n%s%sUSAGE:%s\n", Bold, BrightWhite, Reset)
	fmt.Printf("  %scloudlab%s <command> [options]\n", BrightCyan, Reset)

	fmt.Printf("\n%s%sSERVICE COMMANDS:%s\n", Bold, BrightGreen, Reset)
	fmt.Printf("  %sinit%s                    Initialize and configure CloudLab\n", BrightYellow, Reset)
	fmt.Printf("  %sinstall%s [component]     Install components %s(all|jupyter|vscode|cloudflare|uv)%s\n", BrightYellow, Reset, Dim, Reset)
	fmt.Printf("  %sstart%s [service]         Start services %s(all|jupyter|lab|notebook|vscode|tunnel)%s\n", BrightYellow, Reset, Dim, Reset)
	fmt.Printf("  %sstop%s [service]          Stop services %s(all|jupyter|vscode|tunnel)%s\n", BrightYellow, Reset, Dim, Reset)
	fmt.Printf("  %srestart%s [service]       Restart services\n", BrightYellow, Reset)
	fmt.Printf("  %sstatus%s                  Show status of all services\n", BrightYellow, Reset)
	fmt.Printf("  %slogs%s <service>          Show logs for a service\n", BrightYellow, Reset)

	fmt.Printf("\n%s%sCONFIGURATION:%s\n", Bold, BrightBlue, Reset)
	fmt.Printf("  %sconfig%s                  Show current configuration\n", BrightYellow, Reset)
	fmt.Printf("  %sconfig set%s <key> <val>  Set configuration value\n", BrightYellow, Reset)
	fmt.Printf("  %sconfig reset%s            Reset configuration to defaults\n", BrightYellow, Reset)

	fmt.Printf("\n%s%sTUNNEL COMMANDS:%s\n", Bold, BrightMagenta, Reset)
	fmt.Printf("  %stunnel%s                  Show tunnel status and URLs\n", BrightYellow, Reset)
	fmt.Printf("  %stunnel start%s            Start Cloudflare tunnels\n", BrightYellow, Reset)
	fmt.Printf("  %stunnel stop%s             Stop Cloudflare tunnels\n", BrightYellow, Reset)
	fmt.Printf("  %stunnel restart%s          Restart tunnels and get new URLs\n", BrightYellow, Reset)

	fmt.Printf("\n%s%sKERNEL MANAGEMENT:%s\n", Bold, BrightCyan, Reset)
	fmt.Printf("  %skernel list%s             List installed Jupyter kernels\n", BrightYellow, Reset)
	fmt.Printf("  %skernel add%s <name>       Add a new kernel with default Python\n", BrightYellow, Reset)
	fmt.Printf("  %skernel add%s <name> <ver> Add a new kernel with specific Python version\n", BrightYellow, Reset)
	fmt.Printf("  %skernel remove%s <name>    Remove a kernel\n", BrightYellow, Reset)

	fmt.Printf("\n%s%sENVIRONMENT MANAGEMENT:%s\n", Bold, BrightYellow, Reset)
	fmt.Printf("  %senv list%s                List Python environments\n", BrightYellow, Reset)
	fmt.Printf("  %senv create%s <name> <ver> Create new environment with Python version\n", BrightYellow, Reset)
	fmt.Printf("  %senv remove%s <name>       Remove environment\n", BrightYellow, Reset)
	fmt.Printf("  %senv activate%s <name>     Show activation command\n", BrightYellow, Reset)
	fmt.Printf("  %senv install%s <pkg>       Install package in default environment\n", BrightYellow, Reset)

	fmt.Printf("\n%s%sEMAIL NOTIFICATIONS:%s\n", Bold, BrightRed, Reset)
	fmt.Printf("  %semail setup%s             Setup email notifications\n", BrightYellow, Reset)
	fmt.Printf("  %semail test%s              Send a test email\n", BrightYellow, Reset)
	fmt.Printf("  %semail send%s              Send tunnel URLs via email\n", BrightYellow, Reset)

	fmt.Printf("\n%s%sOTHER COMMANDS:%s\n", Bold, Dim, Reset)
	fmt.Printf("  %supdate%s                  Update all components\n", BrightYellow, Reset)
	fmt.Printf("  %suninstall%s               Uninstall all components\n", BrightYellow, Reset)
	fmt.Printf("  %shelp%s                    Show this help message\n", BrightYellow, Reset)
	fmt.Printf("  %sversion%s                 Show version\n", BrightYellow, Reset)

	fmt.Printf("\n%s%sEXAMPLES:%s\n", Bold, BrightWhite, Reset)
	fmt.Printf("  %s$%s cloudlab init\n", BrightGreen, Reset)
	fmt.Printf("  %s$%s cloudlab install all\n", BrightGreen, Reset)
	fmt.Printf("  %s$%s cloudlab start all\n", BrightGreen, Reset)
	fmt.Printf("  %s$%s cloudlab start lab\n", BrightGreen, Reset)
	fmt.Printf("  %s$%s cloudlab start notebook\n", BrightGreen, Reset)
	fmt.Printf("  %s$%s cloudlab tunnel start\n", BrightGreen, Reset)
	fmt.Printf("  %s$%s cloudlab email send\n", BrightGreen, Reset)
	fmt.Printf("  %s$%s cloudlab config set jupyter_mode notebook\n", BrightGreen, Reset)
	fmt.Printf("  %s$%s cloudlab kernel add datascience 3.10\n", BrightGreen, Reset)
	fmt.Printf("  %s$%s cloudlab env create ml 3.11\n", BrightGreen, Reset)
	fmt.Println()
}

// ==================== Configuration ====================

func loadConfig() {
	// Default configuration
	config = Config{
		JupyterPort:      8888,
		VSCodePort:       8080,
		PythonVersion:    "3.11",
		JupyterMode:      "lab",
		WorkingDirectory: homeDir,
		SMTPPort:         587,
		LowPowerMode:     true,
		NotifyOnStart:    true,
	}

	// Detect Apple Silicon
	if runtime.GOOS == "darwin" && runtime.GOARCH == "arm64" {
		config.EnableMPS = true
	}

	// Detect NVIDIA GPU
	if checkNVIDIAGPU() {
		config.EnableCUDA = true
	}

	// Load from file
	data, err := os.ReadFile(configPath)
	if err == nil {
		json.Unmarshal(data, &config)
	}
}

func saveConfig() {
	os.MkdirAll(filepath.Dir(configPath), 0755)
	data, _ := json.MarshalIndent(config, "", "  ")
	os.WriteFile(configPath, data, 0600)
}

func showConfig() {
	fmt.Println(getLogo())

	fmt.Printf("\n%s%s📋 CONFIGURATION%s\n", Bold, BrightCyan, Reset)
	printDivider()

	// Create display config with hidden passwords
	fmt.Printf("  %s%-20s%s : %s%d%s\n", BrightYellow, "jupyter_port", Reset, BrightWhite, config.JupyterPort, Reset)
	fmt.Printf("  %s%-20s%s : %s%d%s\n", BrightYellow, "vscode_port", Reset, BrightWhite, config.VSCodePort, Reset)
	fmt.Printf("  %s%-20s%s : %s%s%s\n", BrightYellow, "python_version", Reset, BrightWhite, config.PythonVersion, Reset)
	fmt.Printf("  %s%-20s%s : %s%s%s\n", BrightYellow, "jupyter_mode", Reset, BrightGreen, config.JupyterMode, Reset)
	fmt.Printf("  %s%-20s%s : %s%s%s\n", BrightYellow, "working_directory", Reset, BrightBlue, config.WorkingDirectory, Reset)

	// Passwords (hidden)
	if config.JupyterPassword != "" {
		fmt.Printf("  %s%-20s%s : %s********%s\n", BrightYellow, "jupyter_password", Reset, Dim, Reset)
	} else {
		fmt.Printf("  %s%-20s%s : %s(not set)%s\n", BrightYellow, "jupyter_password", Reset, Dim, Reset)
	}
	if config.VSCodePassword != "" {
		fmt.Printf("  %s%-20s%s : %s********%s\n", BrightYellow, "vscode_password", Reset, Dim, Reset)
	} else {
		fmt.Printf("  %s%-20s%s : %s(not set)%s\n", BrightYellow, "vscode_password", Reset, Dim, Reset)
	}

	// Email
	if config.EmailAddress != "" {
		fmt.Printf("  %s%-20s%s : %s%s%s\n", BrightYellow, "email_address", Reset, BrightMagenta, config.EmailAddress, Reset)
	} else {
		fmt.Printf("  %s%-20s%s : %s(not set)%s\n", BrightYellow, "email_address", Reset, Dim, Reset)
	}
	fmt.Printf("  %s%-20s%s : %s%s:%d%s\n", BrightYellow, "smtp", Reset, Dim, config.SMTPServer, config.SMTPPort, Reset)

	// Hardware
	fmt.Printf("  %s%-20s%s : %s%v%s\n", BrightYellow, "enable_mps", Reset, boolColor(config.EnableMPS), config.EnableMPS, Reset)
	fmt.Printf("  %s%-20s%s : %s%v%s\n", BrightYellow, "enable_cuda", Reset, boolColor(config.EnableCUDA), config.EnableCUDA, Reset)
	fmt.Printf("  %s%-20s%s : %s%v%s\n", BrightYellow, "low_power_mode", Reset, boolColor(config.LowPowerMode), config.LowPowerMode, Reset)
	fmt.Printf("  %s%-20s%s : %s%v%s\n", BrightYellow, "notify_on_start", Reset, boolColor(config.NotifyOnStart), config.NotifyOnStart, Reset)

	fmt.Printf("\n%sTo change a value:%s\n", Dim, Reset)
	fmt.Printf("  %s$%s cloudlab config set <key> <value>\n", BrightGreen, Reset)
	fmt.Println()
}

func boolColor(b bool) string {
	if b {
		return BrightGreen
	}
	return BrightRed
}

func configureOption(args []string) {
	if args[0] == "reset" {
		os.Remove(configPath)
		loadConfig()
		saveConfig()
		printSuccess("Configuration reset to defaults!")
		return
	}

	if args[0] == "set" && len(args) >= 3 {
		key := args[1]
		value := strings.Join(args[2:], " ")

		switch key {
		case "jupyter_port":
			if p, err := strconv.Atoi(value); err == nil {
				config.JupyterPort = p
			}
		case "vscode_port":
			if p, err := strconv.Atoi(value); err == nil {
				config.VSCodePort = p
			}
		case "python_version":
			config.PythonVersion = value
		case "jupyter_mode":
			if value == "lab" || value == "notebook" {
				config.JupyterMode = value
			} else {
				printError("jupyter_mode must be 'lab' or 'notebook'")
				return
			}
		case "working_directory":
			if _, err := os.Stat(value); os.IsNotExist(err) {
				printError("Directory does not exist: " + value)
				return
			}
			config.WorkingDirectory = value
		case "jupyter_password":
			config.JupyterPassword = value
			configureJupyter()
		case "vscode_password":
			config.VSCodePassword = value
			configureVSCode()
		case "email_address":
			config.EmailAddress = value
		case "email_app_password":
			config.EmailAppPassword = value
		case "smtp_server":
			config.SMTPServer = value
		case "smtp_port":
			if p, err := strconv.Atoi(value); err == nil {
				config.SMTPPort = p
			}
		case "enable_mps":
			config.EnableMPS = value == "true" || value == "1" || value == "yes"
		case "enable_cuda":
			config.EnableCUDA = value == "true" || value == "1" || value == "yes"
		case "low_power_mode":
			config.LowPowerMode = value == "true" || value == "1" || value == "yes"
		case "notify_on_start":
			config.NotifyOnStart = value == "true" || value == "1" || value == "yes"
		default:
			printError("Unknown configuration key: " + key)
			return
		}

		saveConfig()
		printSuccess(fmt.Sprintf("Set %s%s%s = %s%s%s", BrightYellow, key, Reset, BrightCyan, value, Reset))
	} else {
		printError("Usage: cloudlab config set <key> <value>")
	}
}

// ==================== Initialization ====================

func initSetup() {
	fmt.Println(getLogo())
	printHeader("🚀 INITIALIZATION")

	reader := bufio.NewReader(os.Stdin)

	// Working directory
	fmt.Printf("\n%s[1/8]%s Enter working directory [%s%s%s]: ", BrightCyan, Reset, Dim, config.WorkingDirectory, Reset)
	workDir, _ := reader.ReadString('\n')
	workDir = strings.TrimSpace(workDir)
	if workDir != "" {
		if _, err := os.Stat(workDir); os.IsNotExist(err) {
			printWarning("Directory does not exist, creating...")
			os.MkdirAll(workDir, 0755)
		}
		config.WorkingDirectory = workDir
	}

	// Jupyter mode
	fmt.Printf("%s[2/8]%s Jupyter mode (lab/notebook) [%s%s%s]: ", BrightCyan, Reset, Dim, config.JupyterMode, Reset)
	mode, _ := reader.ReadString('\n')
	mode = strings.TrimSpace(strings.ToLower(mode))
	if mode == "lab" || mode == "notebook" {
		config.JupyterMode = mode
	}

	// Jupyter port
	fmt.Printf("%s[3/8]%s Enter Jupyter port [%s%d%s]: ", BrightCyan, Reset, Dim, config.JupyterPort, Reset)
	portStr, _ := reader.ReadString('\n')
	portStr = strings.TrimSpace(portStr)
	if portStr != "" {
		if port, err := strconv.Atoi(portStr); err == nil && port > 0 && port < 65536 {
			config.JupyterPort = port
		}
	}

	// VS Code port
	fmt.Printf("%s[4/8]%s Enter VS Code port [%s%d%s]: ", BrightCyan, Reset, Dim, config.VSCodePort, Reset)
	portStr, _ = reader.ReadString('\n')
	portStr = strings.TrimSpace(portStr)
	if portStr != "" {
		if port, err := strconv.Atoi(portStr); err == nil && port > 0 && port < 65536 {
			config.VSCodePort = port
		}
	}

	// Jupyter password
	fmt.Printf("%s[5/8]%s Enter Jupyter password %s(Enter for auto-generate)%s: ", BrightCyan, Reset, Dim, Reset)
	password, _ := reader.ReadString('\n')
	password = strings.TrimSpace(password)
	if password != "" {
		config.JupyterPassword = password
	} else {
		config.JupyterPassword = generateSecureToken(16)
		fmt.Printf("       %s🔑 Generated:%s %s%s%s\n", Dim, Reset, BrightGreen, config.JupyterPassword, Reset)
	}

	// VS Code password
	fmt.Printf("%s[6/8]%s Enter VS Code password %s(Enter for auto-generate)%s: ", BrightCyan, Reset, Dim, Reset)
	password, _ = reader.ReadString('\n')
	password = strings.TrimSpace(password)
	if password != "" {
		config.VSCodePassword = password
	} else {
		config.VSCodePassword = generateSecureToken(16)
		fmt.Printf("       %s🔑 Generated:%s %s%s%s\n", Dim, Reset, BrightGreen, config.VSCodePassword, Reset)
	}

	// Email setup
	fmt.Printf("%s[7/8]%s Enter email for notifications %s(optional)%s: ", BrightCyan, Reset, Dim, Reset)
	email, _ := reader.ReadString('\n')
	email = strings.TrimSpace(email)
	if email != "" {
		config.EmailAddress = email
		detectSMTPSettings(email)

		fmt.Printf("       Enter email app password: ")
		appPass, _ := reader.ReadString('\n')
		config.EmailAppPassword = strings.TrimSpace(appPass)
	}

	// Python version
	fmt.Printf("%s[8/8]%s Enter Python version [%s%s%s]: ", BrightCyan, Reset, Dim, config.PythonVersion, Reset)
	pyVer, _ := reader.ReadString('\n')
	pyVer = strings.TrimSpace(pyVer)
	if pyVer != "" {
		config.PythonVersion = pyVer
	}

	// Hardware detection
	fmt.Printf("\n%s%s🔧 HARDWARE DETECTION%s\n", Bold, BrightYellow, Reset)
	printDivider()

	if config.EnableMPS {
		printSuccess("Apple Silicon detected - MPS acceleration enabled")
	}
	if config.EnableCUDA {
		printSuccess("NVIDIA GPU detected - CUDA acceleration enabled")
	}
	if !config.EnableMPS && !config.EnableCUDA {
		printInfo("No GPU detected - Using CPU mode")
	}
	if config.LowPowerMode {
		printInfo("Low power mode: enabled (optimized for energy efficiency)")
	}

	saveConfig()
	printSuccess("\n✅ Configuration saved to " + configPath)

	fmt.Printf("\n%sInstall all components now?%s [Y/n]: ", BrightWhite, Reset)
	answer, _ := reader.ReadString('\n')
	answer = strings.TrimSpace(strings.ToLower(answer))
	if answer == "" || answer == "y" || answer == "yes" {
		installAll()
	}
}

func detectSMTPSettings(email string) {
	email = strings.ToLower(email)
	if strings.Contains(email, "gmail") {
		config.SMTPServer = "smtp.gmail.com"
		config.SMTPPort = 587
		printInfo("Gmail detected - SMTP: smtp.gmail.com:587")
		fmt.Printf("       %s💡 Get App Password: https://myaccount.google.com/apppasswords%s\n", Dim, Reset)
	} else if strings.Contains(email, "outlook") || strings.Contains(email, "hotmail") || strings.Contains(email, "live") {
		config.SMTPServer = "smtp-mail.outlook.com"
		config.SMTPPort = 587
		printInfo("Outlook detected - SMTP: smtp-mail.outlook.com:587")
	} else if strings.Contains(email, "yahoo") {
		config.SMTPServer = "smtp.mail.yahoo.com"
		config.SMTPPort = 587
		printInfo("Yahoo detected - SMTP: smtp.mail.yahoo.com:587")
	} else if strings.Contains(email, "icloud") {
		config.SMTPServer = "smtp.mail.me.com"
		config.SMTPPort = 587
		printInfo("iCloud detected - SMTP: smtp.mail.me.com:587")
	} else {
		fmt.Printf("       Enter SMTP server: ")
		reader := bufio.NewReader(os.Stdin)
		smtpServer, _ := reader.ReadString('\n')
		config.SMTPServer = strings.TrimSpace(smtpServer)
		config.SMTPPort = 587
	}
}

// ==================== Installation ====================

func installAll() {
	printHeader("📦 INSTALLING COMPONENTS")
	fmt.Println()

	installComponent("uv")
	installComponent("jupyter")
	installComponent("vscode")
	installComponent("cloudflare")

	fmt.Println()
	printSuccess("✅ All components installed!")
	fmt.Println()
	printBox("NEXT STEPS", []string{
		"1. cloudlab start all     # Start all services",
		"2. cloudlab tunnel start  # Get public URLs",
		"3. cloudlab status        # Check status",
	})
}

func installComponent(component string) {
	switch component {
	case "uv":
		installUV()
	case "jupyter":
		installJupyter()
	case "vscode":
		installVSCode()
	case "cloudflare", "cloudflared":
		installCloudflare()
	case "all":
		installAll()
	default:
		printError("Unknown component: " + component)
		printInfo("Available: all, uv, jupyter, vscode, cloudflare")
	}
}

func installUV() {
	printStep("Installing UV package manager...")

	if _, err := exec.LookPath("uv"); err == nil {
		printSuccess("UV already installed ✓")
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
	if err := cmd.Run(); err != nil {
		printError("Failed to install UV: " + err.Error())
		return
	}
	printSuccess("UV installed ✓")
}

func getUVPath() string {
	locations := []string{
		filepath.Join(homeDir, ".cargo", "bin", "uv"),
		filepath.Join(homeDir, ".local", "bin", "uv"),
		"/usr/local/bin/uv",
	}

	if runtime.GOOS == "windows" {
		locations = []string{
			filepath.Join(homeDir, ".cargo", "bin", "uv.exe"),
			filepath.Join(os.Getenv("LOCALAPPDATA"), "uv", "uv.exe"),
		}
	}

	for _, loc := range locations {
		if _, err := os.Stat(loc); err == nil {
			return loc
		}
	}

	if path, err := exec.LookPath("uv"); err == nil {
		return path
	}

	return ""
}

func installJupyter() {
	printStep("Installing Jupyter...")

	uvPath := getUVPath()
	if uvPath == "" {
		printWarning("UV not found. Installing UV first...")
		installUV()
		uvPath = getUVPath()
		if uvPath == "" {
			printError("Failed to find UV after installation")
			return
		}
	}

	venvPath := filepath.Join(cloudlabDir, "venv")

	// Create virtual environment
	fmt.Printf("  %s→%s Creating Python %s%s%s environment...\n", Dim, Reset, BrightCyan, config.PythonVersion, Reset)
	cmd := exec.Command(uvPath, "venv", venvPath, "--python", config.PythonVersion)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	if err := cmd.Run(); err != nil {
		printError("Failed to create virtual environment: " + err.Error())
		return
	}

	// Install packages
	packages := []string{"jupyterlab", "notebook", "ipykernel", "ipywidgets", "nbconvert"}
	pythonPath := getPythonPath()

	for _, pkg := range packages {
		fmt.Printf("  %s→%s Installing %s%s%s...\n", Dim, Reset, BrightYellow, pkg, Reset)
		cmd := exec.Command(uvPath, "pip", "install", pkg, "--python", pythonPath)
		cmd.Run()
	}

	// Install PyTorch with appropriate backend
	if config.EnableMPS {
		fmt.Printf("  %s→%s Installing PyTorch with %sMPS%s support...\n", Dim, Reset, BrightGreen, Reset)
		cmd := exec.Command(uvPath, "pip", "install", "torch", "torchvision", "torchaudio", "--python", pythonPath)
		cmd.Stdout = os.Stdout
		cmd.Stderr = os.Stderr
		cmd.Run()
	} else if config.EnableCUDA {
		fmt.Printf("  %s→%s Installing PyTorch with %sCUDA%s support...\n", Dim, Reset, BrightGreen, Reset)
		cmd := exec.Command(uvPath, "pip", "install", "torch", "torchvision", "torchaudio",
			"--index-url", "https://download.pytorch.org/whl/cu121", "--python", pythonPath)
		cmd.Stdout = os.Stdout
		cmd.Stderr = os.Stderr
		cmd.Run()
	}

	// Register default kernel
	fmt.Printf("  %s→%s Registering Jupyter kernel...\n", Dim, Reset)
	cmd = exec.Command(pythonPath, "-m", "ipykernel", "install", "--user", "--name", "cloudlab",
		"--display-name", "Python "+config.PythonVersion+" (CloudLab)")
	cmd.Run()

	configureJupyter()
	printSuccess("Jupyter installed ✓")
}

func getPythonPath() string {
	venvPath := filepath.Join(cloudlabDir, "venv")
	if runtime.GOOS == "windows" {
		return filepath.Join(venvPath, "Scripts", "python.exe")
	}
	return filepath.Join(venvPath, "bin", "python")
}

func getJupyterPath() string {
	venvPath := filepath.Join(cloudlabDir, "venv")
	if runtime.GOOS == "windows" {
		return filepath.Join(venvPath, "Scripts", "jupyter.exe")
	}
	return filepath.Join(venvPath, "bin", "jupyter")
}

func configureJupyter() {
	fmt.Printf("  %s→%s Configuring Jupyter...\n", Dim, Reset)

	jupyterDir := filepath.Join(homeDir, ".jupyter")
	os.MkdirAll(jupyterDir, 0755)

	// Generate password hash
	pythonPath := getPythonPath()
	hashCmd := fmt.Sprintf(`from jupyter_server.auth import passwd; print(passwd('%s'))`, config.JupyterPassword)
	out, err := exec.Command(pythonPath, "-c", hashCmd).Output()
	passwordHash := strings.TrimSpace(string(out))
	if err != nil || passwordHash == "" {
		hashCmd = fmt.Sprintf(`from notebook.auth import passwd; print(passwd('%s'))`, config.JupyterPassword)
		out, _ = exec.Command(pythonPath, "-c", hashCmd).Output()
		passwordHash = strings.TrimSpace(string(out))
	}

	if passwordHash == "" {
		passwordHash = "''"
	}

	// Create config
	jupyterConfig := fmt.Sprintf(`# CloudLab Jupyter Configuration
# Author: %s
# GitHub: %s

c = get_config()

# Server settings
c.ServerApp.ip = '0.0.0.0'
c.ServerApp.port = %d
c.ServerApp.open_browser = False
c.ServerApp.allow_root = True
c.ServerApp.allow_origin = '*'
c.ServerApp.root_dir = '%s'
c.ServerApp.password = '%s'
c.ServerApp.token = ''

# Notebook settings (legacy)
c.NotebookApp.ip = '0.0.0.0'
c.NotebookApp.port = %d
c.NotebookApp.open_browser = False
c.NotebookApp.allow_root = True
c.NotebookApp.allow_origin = '*'
c.NotebookApp.notebook_dir = '%s'
c.NotebookApp.password = '%s'
c.NotebookApp.token = ''

# Performance settings
c.MappingKernelManager.cull_idle_timeout = 3600
c.MappingKernelManager.cull_interval = 300
`, AUTHOR, GITHUB, config.JupyterPort, config.WorkingDirectory, passwordHash,
		config.JupyterPort, config.WorkingDirectory, passwordHash)

	os.WriteFile(filepath.Join(jupyterDir, "jupyter_lab_config.py"), []byte(jupyterConfig), 0644)
	os.WriteFile(filepath.Join(jupyterDir, "jupyter_notebook_config.py"), []byte(jupyterConfig), 0644)
	os.WriteFile(filepath.Join(jupyterDir, "jupyter_server_config.py"), []byte(jupyterConfig), 0644)
}

func installVSCode() {
	printStep("Installing VS Code Server (code-server)...")

	if _, err := exec.LookPath("code-server"); err == nil {
		printSuccess("code-server already installed ✓")
		configureVSCode()
		return
	}

	var cmd *exec.Cmd
	if runtime.GOOS == "windows" {
		cmd = exec.Command("npm", "install", "-g", "code-server")
	} else {
		cmd = exec.Command("bash", "-c", "curl -fsSL https://code-server.dev/install.sh | sh")
	}
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	if err := cmd.Run(); err != nil {
		printError("Failed to install code-server: " + err.Error())
		return
	}

	configureVSCode()
	printSuccess("VS Code Server installed ✓")
}

func configureVSCode() {
	fmt.Printf("  %s→%s Configuring VS Code Server...\n", Dim, Reset)

	configDir := filepath.Join(homeDir, ".config", "code-server")
	os.MkdirAll(configDir, 0755)

	vscodeConfig := fmt.Sprintf(`bind-addr: 0.0.0.0:%d
auth: password
password: %s
cert: false
user-data-dir: %s
`, config.VSCodePort, config.VSCodePassword, filepath.Join(cloudlabDir, "vscode-data"))

	os.WriteFile(filepath.Join(configDir, "config.yaml"), []byte(vscodeConfig), 0644)

	// Install common extensions
	codeServerPath, _ := exec.LookPath("code-server")
	if codeServerPath != "" {
		fmt.Printf("  %s→%s Installing VS Code extensions...\n", Dim, Reset)
		exec.Command(codeServerPath, "--install-extension", "ms-python.python").Run()
		exec.Command(codeServerPath, "--install-extension", "ms-toolsai.jupyter").Run()
	}
}

func installCloudflare() {
	printStep("Installing Cloudflared...")

	if _, err := exec.LookPath("cloudflared"); err == nil {
		printSuccess("cloudflared already installed ✓")
		return
	}

	switch runtime.GOOS {
	case "linux":
		var url string
		if runtime.GOARCH == "arm64" {
			url = "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-arm64"
		} else {
			url = "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64"
		}
		tmpFile := "/tmp/cloudflared"
		if err := downloadFile(tmpFile, url); err != nil {
			printError("Download failed: " + err.Error())
			return
		}
		os.Chmod(tmpFile, 0755)
		exec.Command("sudo", "mv", tmpFile, "/usr/local/bin/cloudflared").Run()

	case "darwin":
		cmd := exec.Command("brew", "install", "cloudflared")
		cmd.Stdout = os.Stdout
		cmd.Stderr = os.Stderr
		cmd.Run()

	case "windows":
		exec.Command("winget", "install", "--id", "Cloudflare.cloudflared", "-e").Run()
	}

	printSuccess("Cloudflared installed ✓")
}

// ==================== Service Management ====================

func startService(service string) {
	switch service {
	case "jupyter":
		startJupyter(config.JupyterMode)
	case "lab":
		startJupyter("lab")
	case "notebook":
		startJupyter("notebook")
	case "vscode":
		startVSCode()
	case "tunnel", "tunnels":
		startTunnels()
	case "all":
		startAll()
	default:
		printError("Unknown service: " + service)
		printInfo("Available: all, jupyter, lab, notebook, vscode, tunnel")
	}
}

func startAll() {
	printHeader("🚀 STARTING ALL SERVICES")
	fmt.Println()

	startJupyter(config.JupyterMode)
	startVSCode()
	startTunnels()

	fmt.Println()
	printSuccess("✅ All services started!")
}

func startJupyter(mode string) {
	if mode == "" {
		mode = config.JupyterMode
	}

	modeTitle := strings.ToUpper(mode[:1]) + mode[1:]
	printStep(fmt.Sprintf("Starting Jupyter %s...", modeTitle))

	jupyterPath := getJupyterPath()
	if _, err := os.Stat(jupyterPath); os.IsNotExist(err) {
		printError("Jupyter not found. Run 'cloudlab install jupyter' first.")
		return
	}

	// Stop existing
	stopByPID("jupyter")
	time.Sleep(1 * time.Second)

	// Environment variables
	env := os.Environ()
	if config.EnableMPS {
		env = append(env, "PYTORCH_ENABLE_MPS_FALLBACK=1")
	}
	if config.LowPowerMode {
		env = append(env, "OMP_NUM_THREADS=2", "MKL_NUM_THREADS=2")
	}

	// Build command
	var cmd *exec.Cmd
	if mode == "lab" {
		cmd = exec.Command(jupyterPath, "lab",
			"--no-browser",
			"--ip=0.0.0.0",
			fmt.Sprintf("--port=%d", config.JupyterPort),
			fmt.Sprintf("--notebook-dir=%s", config.WorkingDirectory),
			"--ServerApp.token=''",
			"--ServerApp.allow_origin='*'")
	} else {
		cmd = exec.Command(jupyterPath, "notebook",
			"--no-browser",
			"--ip=0.0.0.0",
			fmt.Sprintf("--port=%d", config.JupyterPort),
			fmt.Sprintf("--notebook-dir=%s", config.WorkingDirectory),
			"--NotebookApp.token=''",
			"--NotebookApp.allow_origin='*'")
	}

	cmd.Dir = config.WorkingDirectory
	cmd.Env = env

	// Log output
	logFile, _ := os.Create(filepath.Join(cloudlabDir, "logs", "jupyter.log"))
	cmd.Stdout = logFile
	cmd.Stderr = logFile

	if err := cmd.Start(); err != nil {
		printError("Failed to start Jupyter: " + err.Error())
		return
	}

	if cmd.Process != nil {
		savePID("jupyter", cmd.Process.Pid)
		fmt.Printf("  %s✓%s Jupyter %s started on port %s%d%s (PID: %d)\n",
			BrightGreen, Reset, modeTitle, BrightCyan, config.JupyterPort, Reset, cmd.Process.Pid)
		fmt.Printf("    %s→%s URL: %shttp://localhost:%d%s\n", Dim, Reset, BrightBlue+Underline, config.JupyterPort, Reset)
		fmt.Printf("    %s→%s Password: %s%s%s\n", Dim, Reset, BrightYellow, config.JupyterPassword, Reset)
		fmt.Printf("    %s→%s Directory: %s%s%s\n", Dim, Reset, Dim, config.WorkingDirectory, Reset)
	}
}

func startVSCode() {
	printStep("Starting VS Code Server...")

	codeServerPath, err := exec.LookPath("code-server")
	if err != nil {
		printError("code-server not found. Run 'cloudlab install vscode' first.")
		return
	}

	// Stop existing
	stopByPID("vscode")
	time.Sleep(1 * time.Second)

	cmd := exec.Command(codeServerPath,
		fmt.Sprintf("--bind-addr=0.0.0.0:%d", config.VSCodePort),
		config.WorkingDirectory)
	cmd.Dir = config.WorkingDirectory

	logFile, _ := os.Create(filepath.Join(cloudlabDir, "logs", "vscode.log"))
	cmd.Stdout = logFile
	cmd.Stderr = logFile

	if err := cmd.Start(); err != nil {
		printError("Failed to start VS Code: " + err.Error())
		return
	}

	if cmd.Process != nil {
		savePID("vscode", cmd.Process.Pid)
		fmt.Printf("  %s✓%s VS Code started on port %s%d%s (PID: %d)\n",
			BrightGreen, Reset, BrightCyan, config.VSCodePort, Reset, cmd.Process.Pid)
		fmt.Printf("    %s→%s URL: %shttp://localhost:%d%s\n", Dim, Reset, BrightBlue+Underline, config.VSCodePort, Reset)
		fmt.Printf("    %s→%s Password: %s%s%s\n", Dim, Reset, BrightYellow, config.VSCodePassword, Reset)
		fmt.Printf("    %s→%s Directory: %s%s%s\n", Dim, Reset, Dim, config.WorkingDirectory, Reset)
	}
}

func startTunnels() {
	printStep("Starting Cloudflare tunnels...")

	cloudflaredPath, err := exec.LookPath("cloudflared")
	if err != nil {
		printError("cloudflared not found. Run 'cloudlab install cloudflare' first.")
		return
	}

	// Stop existing tunnels
	stopByPID("tunnel_jupyter")
	stopByPID("tunnel_vscode")
	time.Sleep(1 * time.Second)

	// Start Jupyter tunnel
	go func() {
		logPath := filepath.Join(cloudlabDir, "logs", "tunnel_jupyter.log")
		logFile, _ := os.Create(logPath)
		cmd := exec.Command(cloudflaredPath, "tunnel", "--url", fmt.Sprintf("http://localhost:%d", config.JupyterPort))
		cmd.Stdout = logFile
		cmd.Stderr = logFile
		if err := cmd.Start(); err == nil && cmd.Process != nil {
			savePID("tunnel_jupyter", cmd.Process.Pid)
		}
		// Wait and extract URL
		time.Sleep(5 * time.Second)
		extractTunnelURL("jupyter", logPath)
	}()

	// Start VS Code tunnel
	go func() {
		logPath := filepath.Join(cloudlabDir, "logs", "tunnel_vscode.log")
		logFile, _ := os.Create(logPath)
		cmd := exec.Command(cloudflaredPath, "tunnel", "--url", fmt.Sprintf("http://localhost:%d", config.VSCodePort))
		cmd.Stdout = logFile
		cmd.Stderr = logFile
		if err := cmd.Start(); err == nil && cmd.Process != nil {
			savePID("tunnel_vscode", cmd.Process.Pid)
		}
		// Wait and extract URL
		time.Sleep(5 * time.Second)
		extractTunnelURL("vscode", logPath)
	}()

	fmt.Printf("  %s⏳%s Waiting for tunnel URLs...\n", BrightYellow, Reset)
	time.Sleep(10 * time.Second)

	// Show URLs
	loadConfig()
	showTunnelStatus()

	// Send email notification
	if config.NotifyOnStart && config.EmailAddress != "" && config.EmailAppPassword != "" {
		sendTunnelURLsEmail()
	}
}

func extractTunnelURL(service, logPath string) {
	for i := 0; i < 30; i++ {
		data, err := os.ReadFile(logPath)
		if err == nil {
			re := regexp.MustCompile(`https://[a-zA-Z0-9-]+\.trycloudflare\.com`)
			matches := re.FindAllString(string(data), -1)
			if len(matches) > 0 {
				url := matches[len(matches)-1]
				switch service {
				case "jupyter":
					config.TunnelURLs.Jupyter = url
				case "vscode":
					config.TunnelURLs.VSCode = url
				}
				saveConfig()
				return
			}
		}
		time.Sleep(1 * time.Second)
	}
}

func stopService(service string) {
	switch service {
	case "jupyter", "lab", "notebook":
		stopByPID("jupyter")
		printSuccess("Jupyter stopped")
	case "vscode":
		stopByPID("vscode")
		printSuccess("VS Code stopped")
	case "tunnel", "tunnels":
		stopTunnels()
	case "all":
		stopAll()
	default:
		printError("Unknown service: " + service)
	}
}

func stopAll() {
	printHeader("🛑 STOPPING ALL SERVICES")
	stopTunnels()
	stopByPID("jupyter")
	stopByPID("vscode")
	printSuccess("All services stopped!")
}

func stopTunnels() {
	stopByPID("tunnel_jupyter")
	stopByPID("tunnel_vscode")
	config.TunnelURLs = TunnelURLs{}
	saveConfig()
	printSuccess("Tunnels stopped")
}

// ==================== Status ====================

func showStatus() {
	fmt.Println(getLogo())

	printHeader("📊 SERVICE STATUS")

	// Jupyter
	if isRunning("jupyter") {
		modeTitle := strings.ToUpper(config.JupyterMode[:1]) + config.JupyterMode[1:]
		fmt.Printf("  %s●%s Jupyter %-8s %s[Running]%s port %s%d%s\n",
			BrightGreen, Reset, modeTitle, BrightGreen, Reset, BrightCyan, config.JupyterPort, Reset)
		fmt.Printf("    %s└─%s URL: %shttp://localhost:%d%s\n", Dim, Reset, BrightBlue+Underline, config.JupyterPort, Reset)
		fmt.Printf("    %s└─%s Dir: %s%s%s\n", Dim, Reset, Dim, config.WorkingDirectory, Reset)
	} else {
		fmt.Printf("  %s○%s Jupyter          %s[Stopped]%s\n", BrightRed, Reset, BrightRed, Reset)
	}

	// VS Code
	if isRunning("vscode") {
		fmt.Printf("  %s●%s VS Code          %s[Running]%s port %s%d%s\n",
			BrightGreen, Reset, BrightGreen, Reset, BrightCyan, config.VSCodePort, Reset)
		fmt.Printf("    %s└─%s URL: %shttp://localhost:%d%s\n", Dim, Reset, BrightBlue+Underline, config.VSCodePort, Reset)
	} else {
		fmt.Printf("  %s○%s VS Code          %s[Stopped]%s\n", BrightRed, Reset, BrightRed, Reset)
	}

	// Tunnels
	printHeader("🌐 TUNNEL STATUS")

	if isRunning("tunnel_jupyter") {
		fmt.Printf("  %s●%s Jupyter Tunnel   %s[Running]%s\n", BrightGreen, Reset, BrightGreen, Reset)
		if config.TunnelURLs.Jupyter != "" {
			fmt.Printf("    %s└─%s %s%s%s\n", Dim, Reset, BrightMagenta+Underline, config.TunnelURLs.Jupyter, Reset)
		}
	} else {
		fmt.Printf("  %s○%s Jupyter Tunnel   %s[Stopped]%s\n", BrightRed, Reset, BrightRed, Reset)
	}

	if isRunning("tunnel_vscode") {
		fmt.Printf("  %s●%s VS Code Tunnel   %s[Running]%s\n", BrightGreen, Reset, BrightGreen, Reset)
		if config.TunnelURLs.VSCode != "" {
			fmt.Printf("    %s└─%s %s%s%s\n", Dim, Reset, BrightMagenta+Underline, config.TunnelURLs.VSCode, Reset)
		}
	} else {
		fmt.Printf("  %s○%s VS Code Tunnel   %s[Stopped]%s\n", BrightRed, Reset, BrightRed, Reset)
	}

	// Credentials
	printHeader("🔐 CREDENTIALS")
	fmt.Printf("  %sJupyter Password:%s  %s%s%s\n", BrightYellow, Reset, BrightWhite, config.JupyterPassword, Reset)
	fmt.Printf("  %sVS Code Password:%s  %s%s%s\n", BrightYellow, Reset, BrightWhite, config.VSCodePassword, Reset)
	fmt.Println()
}

func showLogs(service string) {
	validServices := map[string]bool{
		"jupyter": true, "vscode": true,
		"tunnel_jupyter": true, "tunnel_vscode": true,
	}

	if !validServices[service] {
		printError("Unknown service: " + service)
		printInfo("Available: jupyter, vscode, tunnel_jupyter, tunnel_vscode")
		return
	}

	logPath := filepath.Join(cloudlabDir, "logs", service+".log")
	data, err := os.ReadFile(logPath)
	if err != nil {
		printError("Log file not found: " + logPath)
		return
	}

	fmt.Printf("\n%s%s=== Logs for %s ===%s\n\n", Bold, BrightCyan, service, Reset)
	fmt.Println(string(data))
}

// ==================== Tunnel Management ====================

func manageTunnel(action string) {
	switch action {
	case "start":
		startTunnels()
	case "stop":
		stopTunnels()
	case "restart":
		stopTunnels()
		time.Sleep(2 * time.Second)
		startTunnels()
	case "status":
		showTunnelStatus()
	default:
		printError("Unknown action: " + action)
		printInfo("Available: start, stop, restart, status")
	}
}

func showTunnelStatus() {
	loadConfig()

	printHeader("🌐 TUNNEL URLS")

	if config.TunnelURLs.Jupyter != "" {
		fmt.Printf("  %s🐍 Jupyter:%s  %s%s%s\n", BrightYellow, Reset, BrightMagenta+Underline, config.TunnelURLs.Jupyter, Reset)
	} else {
		fmt.Printf("  %s🐍 Jupyter:%s  %s(no tunnel)%s\n", BrightYellow, Reset, Dim, Reset)
	}

	if config.TunnelURLs.VSCode != "" {
		fmt.Printf("  %s💻 VS Code:%s  %s%s%s\n", BrightBlue, Reset, BrightMagenta+Underline, config.TunnelURLs.VSCode, Reset)
	} else {
		fmt.Printf("  %s💻 VS Code:%s  %s(no tunnel)%s\n", BrightBlue, Reset, Dim, Reset)
	}

	if config.TunnelURLs.Jupyter == "" && config.TunnelURLs.VSCode == "" {
		fmt.Printf("\n  %s💡 Run 'cloudlab tunnel start' to get public URLs%s\n", Dim, Reset)
	}
	fmt.Println()
}

// ==================== Kernel Management ====================

func manageKernel(args []string) {
	switch args[0] {
	case "list":
		listKernels()
	case "add":
		if len(args) < 2 {
			printError("Usage: cloudlab kernel add <name> [python_version]")
			return
		}
		pyVer := config.PythonVersion
		if len(args) > 2 {
			pyVer = args[2]
		}
		addKernel(args[1], pyVer)
	case "remove", "rm", "delete":
		if len(args) < 2 {
			printError("Usage: cloudlab kernel remove <name>")
			return
		}
		removeKernel(args[1])
	default:
		printError("Unknown action: " + args[0])
		printInfo("Available: list, add, remove")
	}
}

func listKernels() {
	printHeader("📓 JUPYTER KERNELS")

	jupyterPath := getJupyterPath()
	if _, err := os.Stat(jupyterPath); os.IsNotExist(err) {
		printError("Jupyter not installed. Run 'cloudlab install jupyter'")
		return
	}

	cmd := exec.Command(jupyterPath, "kernelspec", "list")
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	cmd.Run()
	fmt.Println()
}

func addKernel(name, pyVersion string) {
	printStep(fmt.Sprintf("Creating kernel '%s%s%s' with Python %s%s%s...", BrightCyan, name, Reset, BrightYellow, pyVersion, Reset))

	uvPath := getUVPath()
	if uvPath == "" {
		printError("UV not found. Run 'cloudlab install uv'")
		return
	}

	envPath := filepath.Join(cloudlabDir, "envs", name)

	// Create environment
	cmd := exec.Command(uvPath, "venv", envPath, "--python", pyVersion)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	if err := cmd.Run(); err != nil {
		printError("Failed to create environment: " + err.Error())
		return
	}

	// Get python path for this environment
	var pythonPath string
	if runtime.GOOS == "windows" {
		pythonPath = filepath.Join(envPath, "Scripts", "python.exe")
	} else {
		pythonPath = filepath.Join(envPath, "bin", "python")
	}

	// Install ipykernel
	fmt.Printf("  %s→%s Installing ipykernel...\n", Dim, Reset)
	exec.Command(uvPath, "pip", "install", "ipykernel", "--python", pythonPath).Run()

	// Register kernel
	fmt.Printf("  %s→%s Registering kernel...\n", Dim, Reset)
	exec.Command(pythonPath, "-m", "ipykernel", "install", "--user", "--name", name,
		"--display-name", fmt.Sprintf("Python %s (%s)", pyVersion, name)).Run()

	// Install PyTorch if GPU available
	if config.EnableMPS {
		fmt.Printf("  %s→%s Installing PyTorch with MPS...\n", Dim, Reset)
		exec.Command(uvPath, "pip", "install", "torch", "torchvision", "--python", pythonPath).Run()
	} else if config.EnableCUDA {
		fmt.Printf("  %s→%s Installing PyTorch with CUDA...\n", Dim, Reset)
		exec.Command(uvPath, "pip", "install", "torch", "torchvision",
			"--index-url", "https://download.pytorch.org/whl/cu121", "--python", pythonPath).Run()
	}

	printSuccess(fmt.Sprintf("Kernel '%s%s%s' created!", BrightCyan, name, Reset))
}

func removeKernel(name string) {
	printStep(fmt.Sprintf("Removing kernel '%s%s%s'...", BrightCyan, name, Reset))

	jupyterPath := getJupyterPath()
	if jupyterPath != "" {
		exec.Command(jupyterPath, "kernelspec", "uninstall", name, "-f").Run()
	}

	// Remove environment
	envPath := filepath.Join(cloudlabDir, "envs", name)
	os.RemoveAll(envPath)

	printSuccess(fmt.Sprintf("Kernel '%s%s%s' removed!", BrightCyan, name, Reset))
}

// ==================== Environment Management ====================

func manageEnvironment(args []string) {
	switch args[0] {
	case "list":
		listEnvironments()
	case "create":
		if len(args) < 3 {
			printError("Usage: cloudlab env create <name> <python_version>")
			return
		}
		createEnvironment(args[1], args[2])
	case "remove", "rm", "delete":
		if len(args) < 2 {
			printError("Usage: cloudlab env remove <name>")
			return
		}
		removeEnvironment(args[1])
	case "activate":
		if len(args) < 2 {
			printError("Usage: cloudlab env activate <name>")
			return
		}
		showActivateCommand(args[1])
	case "install":
		if len(args) < 2 {
			printError("Usage: cloudlab env install <package>")
			return
		}
		installPackage(strings.Join(args[1:], " "))
	default:
		printError("Unknown action: " + args[0])
		printInfo("Available: list, create, remove, activate, install")
	}
}

func listEnvironments() {
	printHeader("🐍 PYTHON ENVIRONMENTS")

	// Default environment
	venvPath := filepath.Join(cloudlabDir, "venv")
	if _, err := os.Stat(venvPath); err == nil {
		fmt.Printf("  %s★%s %scloudlab%s (default) - %s%s%s\n", BrightYellow, Reset, BrightCyan, Reset, Dim, venvPath, Reset)
	}

	// Additional environments
	envsDir := filepath.Join(cloudlabDir, "envs")
	entries, _ := os.ReadDir(envsDir)
	for _, e := range entries {
		if e.IsDir() {
			fmt.Printf("  %s○%s %s%s%s - %s%s%s\n", Dim, Reset, BrightCyan, e.Name(), Reset, Dim, filepath.Join(envsDir, e.Name()), Reset)
		}
	}

	fmt.Println()
}

func createEnvironment(name, pyVersion string) {
	printStep(fmt.Sprintf("Creating environment '%s%s%s' with Python %s%s%s...", BrightCyan, name, Reset, BrightYellow, pyVersion, Reset))

	uvPath := getUVPath()
	if uvPath == "" {
		printError("UV not found. Run 'cloudlab install uv'")
		return
	}

	envPath := filepath.Join(cloudlabDir, "envs", name)

	cmd := exec.Command(uvPath, "venv", envPath, "--python", pyVersion)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	if err := cmd.Run(); err != nil {
		printError("Failed to create environment: " + err.Error())
		return
	}

	printSuccess(fmt.Sprintf("Environment '%s%s%s' created!", BrightCyan, name, Reset))
	showActivateCommand(name)
}

func removeEnvironment(name string) {
	envPath := filepath.Join(cloudlabDir, "envs", name)
	if _, err := os.Stat(envPath); os.IsNotExist(err) {
		printError("Environment not found: " + name)
		return
	}

	os.RemoveAll(envPath)
	printSuccess(fmt.Sprintf("Environment '%s%s%s' removed!", BrightCyan, name, Reset))
}

func showActivateCommand(name string) {
	var envPath string
	if name == "cloudlab" || name == "default" {
		envPath = filepath.Join(cloudlabDir, "venv")
	} else {
		envPath = filepath.Join(cloudlabDir, "envs", name)
	}

	fmt.Printf("\n%s💡 To activate this environment:%s\n", Dim, Reset)
	if runtime.GOOS == "windows" {
		fmt.Printf("   %s$%s %s%s\\Scripts\\activate%s\n\n", BrightGreen, Reset, BrightCyan, envPath, Reset)
	} else {
		fmt.Printf("   %s$%s %ssource %s/bin/activate%s\n\n", BrightGreen, Reset, BrightCyan, envPath, Reset)
	}
}

func installPackage(pkg string) {
	printStep(fmt.Sprintf("Installing '%s%s%s' in default environment...", BrightYellow, pkg, Reset))

	uvPath := getUVPath()
	if uvPath == "" {
		printError("UV not found")
		return
	}

	pythonPath := getPythonPath()
	cmd := exec.Command(uvPath, "pip", "install", pkg, "--python", pythonPath)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr

	if err := cmd.Run(); err != nil {
		printError("Failed to install package: " + err.Error())
		return
	}

	printSuccess(fmt.Sprintf("Package '%s%s%s' installed!", BrightYellow, pkg, Reset))
}

// ==================== Email ====================

func manageEmail(args []string) {
	switch args[0] {
	case "setup":
		setupEmail()
	case "test":
		sendTestEmail()
	case "send":
		sendTunnelURLsEmail()
	default:
		printError("Unknown action: " + args[0])
		printInfo("Available: setup, test, send")
	}
}

func showEmailConfig() {
	printHeader("📧 EMAIL CONFIGURATION")

	if config.EmailAddress != "" {
		fmt.Printf("  %sEmail:%s    %s%s%s\n", BrightYellow, Reset, BrightMagenta, config.EmailAddress, Reset)
		fmt.Printf("  %sSMTP:%s     %s%s:%d%s\n", BrightYellow, Reset, Dim, config.SMTPServer, config.SMTPPort, Reset)
		if config.EmailAppPassword != "" {
			fmt.Printf("  %sPassword:%s %s********%s\n", BrightYellow, Reset, Dim, Reset)
		} else {
			fmt.Printf("  %sPassword:%s %s(not set)%s\n", BrightYellow, Reset, BrightRed, Reset)
		}
	} else {
		printWarning("Email not configured. Run 'cloudlab email setup'")
	}
	fmt.Println()
}

func setupEmail() {
	printHeader("📧 EMAIL SETUP")

	reader := bufio.NewReader(os.Stdin)

	fmt.Printf("  %sEmail address:%s ", BrightYellow, Reset)
	email, _ := reader.ReadString('\n')
	email = strings.TrimSpace(email)
	if email == "" {
		printWarning("Email setup cancelled")
		return
	}

	config.EmailAddress = email
	detectSMTPSettings(email)

	fmt.Printf("  %sApp password:%s ", BrightYellow, Reset)
	pass, _ := reader.ReadString('\n')
	config.EmailAppPassword = strings.TrimSpace(pass)

	saveConfig()
	printSuccess("Email configured!")

	fmt.Printf("\n%sSend test email?%s [Y/n]: ", BrightWhite, Reset)
	answer, _ := reader.ReadString('\n')
	if strings.TrimSpace(strings.ToLower(answer)) != "n" {
		sendTestEmail()
	}
}

func sendTestEmail() {
	if config.EmailAddress == "" {
		printError("Email not configured. Run 'cloudlab email setup'")
		return
	}

	printStep("Sending test email to " + config.EmailAddress + "...")

	subject := "CloudLab - Test Email ✓"
	body := fmt.Sprintf(`<html>
<body style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; padding: 40px; background: #f5f5f5;">
<div style="max-width: 500px; margin: 0 auto; background: white; padding: 40px; border-radius: 16px; box-shadow: 0 4px 6px rgba(0,0,0,0.1);">
<h1 style="color: #7c3aed; margin: 0 0 20px 0;">☁️ CloudLab</h1>
<div style="background: #dcfce7; color: #166534; padding: 20px; border-radius: 12px; margin: 20px 0;">
<h2 style="margin: 0;">✅ Email Configuration Working!</h2>
</div>
<p style="color: #666; line-height: 1.6;">Your email notifications are set up correctly. You will receive tunnel URLs when you start CloudLab services.</p>
<hr style="border: none; border-top: 1px solid #e5e7eb; margin: 30px 0;">
<p style="color: #999; font-size: 12px; margin: 0;">
CloudLab CLI v%s<br>
Author: %s<br>
<a href="%s" style="color: #7c3aed;">%s</a>
</p>
</div>
</body>
</html>`, VERSION, AUTHOR, GITHUB, GITHUB)

	if err := sendEmail(subject, body); err != nil {
		printError("Failed to send email: " + err.Error())
		return
	}

	printSuccess("Test email sent! ✓")
}

func sendTunnelURLsEmail() {
	if config.EmailAddress == "" {
		printWarning("Email not configured. Run 'cloudlab email setup'")
		return
	}

	loadConfig()

	if config.TunnelURLs.Jupyter == "" && config.TunnelURLs.VSCode == "" {
		printWarning("No tunnel URLs available. Run 'cloudlab tunnel start'")
		return
	}

	printStep("Sending tunnel URLs to " + config.EmailAddress + "...")

	hostname, _ := os.Hostname()

	subject := fmt.Sprintf("☁️ CloudLab URLs - %s", hostname)
	body := fmt.Sprintf(`<html>
<body style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; padding: 40px; background: #f5f5f5;">
<div style="max-width: 600px; margin: 0 auto; background: white; padding: 40px; border-radius: 16px; box-shadow: 0 4px 6px rgba(0,0,0,0.1);">
<h1 style="color: #7c3aed; margin: 0 0 10px 0;">☁️ CloudLab</h1>
<p style="color: #666; margin: 0 0 30px 0;">Remote Development Environment - %s</p>

<div style="background: linear-gradient(135deg, #fef3c7, #fde68a); padding: 24px; border-radius: 12px; margin: 20px 0;">
<h2 style="color: #92400e; margin: 0 0 12px 0;">🐍 Jupyter %s</h2>
<p style="margin: 8px 0;"><strong>URL:</strong> <a href="%s" style="color: #7c3aed; font-size: 14px;">%s</a></p>
<p style="margin: 8px 0;"><strong>Password:</strong> <code style="background: #fef3c7; padding: 4px 8px; border-radius: 4px; font-family: monospace;">%s</code></p>
</div>

<div style="background: linear-gradient(135deg, #dbeafe, #bfdbfe); padding: 24px; border-radius: 12px; margin: 20px 0;">
<h2 style="color: #1e40af; margin: 0 0 12px 0;">💻 VS Code</h2>
<p style="margin: 8px 0;"><strong>URL:</strong> <a href="%s" style="color: #7c3aed; font-size: 14px;">%s</a></p>
<p style="margin: 8px 0;"><strong>Password:</strong> <code style="background: #dbeafe; padding: 4px 8px; border-radius: 4px; font-family: monospace;">%s</code></p>
</div>

<div style="background: #f3e8ff; padding: 20px; border-radius: 12px; margin: 20px 0;">
<h3 style="color: #7c3aed; margin: 0 0 8px 0;">📁 Working Directory</h3>
<code style="color: #6b21a8; font-family: monospace;">%s</code>
</div>

<hr style="border: none; border-top: 1px solid #e5e7eb; margin: 30px 0;">
<p style="color: #999; font-size: 12px; margin: 0;">
CloudLab CLI v%s | %s<br>
Author: %s | <a href="%s" style="color: #7c3aed;">GitHub</a>
</p>
</div>
</body>
</html>`,
		hostname,
		strings.ToUpper(config.JupyterMode[:1])+config.JupyterMode[1:],
		config.TunnelURLs.Jupyter, config.TunnelURLs.Jupyter, config.JupyterPassword,
		config.TunnelURLs.VSCode, config.TunnelURLs.VSCode, config.VSCodePassword,
		config.WorkingDirectory,
		VERSION, time.Now().Format("2006-01-02 15:04:05"),
		AUTHOR, GITHUB)

	if err := sendEmail(subject, body); err != nil {
		printError("Failed to send email: " + err.Error())
		return
	}

	printSuccess("Tunnel URLs sent to " + config.EmailAddress + " ✓")
}

func sendEmail(subject, body string) error {
	from := config.EmailAddress
	to := config.EmailAddress
	password := config.EmailAppPassword

	headers := fmt.Sprintf("From: CloudLab <%s>\r\nTo: %s\r\nSubject: %s\r\nMIME-Version: 1.0\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n",
		from, to, subject)

	message := headers + body
	addr := fmt.Sprintf("%s:%d", config.SMTPServer, config.SMTPPort)

	// Connect
	client, err := smtp.Dial(addr)
	if err != nil {
		return fmt.Errorf("connection failed: %v", err)
	}
	defer client.Close()

	// TLS
	tlsConfig := &tls.Config{ServerName: config.SMTPServer}
	if err := client.StartTLS(tlsConfig); err != nil {
		return fmt.Errorf("TLS failed: %v", err)
	}

	// Auth
	auth := smtp.PlainAuth("", from, password, config.SMTPServer)
	if err := client.Auth(auth); err != nil {
		return fmt.Errorf("auth failed: %v", err)
	}

	// Send
	if err := client.Mail(from); err != nil {
		return err
	}
	if err := client.Rcpt(to); err != nil {
		return err
	}

	w, err := client.Data()
	if err != nil {
		return err
	}
	w.Write([]byte(message))
	return w.Close()
}

// ==================== Update & Uninstall ====================

func updateComponents() {
	printHeader("🔄 UPDATING COMPONENTS")

	uvPath := getUVPath()
	if uvPath != "" {
		printStep("Updating Python packages...")
		pythonPath := getPythonPath()
		exec.Command(uvPath, "pip", "install", "--upgrade", "jupyterlab", "notebook", "--python", pythonPath).Run()
	}

	printSuccess("Update complete! ✓")
}

func uninstallAll() {
	reader := bufio.NewReader(os.Stdin)
	fmt.Printf("\n%s⚠️  Are you sure you want to uninstall CloudLab?%s [y/N]: ", BrightRed, Reset)
	answer, _ := reader.ReadString('\n')

	if strings.TrimSpace(strings.ToLower(answer)) != "y" {
		printInfo("Uninstall cancelled")
		return
	}

	printStep("Stopping services...")
	stopAll()

	printStep("Removing CloudLab directory...")
	os.RemoveAll(cloudlabDir)
	os.RemoveAll(filepath.Join(homeDir, ".jupyter"))
	os.RemoveAll(filepath.Join(homeDir, ".config", "code-server"))

	printSuccess("CloudLab uninstalled! ✓")
	fmt.Printf("%sNote: UV, code-server, and cloudflared were not removed.%s\n", Dim, Reset)
}

// ==================== Utility Functions ====================

func downloadFile(filepath string, url string) error {
	resp, err := http.Get(url)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	out, err := os.Create(filepath)
	if err != nil {
		return err
	}
	defer out.Close()

	_, err = io.Copy(out, resp.Body)
	return err
}

func generateSecureToken(length int) string {
	bytes := make([]byte, length)
	rand.Read(bytes)
	return hex.EncodeToString(bytes)[:length]
}

func checkNVIDIAGPU() bool {
	_, err := exec.LookPath("nvidia-smi")
	return err == nil
}

func savePID(service string, pid int) {
	pidFile := filepath.Join(cloudlabDir, "pids", service+".pid")
	os.WriteFile(pidFile, []byte(strconv.Itoa(pid)), 0644)
}

func getPID(service string) int {
	pidFile := filepath.Join(cloudlabDir, "pids", service+".pid")
	data, err := os.ReadFile(pidFile)
	if err != nil {
		return 0
	}
	pid, _ := strconv.Atoi(strings.TrimSpace(string(data)))
	return pid
}

func stopByPID(service string) {
	pid := getPID(service)
	if pid == 0 {
		return
	}

	if runtime.GOOS == "windows" {
		exec.Command("taskkill", "/F", "/PID", strconv.Itoa(pid)).Run()
	} else {
		process, err := os.FindProcess(pid)
		if err == nil {
			process.Signal(syscall.SIGTERM)
			time.Sleep(500 * time.Millisecond)
			process.Kill()
		}
	}

	os.Remove(filepath.Join(cloudlabDir, "pids", service+".pid"))
}

func isRunning(service string) bool {
	pid := getPID(service)
	if pid == 0 {
		return false
	}

	if runtime.GOOS == "windows" {
		cmd := exec.Command("tasklist", "/FI", fmt.Sprintf("PID eq %d", pid), "/NH")
		output, _ := cmd.Output()
		return strings.Contains(string(output), strconv.Itoa(pid))
	}

	process, err := os.FindProcess(pid)
	if err != nil {
		return false
	}

	err = process.Signal(syscall.Signal(0))
	return err == nil
}

// ==================== Print Helpers ====================

func printDivider() {
	fmt.Printf("%s%s%s\n", Dim, strings.Repeat("─", 50), Reset)
}

func printHeader(title string) {
	fmt.Printf("\n%s%s%s\n", Bold+BrightWhite, title, Reset)
	printDivider()
}

func printStep(msg string) {
	fmt.Printf("  %s▶%s %s\n", BrightBlue, Reset, msg)
}

func printInfo(msg string) {
	fmt.Printf("  %s💡%s %s\n", BrightBlue, Reset, msg)
}

func printSuccess(msg string) {
	fmt.Printf("  %s✓%s %s\n", BrightGreen, Reset, msg)
}

func printError(msg string) {
	fmt.Printf("  %s✗%s %s\n", BrightRed, Reset, msg)
}

func printWarning(msg string) {
	fmt.Printf("  %s⚠%s %s\n", BrightYellow, Reset, msg)
}

func printBox(title string, lines []string) {
	width := len(title) + 4
	for _, l := range lines {
		if len(l)+4 > width {
			width = len(l) + 4
		}
	}

	fmt.Printf("  %s╭%s%s╮%s\n", BrightCyan, strings.Repeat("─", width), "─", Reset)
	fmt.Printf("  %s│%s %s%s%s%s │%s\n", BrightCyan, Reset, Bold+BrightWhite, title, strings.Repeat(" ", width-len(title)-1), BrightCyan, Reset)
	fmt.Printf("  %s├%s%s┤%s\n", BrightCyan, strings.Repeat("─", width), "─", Reset)
	for _, l := range lines {
		fmt.Printf("  %s│%s %s%s%s │%s\n", BrightCyan, Reset, l, strings.Repeat(" ", width-len(l)-1), BrightCyan, Reset)
	}
	fmt.Printf("  %s╰%s%s╯%s\n", BrightCyan, strings.Repeat("─", width), "─", Reset)
}
