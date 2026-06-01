# Dockerfile: Rust + Actix Web (Multi-Stage)

# Etapa 1: Builder
FROM rust:latest AS builder

WORKDIR /app

# Copiar manifiestos primero para cachear dependencias
COPY . .

# Copiar el código fuente real y recompilar
RUN cargo build --release

# Etapa 2: Producción 
FROM debian:bookworm-slim

WORKDIR /app

COPY --from=builder /app/target/release/rust-actixweb .

EXPOSE 8080

CMD ["./rust-actixweb"]
