# CloudLab CLI Docker Image
# Multi-stage build for minimal image size

# Build stage
FROM golang:1.21-alpine AS builder

WORKDIR /build

# Copy source
COPY cloudlab.go .

# Initialize module and build
RUN go mod init cloudlab && \
    CGO_ENABLED=0 GOOS=linux go build -ldflags="-s -w" -o cloudlab cloudlab.go

# Runtime stage
FROM ubuntu:22.04

LABEL maintainer="CloudLab Team"
LABEL version="1.0.0"
LABEL description="Self-hosted web editor with Jupyter Lab and VS Code"

# Avoid interactive prompts
ENV DEBIAN_FRONTEND=noninteractive
ENV TZ=UTC

# Install base dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    wget \
    git \
    openssh-server \
    ca-certificates \
    sudo \
    locales \
    && rm -rf /var/lib/apt/lists/* \
    && locale-gen en_US.UTF-8

ENV LANG=en_US.UTF-8
ENV LANGUAGE=en_US:en
ENV LC_ALL=en_US.UTF-8

# Copy CloudLab CLI from builder
COPY --from=builder /build/cloudlab /usr/local/bin/cloudlab
RUN chmod +x /usr/local/bin/cloudlab

# Create non-root user
RUN useradd -m -s /bin/bash cloudlab && \
    echo "cloudlab ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers

USER cloudlab
WORKDIR /home/cloudlab

# Install Miniconda
RUN curl -sL https://repo.anaconda.com/miniconda/Miniconda3-latest-Linux-x86_64.sh -o /tmp/miniconda.sh && \
    bash /tmp/miniconda.sh -b -p $HOME/miniconda3 && \
    rm /tmp/miniconda.sh

# Set up conda
ENV PATH="/home/cloudlab/miniconda3/bin:${PATH}"
RUN conda init bash && \
    conda config --set auto_activate_base false

# Create default environment with Jupyter
RUN conda create -n cloudlab python=3.11 -y && \
    conda install -n cloudlab -c conda-forge jupyterlab notebook ipykernel -y && \
    conda clean -afy

# Install code-server
USER root
RUN curl -fsSL https://code-server.dev/install.sh | sh
USER cloudlab

# Install cloudflared
USER root
RUN curl -L https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64 -o /usr/local/bin/cloudflared && \
    chmod +x /usr/local/bin/cloudflared
USER cloudlab

# Configure SSH
USER root
RUN mkdir -p /var/run/sshd && \
    sed -i 's/#PermitRootLogin.*/PermitRootLogin no/' /etc/ssh/sshd_config && \
    sed -i 's/#PasswordAuthentication.*/PasswordAuthentication yes/' /etc/ssh/sshd_config
USER cloudlab

# Create directories
RUN mkdir -p ~/.cloudlab ~/.jupyter ~/.config/code-server

# Set default configuration
RUN cloudlab config set jupyter_port 8888 && \
    cloudlab config set vscode_port 8080 && \
    cloudlab config set low_power_mode true

# Expose ports
EXPOSE 8888 8080 22

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD curl -f http://localhost:8888/api || exit 1

# Create entrypoint script
USER root
RUN echo '#!/bin/bash\n\
set -e\n\
\n\
# Start SSH if enabled\n\
if [ "${ENABLE_SSH:-false}" = "true" ]; then\n\
    sudo /usr/sbin/sshd\n\
    echo "SSH server started on port 22"\n\
fi\n\
\n\
# Start services\n\
echo "Starting CloudLab services..."\n\
\n\
# Start Jupyter in background\n\
source ~/miniconda3/etc/profile.d/conda.sh\n\
conda activate cloudlab\n\
jupyter lab --ip=0.0.0.0 --port=8888 --no-browser --NotebookApp.token="" --NotebookApp.password="" &\n\
echo "Jupyter Lab started on port 8888"\n\
\n\
# Start code-server in background\n\
code-server --bind-addr 0.0.0.0:8080 --auth none &\n\
echo "VS Code Server started on port 8080"\n\
\n\
# Start tunnel if token is provided\n\
if [ -n "${CLOUDFLARE_TOKEN}" ]; then\n\
    cloudflared tunnel run ${TUNNEL_NAME:-cloudlab} &\n\
    echo "Cloudflare tunnel started"\n\
fi\n\
\n\
echo ""\n\
echo "CloudLab is ready!"\n\
echo "Jupyter Lab: http://localhost:8888"\n\
echo "VS Code: http://localhost:8080"\n\
echo ""\n\
\n\
# Keep container running\n\
tail -f /dev/null\n\
' > /entrypoint.sh && chmod +x /entrypoint.sh

USER cloudlab

ENTRYPOINT ["/entrypoint.sh"]
