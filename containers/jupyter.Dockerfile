FROM python:3.12-slim-bookworm
RUN pip install --no-cache-dir jupyterlab==4.4.9 ipykernel==6.30.1 && useradd --create-home --uid 1000 --shell /bin/bash lab && mkdir -p /home/lab/projects && chown -R 1000:1000 /home/lab
RUN apt-get update && apt-get install -y --no-install-recommends socat && rm -rf /var/lib/apt/lists/*
USER 1000:1000
ENV HOME=/home/lab PYTHONDONTWRITEBYTECODE=1
WORKDIR /home/lab
EXPOSE 8080
