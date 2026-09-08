FROM debian:bookworm-slim
RUN useradd --create-home --uid 1000 --shell /bin/bash lab && mkdir -p /home/lab/projects && chown -R 1000:1000 /home/lab
USER 1000:1000
ENV HOME=/home/lab
WORKDIR /home/lab
CMD ["sleep", "infinity"]
