# api-rest-axum — API REST de tareas (Axum + SQLx)

API REST CRUD real para gestionar tareas, construida con **Axum** (el
framework HTTP async del ecosistema Tokio) y **SQLx** (acceso a base de
datos async, verificado por tipos).

## Por qué SQLite y no Postgres

**En producción real, este servicio usaría Postgres.** Para este repo de
portafolio/estudio se usa **SQLite** deliberadamente: así el proyecto
**compila y corre en cualquier máquina sin infraestructura externa** — sin
Docker Compose, sin levantar un servidor de Postgres, sin credenciales.
`cargo test` y `cargo run` simplemente funcionan. Ver el comentario en
[`src/db.rs`](src/db.rs) para el detalle de qué cambiaría al migrar a
Postgres (en gran parte: el feature de sqlx y el tipo de pool — el resto
del código queda igual).

## Endpoints

| Método | Ruta | Descripción |
|---|---|---|
| `GET` | `/salud` | Health check (200 OK) |
| `POST` | `/tareas` | Crear una tarea (`{ "titulo": "..." }`) |
| `GET` | `/tareas` | Listar todas las tareas |
| `GET` | `/tareas/{id}` | Obtener una tarea |
| `PUT` | `/tareas/{id}` | Actualizar parcialmente (`titulo` y/o `completada`) |
| `DELETE` | `/tareas/{id}` | Eliminar una tarea |

## Cómo correrlo

```bash
cargo run
# servidor en http://localhost:3000, crea tareas.db en el directorio actual

curl -X POST localhost:3000/tareas -H 'content-type: application/json' \
  -d '{"titulo": "aprender Axum"}'

curl localhost:3000/tareas

curl -X PUT localhost:3000/tareas/1 -H 'content-type: application/json' \
  -d '{"completada": true}'

curl -X DELETE localhost:3000/tareas/1
```

## Tests

```bash
cargo test
```

9 tests de integración en [`tests/integracion.rs`](tests/integracion.rs):
ejercitan el **router completo de Axum** (handlers + SQLx) contra una base
SQLite real **en memoria**, usando `tower::ServiceExt::oneshot` — un
request in-process, sin abrir un socket TCP real. Cada test crea su propia
base aislada, así que corren en paralelo sin pisarse.

Cubren: creación, listado, obtención, actualización parcial (PATCH-like),
eliminación, validación de título vacío (400), y recursos inexistentes (404).

## Docker

```bash
docker build -t api-rest-axum .
docker run -p 3000:3000 api-rest-axum
```

Build multi-stage: la etapa de compilación usa la imagen completa de Rust;
la imagen final es `debian:bookworm-slim` con solo el binario y las
librerías de runtime necesarias (`libsqlite3`, `ca-certificates`, `curl`
para el healthcheck) — sin el toolchain de Rust, mucho más liviana.

## Qué conceptos de Rust demuestra

- **Axum + extractors por tipos**: `State<SqlitePool>`, `Path<i64>`,
  `Json<T>` declaran qué necesita cada handler; Axum los resuelve antes de
  llamarlo — inyección de dependencias sin contenedor DI ni reflection.
- **SQLx async**: queries no bloqueantes contra SQLite, con `FromRow`
  derivado para mapear filas a structs de Rust automáticamente.
- **`IntoResponse` para errores de dominio**: `ErrorApi` (con `thiserror`)
  se traduce a `(StatusCode, JSON)` en un solo lugar — el equivalente de
  un exception handler global, pero resuelto por el sistema de tipos.
- **`#[from]` de thiserror**: `sqlx::Error` se convierte automáticamente a
  `ErrorApi` con `?`, mismo mecanismo que `From` manual visto en
  `fase-1-base/errores_result.rs`.
- **Separación lib/bin**: `construir_app()` vive en la lib para que los
  tests de integración usen EXACTAMENTE la misma app que corre en
  producción, sin duplicar el armado del router.
- **Modelos de entrada/salida separados**: `NuevaTarea`, `ActualizarTarea`
  y `Tarea` son tipos distintos — el cliente nunca puede mandar un `id` o
  una `creada_en` inventados.
- **Migraciones embebidas** (`sqlx::migrate!`): el esquema SQL vive en
  `migrations/` y se aplica automáticamente al arrancar, sin paso manual.

## Estructura

```
src/
  main.rs       # arranque: tracing, pool, bind, axum::serve
  lib.rs        # construir_app(): arma el Router (usado por main y tests)
  db.rs         # creación del pool + migraciones
  modelos.rs    # Tarea, NuevaTarea, ActualizarTarea
  rutas.rs      # los 5 handlers del CRUD
  error.rs      # ErrorApi -> IntoResponse
migrations/
  0001_crear_tareas.sql
tests/
  integracion.rs
Dockerfile
```

Parte de la [fase 3 del roadmap](../README.md). Proyecto guiado recomendado
en paralelo: [zero2prod.com](https://www.zero2prod.com) (Luca Palmieri).
