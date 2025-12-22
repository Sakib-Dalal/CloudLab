# CloudLab CLI Installer for Windows
# Author: Sakib Dalal
# GitHub: https://github.com/Sakib-Dalal
# Supports: PowerShell 5.1+, Windows 10/11
# Run as Administrator: .\install.ps1

param(
    [switch]$Uninstall,
    [switch]$SkipGo,
    [switch]$SkipDeps,
    [switch]$Force,
    [switch]$Help
)

# Configuration
$VERSION = "1.2.0"
$AUTHOR = "Sakib Dalal"
$GITHUB = "https://github.com/Sakib-Dalal"
$INSTALL_DIR = "$env:LOCALAPPDATA\CloudLab"
$BIN_DIR = "$env:LOCALAPPDATA\CloudLab\bin"
$BINARY_NAME = "cloudlab.exe"
$GO_VERSION = "1.21.6"

# Colors using Write-Host
function Write-Color {
    param([string]$Text, [string]$Color = "White")
    Write-Host $Text -ForegroundColor $Color -NoNewline
}

function Write-ColorLine {
    param([string]$Text, [string]$Color = "White")
    Write-Host $Text -ForegroundColor $Color
}

function Show-Banner {
    Write-Host ""
    Write-ColorLine "   _____ _                 _ _           _     " "Cyan"
    Write-ColorLine "  / ____| |               | | |         | |    " "Cyan"
    Write-ColorLine " | |    | | ___  _   _  __| | |     __ _| |__  " "Blue"
    Write-ColorLine " | |    | |/ _ \| | | |/ _' | |    / _' | '_ \ " "Blue"
    Write-ColorLine " | |____| | (_) | |_| | (_| | |___| (_| | |_) |" "Magenta"
    Write-ColorLine "  \_____|_|\___/ \__,_|\__,_|______\__,_|_.__/ " "Magenta"
    Write-Host ""
    Write-Color "  ☁️  CloudLab CLI Installer " "White"
    Write-ColorLine "v$VERSION" "Green"
    Write-Color "  👤 Author: " "DarkGray"
    Write-ColorLine $AUTHOR "Yellow"
    Write-Color "  🔗 GitHub: " "DarkGray"
    Write-ColorLine $GITHUB "Blue"
    Write-Host ""
}

function Show-Help {
    Show-Banner
    Write-Host ""
    Write-ColorLine "USAGE:" "White"
    Write-Host "  .\install.ps1 [options]"
    Write-Host ""
    Write-ColorLine "OPTIONS:" "White"
    Write-Host "  -Help        Show this help message"
    Write-Host "  -Uninstall   Uninstall CloudLab"
    Write-Host "  -SkipGo      Skip Go installation"
    Write-Host "  -SkipDeps    Skip dependency installation"
    Write-Host "  -Force       Force reinstall"
    Write-Host ""
    Write-ColorLine "EXAMPLES:" "White"
    Write-Host "  .\install.ps1              # Full installation"
    Write-Host "  .\install.ps1 -SkipGo      # Skip Go if already installed"
    Write-Host "  .\install.ps1 -Uninstall   # Remove CloudLab"
    Write-Host ""
}

