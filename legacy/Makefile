# CloudLab Makefile

BINARY := cloudlab
BUILD_DIR := build

.PHONY: all build clean install

all: build

build:
	@echo "Building CloudLab..."
	@mkdir -p $(BUILD_DIR)
	@go mod init cloudlab 2>/dev/null || true
	@go get golang.org/x/text/cases golang.org/x/text/language 2>/dev/null || true
	@go mod tidy
	@CGO_ENABLED=0 go build -ldflags="-s -w" -o $(BUILD_DIR)/$(BINARY) cloudlab.go
	@echo "Done: $(BUILD_DIR)/$(BINARY)"

install: build
	@sudo cp $(BUILD_DIR)/$(BINARY) /usr/local/bin/
	@sudo chmod +x /usr/local/bin/$(BINARY)
	@echo "Installed to /usr/local/bin/$(BINARY)"

clean:
	@rm -rf $(BUILD_DIR) go.sum

cross:
	@mkdir -p $(BUILD_DIR)
	GOOS=linux GOARCH=amd64 go build -ldflags="-s -w" -o $(BUILD_DIR)/$(BINARY)-linux-amd64 cloudlab.go
	GOOS=linux GOARCH=arm64 go build -ldflags="-s -w" -o $(BUILD_DIR)/$(BINARY)-linux-arm64 cloudlab.go
	GOOS=darwin GOARCH=amd64 go build -ldflags="-s -w" -o $(BUILD_DIR)/$(BINARY)-darwin-amd64 cloudlab.go
	GOOS=darwin GOARCH=arm64 go build -ldflags="-s -w" -o $(BUILD_DIR)/$(BINARY)-darwin-arm64 cloudlab.go
	GOOS=windows GOARCH=amd64 go build -ldflags="-s -w" -o $(BUILD_DIR)/$(BINARY)-windows.exe cloudlab.go
