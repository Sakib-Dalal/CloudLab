FROM python:3.12-slim-bookworm
COPY --from=ghcr.io/astral-sh/uv:0.12.17 /uv /uvx /usr/local/bin/
RUN apt-get update && apt-get install -y --no-install-recommends socat ca-certificates && rm -rf /var/lib/apt/lists/*
RUN uv pip install --system --no-cache ipykernel==6.30.1 jupyterlab==4.4.9
RUN useradd --create-home --uid 1000 --shell /bin/bash lab
RUN mkdir -p /home/lab/projects && chown -R 1000:1000 /home/lab
COPY --chmod=755 containers/shared/entrypoint.sh /usr/local/bin/cloudlab-entrypoint
COPY --chmod=755 containers/shared/packages.py /usr/local/bin/cloudlab-packages
COPY containers/shared/profile.sh /etc/profile.d/cloudlab.sh
ENV HOME=/home/lab PYTHONDONTWRITEBYTECODE=1 VIRTUAL_ENV=/home/lab/.venv UV_PYTHON_DOWNLOADS=never UV_LINK_MODE=copy CLOUDLAB_BASE_PYTHON=/usr/local/bin/python3 CLOUDLAB_TEMPLATE=jupyter
ENV PATH="/home/lab/.venv/bin:$PATH"
USER 1000:1000
WORKDIR /home/lab
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/cloudlab-entrypoint"]
CMD ["jupyter", "lab", "--ip=0.0.0.0", "--port=8080", "--no-browser"]