function Test-Administrator {
    $user = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($user)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Print-Step {
    param([string]$Message)
    Write-Color "  ▶ " "Blue"
    Write-ColorLine $Message "White"
}

function Print-Success {
    param([string]$Message)
    Write-Color "  ✓ " "Green"
    Write-ColorLine $Message "White"
}

function Print-Error {
    param([string]$Message)
    Write-Color "  ✗ " "Red"
    Write-ColorLine $Message "White"
}

function Print-Warning {
    param([string]$Message)
    Write-Color "  ⚠ " "Yellow"
    Write-ColorLine $Message "White"
}

function Print-Info {
    param([string]$Message)
    Write-Color "  💡 " "Cyan"
    Write-ColorLine $Message "White"
}

function Print-Header {
    param([string]$Title)
    Write-Host ""
    Write-ColorLine $Title "Cyan"
    Write-ColorLine ("─" * 50) "DarkGray"
}

function Test-Command {
    param([string]$Command)
    $null = Get-Command $Command -ErrorAction SilentlyContinue
    return $?
}

function Add-ToPath {
    param([string]$PathToAdd)
    
    $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($currentPath -notlike "*$PathToAdd*") {
        [Environment]::SetEnvironmentVariable(
            "Path",
            "$currentPath;$PathToAdd",
            "User"
        )
        $env:Path = "$env:Path;$PathToAdd"
        Print-Success "Added to PATH: $PathToAdd"
    }
}

function Install-Go {
    Print-Header "📦 INSTALLING GO"
    
    if (Test-Command "go") {
        $goVer = (go version) -replace "go version ", ""
        Print-Success "Go already installed: $goVer"
        return $true
    }
    
    Print-Step "Downloading Go $GO_VERSION..."
    
    $goUrl = "https://go.dev/dl/go$GO_VERSION.windows-amd64.msi"
    $goInstaller = "$env:TEMP\go-installer.msi"
    
    try {
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        Invoke-WebRequest -Uri $goUrl -OutFile $goInstaller -UseBasicParsing
        
        Print-Step "Installing Go (this may take a moment)..."
        Start-Process msiexec.exe -ArgumentList "/i", $goInstaller, "/quiet", "/norestart" -Wait
        
        Remove-Item $goInstaller -Force -ErrorAction SilentlyContinue
        
        # Refresh PATH
        $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + 
                    [System.Environment]::GetEnvironmentVariable("Path", "User")
        
        if (Test-Command "go") {
            Print-Success "Go $GO_VERSION installed!"
            return $true
        } else {
            # Add Go to path manually
            $goPath = "C:\Program Files\Go\bin"
            if (Test-Path $goPath) {
                Add-ToPath $goPath
                Print-Success "Go $GO_VERSION installed!"
                return $true
            }
        }
    } catch {
        Print-Error "Failed to install Go: $_"
        Print-Info "Please install Go manually from https://go.dev/dl/"
        return $false
    }
    
    return $false
}

function Install-UV {
    Print-Header "📦 INSTALLING UV PACKAGE MANAGER"
    
    if (Test-Command "uv") {
        Print-Success "UV already installed"
        return $true
    }
    
    Print-Step "Installing UV..."
    
    try {
        Invoke-Expression "& { $(Invoke-WebRequest -UseBasicParsing https://astral.sh/uv/install.ps1) }"
        
        # Add UV to PATH
        $uvPath = "$env:USERPROFILE\.cargo\bin"
        if (Test-Path $uvPath) {
            Add-ToPath $uvPath
        }
        
        $uvPath2 = "$env:LOCALAPPDATA\uv"
        if (Test-Path $uvPath2) {
            Add-ToPath $uvPath2
        }
        
        Print-Success "UV installed!"
        return $true
    } catch {
        Print-Warning "Failed to install UV via script, trying pip..."
        try {
            pip install uv
            Print-Success "UV installed via pip!"
            return $true
        } catch {
            Print-Error "Failed to install UV: $_"
            return $false
        }
    }
}

function Install-Cloudflared {
    Print-Header "📦 INSTALLING CLOUDFLARED"
    
    if (Test-Command "cloudflared") {
        Print-Success "Cloudflared already installed"
        return $true
    }
    
    Print-Step "Installing Cloudflared..."
    
    # Try winget first
    if (Test-Command "winget") {
        try {
            winget install --id Cloudflare.cloudflared -e --silent
            Print-Success "Cloudflared installed via winget!"
            return $true
        } catch {
            Print-Warning "winget install failed, trying direct download..."
        }
    }
    
    # Direct download
    try {
        $cfUrl = "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-windows-amd64.exe"
        $cfPath = "$BIN_DIR\cloudflared.exe"
        
        New-Item -ItemType Directory -Force -Path $BIN_DIR | Out-Null
        Invoke-WebRequest -Uri $cfUrl -OutFile $cfPath -UseBasicParsing
        
        Add-ToPath $BIN_DIR
        Print-Success "Cloudflared installed!"
        return $true
    } catch {
        Print-Error "Failed to install Cloudflared: $_"
        return $false
    }
}

function Install-CodeServer {
    Print-Header "📦 INSTALLING VS CODE SERVER"
    
    if (Test-Command "code-server") {
        Print-Success "code-server already installed"
        return $true
    }
    
    Print-Step "Installing code-server..."
    
    # Try npm
    if (Test-Command "npm") {
        try {
            npm install -g code-server
            Print-Success "code-server installed via npm!"
            return $true
        } catch {
            Print-Warning "npm install failed..."
        }
    }
    
    # Try winget
    if (Test-Command "winget") {
        try {
            winget install --id coder.code-server -e --silent
            Print-Success "code-server installed via winget!"
            return $true
        } catch {
            Print-Warning "winget install failed..."
        }
    }
    
    Print-Warning "Please install code-server manually:"
    Print-Info "npm install -g code-server"
    Print-Info "Or visit: https://github.com/coder/code-server/releases"
    return $false
}

function Install-TTYD {
    Print-Header "📦 INSTALLING WEB TERMINAL (TTYD)"
    
    $ttydPath = "$BIN_DIR\ttyd.exe"
    
    if ((Test-Command "ttyd") -or (Test-Path $ttydPath)) {
        Print-Success "ttyd already installed"
        return $true
    }
    
    Print-Step "Downloading ttyd..."
    
    try {
        $ttydUrl = "https://github.com/tsl0922/ttyd/releases/latest/download/ttyd.win32.exe"
        
        New-Item -ItemType Directory -Force -Path $BIN_DIR | Out-Null
        Invoke-WebRequest -Uri $ttydUrl -OutFile $ttydPath -UseBasicParsing
        
        Add-ToPath $BIN_DIR
        Print-Success "ttyd installed!"
        return $true
    } catch {
        Print-Error "Failed to install ttyd: $_"
        Print-Info "You can use WSL for SSH terminal instead"
        return $false
    }
}

function Build-CloudLab {
    Print-Header "🔨 BUILDING CLOUDLAB CLI"
    
    $scriptDir = Split-Path -Parent $MyInvocation.ScriptName
    if (-not $scriptDir) {
        $scriptDir = Get-Location
    }
    
    $sourceFile = Join-Path $scriptDir "cloudlab.go"
    
    if (-not (Test-Path $sourceFile)) {
        Print-Error "cloudlab.go not found in $scriptDir"
        return $false
    }
    
    Print-Step "Initializing Go module..."
    Push-Location $scriptDir
    
    try {
        # Initialize module
        if (-not (Test-Path "go.mod")) {
            go mod init cloudlab 2>$null
        }
        
        # Get dependencies
        go get golang.org/x/text/cases 2>$null
        go get golang.org/x/text/language 2>$null
        go mod tidy 2>$null
        
        Print-Step "Compiling CloudLab..."
        
        $env:CGO_ENABLED = "0"
        go build -ldflags="-s -w" -o "$BIN_DIR\$BINARY_NAME" cloudlab.go
        
        if (Test-Path "$BIN_DIR\$BINARY_NAME") {
            Add-ToPath $BIN_DIR
            
            $size = (Get-Item "$BIN_DIR\$BINARY_NAME").Length / 1MB
            Print-Success "Built: $BIN_DIR\$BINARY_NAME ({0:N2} MB)" -f $size
            return $true
        } else {
            Print-Error "Build failed!"
            return $false
        }
    } catch {
        Print-Error "Build failed: $_"
        return $false
    } finally {
        Pop-Location
    }
}

function Copy-DashboardFiles {
    Print-Header "📁 COPYING DASHBOARD FILES"
    
    $scriptDir = Split-Path -Parent $MyInvocation.ScriptName
    if (-not $scriptDir) {
        $scriptDir = Get-Location
    }
    
    $cloudlabDir = "$env:USERPROFILE\.cloudlab"
    New-Item -ItemType Directory -Force -Path $cloudlabDir | Out-Null
    New-Item -ItemType Directory -Force -Path "$cloudlabDir\logs" | Out-Null
    New-Item -ItemType Directory -Force -Path "$cloudlabDir\pids" | Out-Null
    New-Item -ItemType Directory -Force -Path "$cloudlabDir\envs" | Out-Null
    
    # Copy dashboard files
    $indexHtml = Join-Path $scriptDir "index.html"
    $serverPy = Join-Path $scriptDir "server.py"
    
    if (Test-Path $indexHtml) {
        Copy-Item $indexHtml "$cloudlabDir\dashboard.html" -Force
        Print-Success "Copied dashboard.html"
    }
    
    if (Test-Path $serverPy) {
        Copy-Item $serverPy "$cloudlabDir\server.py" -Force
        Print-Success "Copied server.py"
    }
    
    return $true
}

function Create-Shortcuts {
    Print-Header "🔗 CREATING SHORTCUTS"
    
    # Create Start Menu shortcut
    $startMenuPath = "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\CloudLab"
    New-Item -ItemType Directory -Force -Path $startMenuPath | Out-Null
    
    # Create a batch file to start all services
    $batchContent = @"
@echo off
title CloudLab Services
echo Starting CloudLab services...
cloudlab start all
echo.
echo CloudLab services started!
echo Press any key to open dashboard...
pause >nul
start http://localhost:3000
"@
    
    Set-Content -Path "$startMenuPath\Start CloudLab.bat" -Value $batchContent
    Print-Success "Created Start Menu shortcut"
    
    # Create Desktop shortcut
    $desktopPath = [Environment]::GetFolderPath("Desktop")
    $shortcutPath = "$desktopPath\CloudLab Dashboard.url"
    $shortcutContent = @"
[InternetShortcut]
URL=http://localhost:3000
IconFile=$BIN_DIR\cloudlab.exe
IconIndex=0
"@
    
    Set-Content -Path $shortcutPath -Value $shortcutContent
    Print-Success "Created Desktop shortcut"
}

function Register-ScheduledTask {
    Print-Header "⏰ REGISTERING SCHEDULED TASKS"
    
    # Remove existing tasks
    Unregister-ScheduledTask -TaskName "CloudLab-*" -Confirm:$false -ErrorAction SilentlyContinue
    
    # Create task to start CloudLab on login
    $action = New-ScheduledTaskAction -Execute "$BIN_DIR\$BINARY_NAME" -Argument "start all"
    $trigger = New-ScheduledTaskTrigger -AtLogOn -User $env:USERNAME
    $settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -StartWhenAvailable
    $principal = New-ScheduledTaskPrincipal -UserId $env:USERNAME -LogonType Interactive -RunLevel Limited
    
    try {
        Register-ScheduledTask -TaskName "CloudLab-AutoStart" -Action $action -Trigger $trigger -Settings $settings -Principal $principal -Description "Start CloudLab services on login" | Out-Null
        Print-Success "Created auto-start task (disabled by default)"
        Print-Info "To enable: schtasks /change /tn CloudLab-AutoStart /enable"
    } catch {
        Print-Warning "Could not create scheduled task: $_"
    }
}

function Uninstall-CloudLab {
    Print-Header "🗑️ UNINSTALLING CLOUDLAB"
    
    $confirm = Read-Host "Are you sure you want to uninstall CloudLab? [y/N]"
    if ($confirm -ne "y" -and $confirm -ne "Y") {
        Print-Info "Uninstall cancelled"
        return
    }
    
    Print-Step "Stopping services..."
    try {
        & "$BIN_DIR\$BINARY_NAME" stop all 2>$null
    } catch {}
    
    Print-Step "Removing scheduled tasks..."
    Unregister-ScheduledTask -TaskName "CloudLab-*" -Confirm:$false -ErrorAction SilentlyContinue
    
    Print-Step "Removing files..."
    Remove-Item -Recurse -Force "$env:USERPROFILE\.cloudlab" -ErrorAction SilentlyContinue
    Remove-Item -Recurse -Force $INSTALL_DIR -ErrorAction SilentlyContinue
    
    Print-Step "Removing shortcuts..."
    Remove-Item -Force "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\CloudLab" -Recurse -ErrorAction SilentlyContinue
    Remove-Item -Force ([Environment]::GetFolderPath("Desktop") + "\CloudLab Dashboard.url") -ErrorAction SilentlyContinue
    
    Print-Step "Removing from PATH..."
    $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
    $newPath = ($currentPath -split ";" | Where-Object { $_ -notlike "*CloudLab*" }) -join ";"
    [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
    
    Print-Success "CloudLab uninstalled!"
    Print-Info "Note: Go, UV, code-server, and cloudflared were not removed"
}

function Show-PostInstall {
    Print-Header "✅ INSTALLATION COMPLETE"
    
    Write-Host ""
    Write-ColorLine "  CloudLab CLI has been installed successfully!" "Green"
    Write-Host ""
    
    Write-ColorLine "  QUICK START:" "Cyan"
    Write-Host "  ─────────────────────────────────────────────"
    Write-Host ""
    Write-Color "    1. " "Yellow"
    Write-ColorLine "Open a NEW PowerShell window" "White"
    Write-Host ""
    Write-Color "    2. " "Yellow"
    Write-Host "Run: " -NoNewline
    Write-ColorLine "cloudlab init" "Cyan"
    Write-Host ""
    Write-Color "    3. " "Yellow"
    Write-Host "Run: " -NoNewline
    Write-ColorLine "cloudlab install all" "Cyan"
    Write-Host ""
    Write-Color "    4. " "Yellow"
    Write-Host "Run: " -NoNewline
    Write-ColorLine "cloudlab start all" "Cyan"
    Write-Host ""
    Write-Color "    5. " "Yellow"
    Write-Host "Open: " -NoNewline
    Write-ColorLine "http://localhost:3000" "Blue"
    Write-Host ""
    
    Write-ColorLine "  SERVICES:" "Cyan"
    Write-Host "  ─────────────────────────────────────────────"
    Write-Host "    🐍 Jupyter Lab/Notebook  - Port 8888"
    Write-Host "    💻 VS Code Server        - Port 8080"
    Write-Host "    🔒 SSH Terminal          - Port 7681"
    Write-Host "    📊 Web Dashboard         - Port 3000"
    Write-Host ""
    
    Write-ColorLine "  USEFUL COMMANDS:" "Cyan"
    Write-Host "  ─────────────────────────────────────────────"
    Write-Host "    cloudlab status         # Check all services"
    Write-Host "    cloudlab tunnel start   # Get public URLs"
    Write-Host "    cloudlab email send     # Email all URLs"
    Write-Host "    cloudlab help           # Full help"
    Write-Host ""
}

# Main execution
if ($Help) {
    Show-Help
    exit 0
}

Show-Banner

if ($Uninstall) {
    Uninstall-CloudLab
    exit 0
}

# Check if running as admin for certain operations
$isAdmin = Test-Administrator
if (-not $isAdmin) {
    Print-Warning "Not running as Administrator. Some features may be limited."
    Print-Info "For full installation, run PowerShell as Administrator"
    Write-Host ""
}

# Create directories
New-Item -ItemType Directory -Force -Path $INSTALL_DIR | Out-Null
New-Item -ItemType Directory -Force -Path $BIN_DIR | Out-Null

# Install components
if (-not $SkipGo) {
    Install-Go
}

if (-not $SkipDeps) {
    Install-UV
    Install-Cloudflared
    Install-CodeServer
    Install-TTYD
}

# Build CloudLab
$buildSuccess = Build-CloudLab

if ($buildSuccess) {
    Copy-DashboardFiles
    Create-Shortcuts
    
    if ($isAdmin) {
        Register-ScheduledTask
    }
    
    Show-PostInstall
} else {
    Print-Error "Installation failed. Please check the errors above."
    exit 1
}