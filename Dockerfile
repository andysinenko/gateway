# ---------- build stage ----------
FROM rust:1.94-slim-bookworm as builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

COPY src ./src
RUN touch src/main.rs && cargo build --release

# ---------- runtime stage ----------
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/gateway .
COPY route_config.yaml .

ENV GW_HOST=0.0.0.0
ENV GW_PORT=3000
ENV GW_CONFIG_PATH=/app/config.yaml
EXPOSE 3000

# ---------- Ooooooooo! ----------
CMD ["./gateway"]