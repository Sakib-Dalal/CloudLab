# CloudLab Docker Image
# Author: Sakib Dalal
# GitHub: https://github.com/Sakib-Dalal/CloudLab

FROM ubuntu:22.04

LABEL maintainer="Sakib Dalal"
LABEL version="1.2.0"

ENV DEBIAN_FRONTEND=noninteractive
ENV TZ=UTC
ENV PATH="/root/.local/bin:$PATH"

# Install all dependencies
RUN apt-get update && apt-get install -y \
    curl wget git ca-certificates \
    python3 python3-pip python3-venv \
    nodejs npm \
    && rm -rf /var/lib/apt/lists/*

# Install code-server
RUN curl -fsSL https://code-server.dev/install.sh | sh

# Install ttyd
RUN ARCH=$(dpkg --print-architecture) && \
    if [ "$ARCH" = "amd64" ]; then \
        curl -L https://github.com/tsl0922/ttyd/releases/latest/download/ttyd.x86_64 -o /usr/local/bin/ttyd; \
    else \
        curl -L https://github.com/tsl0922/ttyd/releases/latest/download/ttyd.aarch64 -o /usr/local/bin/ttyd; \
    fi && chmod +x /usr/local/bin/ttyd

# Install cloudflared
RUN ARCH=$(dpkg --print-architecture) && \
    if [ "$ARCH" = "amd64" ]; then \
        curl -L https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64 -o /usr/local/bin/cloudflared; \
    else \
        curl -L https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-arm64 -o /usr/local/bin/cloudflared; \
    fi && chmod +x /usr/local/bin/cloudflared

# Install Python packages
RUN pip3 install --no-cache-dir \
    jupyterlab notebook ipykernel ipywidgets \
    numpy pandas matplotlib

# Register kernel
RUN python3 -m ipykernel install --name cloudlab --display-name "Python 3 (CloudLab)"

# Create directories
RUN mkdir -p /root/.cloudlab/logs /root/.cloudlab/pids /workspace

# Copy dashboard files
COPY index.html /root/.cloudlab/dashboard.html
COPY server.py /root/.cloudlab/server.py

# Configure Jupyter (no password, no token)
RUN mkdir -p /root/.jupyter && echo "\
c = get_config()\n\
c.ServerApp.ip = '0.0.0.0'\n\
c.ServerApp.port = 8888\n\
c.ServerApp.open_browser = False\n\
c.ServerApp.allow_root = True\n\
c.ServerApp.allow_origin = '*'\n\
c.ServerApp.token = ''\n\
c.ServerApp.password = ''\n\
c.NotebookApp.token = ''\n\
c.NotebookApp.password = ''\n\
" > /root/.jupyter/jupyter_server_config.py

# Configure code-server
RUN mkdir -p /root/.config/code-server && echo "\
bind-addr: 0.0.0.0:8080\n\
auth: password\n\
password: cloudlab\n\
cert: false\n\
" > /root/.config/code-server/config.yaml

# Create config
RUN echo '{"jupyter_port":8888,"vscode_port":8080,"ssh_port":7681,"dashboard_port":3000}' > /root/.cloudlab/config.json

WORKDIR /workspace
EXPOSE 8888 8080 7681 3000

# Simple startup script
RUN echo '#!/bin/bash\n\
echo ""\n\
echo "============================================"\n\
echo "  CloudLab Docker v1.2.0"\n\
echo "  Author: Sakib Dalal"\n\
echo "  GitHub: github.com/Sakib-Dalal/CloudLab"\n\
echo "============================================"\n\
echo ""\n\
echo "Starting Jupyter Lab..."\n\
jupyter lab --ip=0.0.0.0 --port=8888 --no-browser --allow-root --notebook-dir=/workspace --ServerApp.token="" --ServerApp.password="" > /root/.cloudlab/logs/jupyter.log 2>&1 &\n\
echo "Starting VS Code..."\n\
code-server --bind-addr=0.0.0.0:8080 /workspace > /root/.cloudlab/logs/vscode.log 2>&1 &\n\
echo "Starting Terminal..."\n\
ttyd --port 7681 --writable bash > /root/.cloudlab/logs/ssh.log 2>&1 &\n\
echo "Starting Dashboard..."\n\
python3 /root/.cloudlab/server.py > /root/.cloudlab/logs/dashboard.log 2>&1 &\n\
echo ""\n\
echo "============================================"\n\
echo "  All services started!"\n\
echo "============================================"\n\
echo "  Jupyter:   http://localhost:8888"\n\
echo "  VS Code:   http://localhost:8080  (password: cloudlab)"\n\
echo "  Terminal:  http://localhost:7681"\n\
echo "  Dashboard: http://localhost:3000"\n\
echo "============================================"\n\
echo ""\n\
tail -f /dev/null\n\
' > /start.sh && chmod +x /start.sh

CMD ["/start.sh"]