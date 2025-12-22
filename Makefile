# CloudLab CLI Makefile
# Author: Sakib Dalal
# GitHub: https://github.com/Sakib-Dalal
# Supports: Linux, macOS, Windows (via MinGW/Git Bash)

BINARY := cloudlab
VERSION := 1.2.0
BUILD_DIR := build
INSTALL_DIR := /usr/local/bin

# Go parameters
GOCMD := go
GOBUILD := $(GOCMD) build
GOCLEAN := $(GOCMD) clean
GOMOD := $(GOCMD) mod
GOGET := $(GOCMD) get

# Build flags for smaller binary
LDFLAGS := -ldflags="-s -w"
CGO_ENABLED := 0

# Detect OS
ifeq ($(OS),Windows_NT)
    DETECTED_OS := windows
    BINARY_EXT := .exe
    INSTALL_DIR := $(LOCALAPPDATA)/CloudLab/bin
else
    DETECTED_OS := $(shell uname -s | tr '[:upper:]' '[:lower:]')
    BINARY_EXT :=
endif

# Detect ARCH
DETECTED_ARCH := $(shell uname -m)
ifeq ($(DETECTED_ARCH),x86_64)
    DETECTED_ARCH := amd64
endif
ifeq ($(DETECTED_ARCH),aarch64)
    DETECTED_ARCH := arm64
endif

.PHONY: all build clean install uninstall deps test run help
.PHONY: build-all build-linux build-darwin build-windows
.PHONY: docker docker-build docker-run docker-stop
.PHONY: release

# Default target
all: build

# Show help
help:
	@echo ""
	@echo "CloudLab CLI Makefile"
	@echo "Author: Sakib Dalal"
	@echo "GitHub: https://github.com/Sakib-Dalal"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "Build Targets:"
	@echo "  build          Build for current platform"
	@echo "  build-all      Build for all platforms"
	@echo "  build-linux    Build for Linux (amd64, arm64)"
	@echo "  build-darwin   Build for macOS (amd64, arm64)"
	@echo "  build-windows  Build for Windows (amd64)"
	@echo ""
	@echo "Install Targets:"
	@echo "  install        Install to system"
	@echo "  uninstall      Remove from system"
	@echo ""
	@echo "Docker Targets:"
	@echo "  docker         Build and run Docker container"
	@echo "  docker-build   Build Docker image"
	@echo "  docker-run     Run Docker container"
	@echo "  docker-stop    Stop Docker container"
	@echo ""
	@echo "Other Targets:"
	@echo "  deps           Install Go dependencies"
	@echo "  clean          Clean build artifacts"
	@echo "  test           Run tests"
	@echo "  release        Create release packages"
	@echo ""

# Install dependencies
deps:
	@echo "📦 Installing dependencies..."
	@$(GOMOD) init cloudlab 2>/dev/null || true
	@$(GOGET) golang.org/x/text/cases
	@$(GOGET) golang.org/x/text/language
	@$(GOMOD) tidy
	@echo "✓ Dependencies installed"

# Build for current platform
build: deps
	@echo "🔨 Building CloudLab CLI..."
	@mkdir -p $(BUILD_DIR)
	@CGO_ENABLED=$(CGO_ENABLED) $(GOBUILD) $(LDFLAGS) -o $(BUILD_DIR)/$(BINARY)$(BINARY_EXT) cloudlab.go
	@echo "✓ Built: $(BUILD_DIR)/$(BINARY)$(BINARY_EXT)"
	@ls -lh $(BUILD_DIR)/$(BINARY)$(BINARY_EXT)

# Clean build artifacts
clean:
	@echo "🧹 Cleaning..."
	@$(GOCLEAN)
	@rm -rf $(BUILD_DIR)
	@rm -f go.sum
	@echo "✓ Clean"

# Install to system
install: build
	@echo "📥 Installing to $(INSTALL_DIR)..."
ifeq ($(DETECTED_OS),windows)
	@mkdir -p "$(INSTALL_DIR)"
	@cp $(BUILD_DIR)/$(BINARY)$(BINARY_EXT) "$(INSTALL_DIR)/$(BINARY)$(BINARY_EXT)"
else
	@sudo cp $(BUILD_DIR)/$(BINARY) $(INSTALL_DIR)/$(BINARY)
	@sudo chmod +x $(INSTALL_DIR)/$(BINARY)
endif
	@echo "✓ Installed: $(INSTALL_DIR)/$(BINARY)$(BINARY_EXT)"
	@echo ""
	@echo "📁 Copying dashboard files..."
	@mkdir -p ~/.cloudlab
	@cp index.html ~/.cloudlab/dashboard.html 2>/dev/null || true
	@cp server.py ~/.cloudlab/server.py 2>/dev/null || true
	@chmod +x ~/.cloudlab/server.py 2>/dev/null || true
	@echo "✓ Dashboard files copied"

# Uninstall from system
uninstall:
	@echo "🗑️  Uninstalling..."
ifeq ($(DETECTED_OS),windows)
	@rm -f "$(INSTALL_DIR)/$(BINARY)$(BINARY_EXT)"
else
	@sudo rm -f $(INSTALL_DIR)/$(BINARY)
endif
	@echo "✓ Uninstalled"

