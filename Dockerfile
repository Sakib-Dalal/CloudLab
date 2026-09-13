FROM node:22-bookworm-slim AS frontend
WORKDIR /source
COPY package.json package-lock.json ./
RUN npm ci
COPY index.html vite.config.ts svelte.config.js tsconfig.json ./
COPY src ./src
COPY shared ./shared
COPY public ./public
RUN npm run build

FROM rust:1-bookworm AS backend
WORKDIR /source
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY src-tauri ./src-tauri
RUN cargo build --release --locked -p cloudlab

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/* && useradd --uid 1000 --create-home cloudlab && mkdir -p /data && chown cloudlab:cloudlab /data
COPY --from=backend /source/target/release/cloudlab /usr/local/bin/cloudlab
COPY --from=frontend /source/dist /opt/cloudlab/web
USER 1000:1000
WORKDIR /data
EXPOSE 8088 8089
ENV CLOUDLAB_BIND=0.0.0.0:8088 CLOUDLAB_APP_BIND=0.0.0.0:8089 CLOUDLAB_DATA_DIR=/data CLOUDLAB_WEB_DIR=/opt/cloudlab/web
ENTRYPOINT ["cloudlab"]
CMD ["serve"]
