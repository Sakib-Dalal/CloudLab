FROM ghcr.io/coder/code-server:4.104.2
COPY --from=ghcr.io/astral-sh/uv:0.12.17 /uv /uvx /usr/local/bin/
USER root
RUN apt-get update && apt-get install -y --no-install-recommends socat python3 python3-venv ca-certificates && rm -rf /var/lib/apt/lists/*
RUN uv pip install --system --break-system-packages --no-cache ipykernel==6.30.1
RUN mkdir -p /home/lab/projects && chown -R 1000:1000 /home/lab
COPY --chmod=755 containers/shared/entrypoint.sh /usr/local/bin/cloudlab-entrypoint
COPY --chmod=755 containers/shared/packages.py /usr/local/bin/cloudlab-packages
COPY containers/shared/profile.sh /etc/profile.d/cloudlab.sh
ENV HOME=/home/lab PYTHONDONTWRITEBYTECODE=1 VIRTUAL_ENV=/home/lab/.venv UV_PYTHON_DOWNLOADS=never UV_LINK_MODE=copy CLOUDLAB_BASE_PYTHON=/usr/bin/python3 CLOUDLAB_TEMPLATE=code
ENV PATH="/home/lab/.venv/bin:$PATH"
USER 1000:1000
WORKDIR /home/lab
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/cloudlab-entrypoint"]
CMD ["--bind-addr", "0.0.0.0:8080", "--auth", "none", "/home/lab"]