# Run tests
test: build
	@echo "🧪 Testing..."
	@./$(BUILD_DIR)/$(BINARY)$(BINARY_EXT) version
	@./$(BUILD_DIR)/$(BINARY)$(BINARY_EXT) help
	@echo "✓ Tests passed"

# Run locally
run: build
	@./$(BUILD_DIR)/$(BINARY)$(BINARY_EXT) $(ARGS)

# Build for all platforms
build-all: deps build-linux build-darwin build-windows
	@echo ""
	@echo "✓ All platforms built!"
	@echo ""
	@ls -lh $(BUILD_DIR)/

# Build for Linux
build-linux: deps
	@echo "🐧 Building for Linux..."
	@mkdir -p $(BUILD_DIR)
	@GOOS=linux GOARCH=amd64 CGO_ENABLED=0 $(GOBUILD) $(LDFLAGS) -o $(BUILD_DIR)/$(BINARY)-linux-amd64 cloudlab.go
	@GOOS=linux GOARCH=arm64 CGO_ENABLED=0 $(GOBUILD) $(LDFLAGS) -o $(BUILD_DIR)/$(BINARY)-linux-arm64 cloudlab.go
	@echo "✓ Linux builds complete"

# Build for macOS
build-darwin: deps
	@echo "🍎 Building for macOS..."
	@mkdir -p $(BUILD_DIR)
	@GOOS=darwin GOARCH=amd64 CGO_ENABLED=0 $(GOBUILD) $(LDFLAGS) -o $(BUILD_DIR)/$(BINARY)-darwin-amd64 cloudlab.go
	@GOOS=darwin GOARCH=arm64 CGO_ENABLED=0 $(GOBUILD) $(LDFLAGS) -o $(BUILD_DIR)/$(BINARY)-darwin-arm64 cloudlab.go
	@echo "✓ macOS builds complete"

# Build for Windows
build-windows: deps
	@echo "🪟 Building for Windows..."
	@mkdir -p $(BUILD_DIR)
	@GOOS=windows GOARCH=amd64 CGO_ENABLED=0 $(GOBUILD) $(LDFLAGS) -o $(BUILD_DIR)/$(BINARY)-windows-amd64.exe cloudlab.go
	@echo "✓ Windows build complete"

# Docker targets
docker: docker-build docker-run

docker-build:
	@echo "🐳 Building Docker image..."
	@docker build -t cloudlab:latest .
	@echo "✓ Docker image built"

docker-run:
	@echo "🐳 Starting Docker container..."
	@docker run -d \
		--name cloudlab \
		-p 8888:8888 \
		-p 8080:8080 \
		-p 7681:7681 \
		-p 3000:3000 \
		-v $(PWD)/workspace:/home/cloudlab/workspace \
		cloudlab:latest
	@echo "✓ Container started"
	@echo ""
	@echo "Services:"
	@echo "  🐍 Jupyter:   http://localhost:8888"
	@echo "  💻 VS Code:   http://localhost:8080"
	@echo "  🔒 Terminal:  http://localhost:7681"
	@echo "  📊 Dashboard: http://localhost:3000"
	@echo ""
	@echo "Password: cloudlab"

docker-stop:
	@echo "🐳 Stopping Docker container..."
	@docker stop cloudlab 2>/dev/null || true
	@docker rm cloudlab 2>/dev/null || true
	@echo "✓ Container stopped"

docker-logs:
	@docker logs -f cloudlab

docker-shell:
	@docker exec -it cloudlab bash

# Docker Compose targets
compose-up:
	@echo "🐳 Starting with Docker Compose..."
	@docker-compose up -d
	@echo "✓ Services started"

compose-down:
	@docker-compose down

compose-logs:
	@docker-compose logs -f

# Create release packages
release: build-all
	@echo "📦 Creating release packages..."
	@mkdir -p $(BUILD_DIR)/release
	
	@# Linux AMD64
	@tar -czf $(BUILD_DIR)/release/cloudlab-$(VERSION)-linux-amd64.tar.gz \
		-C $(BUILD_DIR) $(BINARY)-linux-amd64 \
		-C .. README.md index.html server.py
	
	@# Linux ARM64
	@tar -czf $(BUILD_DIR)/release/cloudlab-$(VERSION)-linux-arm64.tar.gz \
		-C $(BUILD_DIR) $(BINARY)-linux-arm64 \
		-C .. README.md index.html server.py
	
	@# macOS AMD64
	@tar -czf $(BUILD_DIR)/release/cloudlab-$(VERSION)-darwin-amd64.tar.gz \
		-C $(BUILD_DIR) $(BINARY)-darwin-amd64 \
		-C .. README.md index.html server.py
	
	@# macOS ARM64
	@tar -czf $(BUILD_DIR)/release/cloudlab-$(VERSION)-darwin-arm64.tar.gz \
		-C $(BUILD_DIR) $(BINARY)-darwin-arm64 \
		-C .. README.md index.html server.py
	
	@# Windows AMD64
	@cd $(BUILD_DIR) && zip -q release/cloudlab-$(VERSION)-windows-amd64.zip $(BINARY)-windows-amd64.exe
	@zip -q $(BUILD_DIR)/release/cloudlab-$(VERSION)-windows-amd64.zip README.md index.html server.py install.ps1 install.bat
	
	@echo "✓ Release packages created"
	@ls -lh $(BUILD_DIR)/release/