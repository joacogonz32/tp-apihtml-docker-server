# Dockerfile: Rust + Actix Web (Multi-Stage)

# Etapa 1: Builder
FROM rust:1.87-bookworm AS builder

WORKDIR /app

# Copiar manifiestos primero para cachear dependencias
COPY Cargo.toml Cargo.lock* ./

# Crear un main.rs dummy para compilar solo dependencias
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# Copiar el código fuente real y recompilar
COPY src ./src
RUN touch src/main.rs && cargo build --release

# Etapa 2: Producción 
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/rust-actixweb .

EXPOSE 8080

CMD ["./rust-actixweb"]
