FROM ghcr.io/coder/code-server:4.104.2
USER root
RUN mkdir -p /home/lab/projects && chown -R 1000:1000 /home/lab
RUN apt-get update && apt-get install -y --no-install-recommends socat && rm -rf /var/lib/apt/lists/*
ENV HOME=/home/lab
USER 1000:1000
WORKDIR /home/lab
EXPOSE 8080
ENTRYPOINT ["/usr/bin/code-server"]
