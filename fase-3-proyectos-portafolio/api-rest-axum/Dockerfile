# syntax=docker/dockerfile:1

# ---- Etapa 1: build ---------------------------------------------------
# Compilamos en una imagen con el toolchain completo de Rust. Esta capa
# NO viaja a producción: solo nos interesa el binario final que produce.
FROM rust:1.82-slim AS builder

WORKDIR /app

# sqlx con el driver de sqlite necesita libsqlite3 + herramientas de build
# (el crate `libsqlite3-sys` compila SQLite embebido desde C si hace falta).
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

# Copiamos todo el proyecto (manifiestos + código + migraciones).
# Nota: para un cacheo de capas más fino en un proyecto grande, se
# copiarían primero Cargo.toml/Cargo.lock y se haría un build "vacío"
# para cachear las dependencias; acá el proyecto es chico y no vale la
# complejidad extra.
COPY . .

RUN cargo build --release

# ---- Etapa 2: runtime ---------------------------------------------------
# Imagen final MÍNIMA: sin compilador, sin toolchain, solo lo necesario
# para EJECUTAR el binario. Esto reduce drásticamente el tamaño de la
# imagen y la superficie de ataque (menos paquetes = menos CVEs posibles).
FROM debian:bookworm-slim

# libsqlite3 en runtime (el binario está linkeado dinámicamente contra ella)
# + curl, solo para que el HEALTHCHECK de abajo pueda pegarle al endpoint.
RUN apt-get update && apt-get install -y --no-install-recommends \
    libsqlite3-0 \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copiamos SOLO el binario compilado de la etapa anterior.
COPY --from=builder /app/target/release/api-rest-axum /app/api-rest-axum

# La base SQLite se crea en este volumen si no existe (create_if_missing).
# En un despliegue real, montar un volumen persistente acá.
ENV DATABASE_URL=sqlite:/app/data/tareas.db?mode=rwc
RUN mkdir -p /app/data

EXPOSE 3000

# Healthcheck simple contra el endpoint /salud (requiere curl; se podría
# reemplazar por un binario propio de healthcheck para no depender de curl).
HEALTHCHECK --interval=30s --timeout=3s \
    CMD curl -f http://localhost:3000/salud || exit 1

CMD ["/app/api-rest-axum"]
