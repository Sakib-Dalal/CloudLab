# CloudLab CLI Installer for Windows
# Requires: PowerShell 5.1 or later, Administrator privileges

param(
    [switch]$Uninstall,
    [switch]$Force
)

$ErrorActionPreference = "Stop"

# Configuration
$VERSION = "1.0.0"
$INSTALL_DIR = "$env:ProgramFiles\CloudLab"
$BINARY_NAME = "cloudlab.exe"

# Colors
function Write-ColorOutput($ForegroundColor) {
    $fc = $host.UI.RawUI.ForegroundColor
    $host.UI.RawUI.ForegroundColor = $ForegroundColor
    if ($args) {
        Write-Output $args
    }
    $host.UI.RawUI.ForegroundColor = $fc
}

function Print-Banner {
    Write-Host @"

   _____ _                 _ _           _     
  / ____| |               | | |         | |    
 | |    | | ___  _   _  __| | |     __ _| |__  
 | |    | |/ _ \| | | |/ _' | |    / _' | '_ \ 
 | |____| | (_) | |_| | (_| | |___| (_| | |_) |
  \_____|_|\___/ \__,_|\__,_|______\__,_|_.__/ 
                                               
  Self-Hosted Web Editor CLI Installer v$VERSION
  Windows Edition

"@ -ForegroundColor Cyan
}

function Print-Info($message) {
    Write-Host "[INFO] $message" -ForegroundColor Blue
}

function Print-Success($message) {
    Write-Host "[OK] $message" -ForegroundColor Green
}

function Print-Error($message) {
    Write-Host "[ERROR] $message" -ForegroundColor Red
}

function Test-Administrator {
    $user = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($user)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Install-Go {
    Print-Info "Installing Go..."
    
    $goVersion = "1.21.5"
    $goUrl = "https://go.dev/dl/go$goVersion.windows-amd64.msi"
    $goInstaller = "$env:TEMP\go.msi"
    
    Print-Info "Downloading Go $goVersion..."
    Invoke-WebRequest -Uri $goUrl -OutFile $goInstaller -UseBasicParsing
    
    Print-Info "Installing Go (this may take a moment)..."
    Start-Process msiexec.exe -ArgumentList "/i", $goInstaller, "/quiet", "/norestart" -Wait
    
    Remove-Item $goInstaller -Force
    
    # Refresh PATH
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")
    
    Print-Success "Go $goVersion installed!"
}

function Test-GoInstalled {
    try {
        $null = Get-Command go -ErrorAction Stop
        return $true
    } catch {
        return $false
    }
}

function Install-Chocolatey {
    if (-not (Get-Command choco -ErrorAction SilentlyContinue)) {
        Print-Info "Installing Chocolatey package manager..."
        Set-ExecutionPolicy Bypass -Scope Process -Force
        [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072
        Invoke-Expression ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))
        Print-Success "Chocolatey installed!"
    }
}

function Build-CloudLab {
    Print-Info "Building CloudLab CLI..."
    
    $buildDir = New-Item -ItemType Directory -Path "$env:TEMP\cloudlab-build-$(Get-Random)" -Force
    $scriptDir = Split-Path -Parent $MyInvocation.ScriptName
    
    # Copy source file
    if (Test-Path "$scriptDir\cloudlab.go") {
        Copy-Item "$scriptDir\cloudlab.go" -Destination $buildDir
    } else {
        Print-Error "cloudlab.go not found in script directory"
        return $false
    }
    
    Push-Location $buildDir
    
    try {
        # Initialize Go module
        & go mod init cloudlab
        
        # Build with optimizations
        Print-Info "Compiling with optimizations..."
        $env:CGO_ENABLED = "0"
        & go build -ldflags="-s -w" -o $BINARY_NAME cloudlab.go
        
        if (-not (Test-Path $BINARY_NAME)) {
            Print-Error "Build failed!"
            return $false
        }
        
        # Create install directory
        if (-not (Test-Path $INSTALL_DIR)) {
            New-Item -ItemType Directory -Path $INSTALL_DIR -Force | Out-Null
        }
        
        # Copy binary
        Copy-Item $BINARY_NAME -Destination "$INSTALL_DIR\$BINARY_NAME" -Force
        
        Print-Success "CloudLab CLI built and installed!"
        return $true
    } finally {
        Pop-Location
        Remove-Item $buildDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}

function Add-ToPath {
    Print-Info "Adding CloudLab to system PATH..."
    
    $currentPath = [Environment]::GetEnvironmentVariable("Path", "Machine")
    
    if ($currentPath -notlike "*$INSTALL_DIR*") {
        [Environment]::SetEnvironmentVariable(
            "Path",
            "$currentPath;$INSTALL_DIR",
            "Machine"
        )
        $env:Path = "$env:Path;$INSTALL_DIR"
        Print-Success "Added to PATH!"
    } else {
        Print-Info "Already in PATH"
    }
}

function Register-ScheduledTasks {
    Print-Info "Creating scheduled tasks..."
    
    # Jupyter Task
    $jupyterAction = New-ScheduledTaskAction -Execute "$INSTALL_DIR\$BINARY_NAME" -Argument "start jupyter"
    $jupyterTrigger = New-ScheduledTaskTrigger -AtLogOn
    $jupyterSettings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries
    
    Register-ScheduledTask -TaskName "CloudLab-Jupyter" -Action $jupyterAction -Trigger $jupyterTrigger -Settings $jupyterSettings -Description "CloudLab Jupyter Lab Server" -Force | Out-Null
    
    # VS Code Task
    $vscodeAction = New-ScheduledTaskAction -Execute "$INSTALL_DIR\$BINARY_NAME" -Argument "start vscode"
    
    Register-ScheduledTask -TaskName "CloudLab-VSCode" -Action $vscodeAction -Trigger $jupyterTrigger -Settings $jupyterSettings -Description "CloudLab VS Code Server" -Force | Out-Null
    
    # Tunnel Task
    $tunnelAction = New-ScheduledTaskAction -Execute "$INSTALL_DIR\$BINARY_NAME" -Argument "tunnel start"
    
    Register-ScheduledTask -TaskName "CloudLab-Tunnel" -Action $tunnelAction -Trigger $jupyterTrigger -Settings $jupyterSettings -Description "CloudLab Cloudflare Tunnel" -Force | Out-Null
    
    Print-Success "Scheduled tasks created!"
    Print-Info "Tasks are disabled by default. Enable in Task Scheduler if needed."
}

function Install-WindowsOpenSSH {
    Print-Info "Installing Windows OpenSSH Server..."
    
    # Check if already installed
    $sshCapability = Get-WindowsCapability -Online | Where-Object Name -like 'OpenSSH.Server*'
    
    if ($sshCapability.State -ne 'Installed') {
        Add-WindowsCapability -Online -Name OpenSSH.Server~~~~0.0.1.0
        Print-Success "OpenSSH Server installed!"
    } else {
        Print-Info "OpenSSH Server already installed"
    }
    
    # Configure SSH
    Start-Service sshd
    Set-Service -Name sshd -StartupType Automatic
    
    # Firewall rule
    $firewallRule = Get-NetFirewallRule -Name "OpenSSH-Server-In-TCP" -ErrorAction SilentlyContinue
    if (-not $firewallRule) {
        New-NetFirewallRule -Name "OpenSSH-Server-In-TCP" -DisplayName "OpenSSH Server (sshd)" -Enabled True -Direction Inbound -Protocol TCP -Action Allow -LocalPort 22 | Out-Null
    }
    
    Print-Success "SSH Server configured!"
}

function Uninstall-CloudLab {
    Print-Info "Uninstalling CloudLab..."
    
    # Stop services
    & "$INSTALL_DIR\$BINARY_NAME" stop all 2>$null
    
    # Remove scheduled tasks
    Unregister-ScheduledTask -TaskName "CloudLab-Jupyter" -Confirm:$false -ErrorAction SilentlyContinue
    Unregister-ScheduledTask -TaskName "CloudLab-VSCode" -Confirm:$false -ErrorAction SilentlyContinue
    Unregister-ScheduledTask -TaskName "CloudLab-Tunnel" -Confirm:$false -ErrorAction SilentlyContinue
    
    # Remove from PATH
    $currentPath = [Environment]::GetEnvironmentVariable("Path", "Machine")
    $newPath = ($currentPath.Split(';') | Where-Object { $_ -ne $INSTALL_DIR }) -join ';'
    [Environment]::SetEnvironmentVariable("Path", $newPath, "Machine")
    
    # Remove installation directory
    Remove-Item $INSTALL_DIR -Recurse -Force -ErrorAction SilentlyContinue
    
    # Remove config
    Remove-Item "$env:USERPROFILE\.cloudlab" -Recurse -Force -ErrorAction SilentlyContinue
    
    Print-Success "CloudLab uninstalled!"
}

function Main {
    Print-Banner
    
    # Check for admin privileges
    if (-not (Test-Administrator)) {
        Print-Error "This script requires Administrator privileges."
        Print-Info "Please run PowerShell as Administrator and try again."
        exit 1
    }
    
    if ($Uninstall) {
        Uninstall-CloudLab
        return
    }
    
    Print-Info "Starting CloudLab installation..."
    
    # Install Chocolatey if needed
    Install-Chocolatey
    
    # Install Go if needed
    if (-not (Test-GoInstalled)) {
        Install-Go
    } else {
        Print-Success "Go is already installed: $(go version)"
    }
    
    # Build and install CloudLab
    if (Build-CloudLab) {
        Add-ToPath
        Register-ScheduledTasks
        Install-WindowsOpenSSH
        
        Write-Host ""
        Print-Success "Installation complete!"
        Write-Host ""
        Write-Host "Quick Start:" -ForegroundColor Cyan
        Write-Host "  1. Open a new PowerShell window"
        Write-Host "  2. Run: cloudlab init"
        Write-Host "  3. Follow the interactive setup"
        Write-Host "  4. Run: cloudlab install all"
        Write-Host "  5. Run: cloudlab start all"
        Write-Host ""
        Write-Host "For help: cloudlab help" -ForegroundColor Cyan
        Write-Host ""
    } else {
        Print-Error "Installation failed!"
        exit 1
    }
}

Main
