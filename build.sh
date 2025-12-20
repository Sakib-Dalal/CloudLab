#!/bin/bash
# CloudLab CLI Build Script
# Author: Sakib Dalal
# GitHub: https://github.com/Sakib-Dalal

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
YELLOW='\033[0;33m'
MAGENTA='\033[0;35m'
BOLD='\033[1m'
DIM='\033[2m'
NC='\033[0m'

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
echo -e "${BOLD}  ☁️  CloudLab CLI Builder v1.2.0${NC}"
echo -e "${DIM}  Author: Sakib Dalal${NC}"
echo -e "${BLUE}  GitHub: https://github.com/Sakib-Dalal${NC}"
echo ""
echo -e "${DIM}══════════════════════════════════════════════════${NC}"
echo ""

# Check Go
if ! command -v go &> /dev/null; then
    echo -e "${RED}Go not found. Please install Go first.${NC}"
    echo -e "${YELLOW}macOS: brew install go${NC}"
    echo -e "${YELLOW}Linux: sudo apt install golang${NC}"
    exit 1
fi

# Clean
rm -rf build go.sum 2>/dev/null || true
mkdir -p build

# Initialize module
echo -e "${BLUE}[1/4]${NC} Initializing Go module..."
if [ ! -f "go.mod" ]; then
    go mod init cloudlab 2>/dev/null || true
fi
go mod tidy 2>/dev/null || true
echo -e "${GREEN}  ✓${NC} Module initialized"

# Build
echo -e "${BLUE}[2/4]${NC} Building optimized binary..."
CGO_ENABLED=0 go build -ldflags="-s -w" -o build/cloudlab cloudlab.go

if [ ! -f "build/cloudlab" ]; then
    echo -e "${RED}  ✗ Build failed!${NC}"
    exit 1
fi

SIZE=$(ls -lh build/cloudlab | awk '{print $5}')
echo -e "${GREEN}  ✓${NC} Built: ${CYAN}build/cloudlab${NC} (${SIZE})"

# Install
echo ""
echo -e "${DIM}══════════════════════════════════════════════════${NC}"
echo ""
echo -e "${YELLOW}Install to /usr/local/bin?${NC} [Y/n]: \c"
read -r install

if [[ ! "$install" =~ ^[Nn]$ ]]; then
    echo -e "${BLUE}[3/4]${NC} Installing..."
    sudo cp build/cloudlab /usr/local/bin/
    sudo chmod +x /usr/local/bin/cloudlab
    echo -e "${GREEN}  ✓${NC} Installed to ${CYAN}/usr/local/bin/cloudlab${NC}"
else
    echo -e "${BLUE}[3/4]${NC} Skipping installation"
fi

# Copy dashboard files
echo -e "${BLUE}[4/4]${NC} Setting up dashboard..."
mkdir -p ~/.cloudlab
cp index.html ~/.cloudlab/dashboard.html 2>/dev/null || true
cp server.py ~/.cloudlab/server.py 2>/dev/null || true
chmod +x ~/.cloudlab/server.py 2>/dev/null || true
echo -e "${GREEN}  ✓${NC} Dashboard files copied"

# Done
echo ""
echo -e "${DIM}══════════════════════════════════════════════════${NC}"
echo ""
echo -e "${GREEN}${BOLD}✅ Build Complete!${NC}"
echo ""
echo -e "${BOLD}Quick Start:${NC}"
echo -e "  ${MAGENTA}\$${NC} cloudlab init         ${DIM}# Configure${NC}"
echo -e "  ${MAGENTA}\$${NC} cloudlab install all  ${DIM}# Install components${NC}"
echo -e "  ${MAGENTA}\$${NC} cloudlab start all    ${DIM}# Start all services${NC}"
echo -e "  ${MAGENTA}\$${NC} cloudlab status       ${DIM}# Check status${NC}"
echo ""
echo -e "${BOLD}Dashboard:${NC}"
echo -e "  ${MAGENTA}\$${NC} cloudlab start dashboard"
echo -e "  ${DIM}Open http://localhost:3000${NC}"
echo ""
echo -e "${BOLD}Services (all get tunnel URLs):${NC}"
echo -e "  🐍 Jupyter Lab/Notebook  ${DIM}(port 8888)${NC}"
echo -e "  💻 VS Code Server        ${DIM}(port 8080)${NC}"
echo -e "  🔒 SSH Terminal          ${DIM}(port 7681)${NC}"
echo -e "  📊 Web Dashboard         ${DIM}(port 3000)${NC}"
echo ""
echo -e "${BOLD}Email URLs:${NC}"
echo -e "  ${MAGENTA}\$${NC} cloudlab email send   ${DIM}# All 4 URLs sent to email${NC}"
echo ""
echo -e "${BOLD}Help:${NC}"
echo -e "  ${MAGENTA}\$${NC} cloudlab help"
echo ""
