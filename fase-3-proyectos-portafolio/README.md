# Fase 3 — Proyectos de portafolio

> ⏱️ Tiempo estimado: **2–3 meses**. Esta fase ES el portafolio: los tres
> proyectos de acá son lo que le mostrás a un cliente freelance.

Requisito: haber completado el [checklist de la fase 2](../fase-2-intermedio/README.md#-checklist-para-pasar-a-la-fase-3).

## 📂 Los tres proyectos

| Proyecto | Qué es | Conceptos clave |
|---|---|---|
| [`cli-tool/`](cli-tool/) | `textkit`: buscador/reemplazador de texto en archivos | `clap`, `anyhow`, tests de integración con `assert_cmd` |
| [`api-rest-axum/`](api-rest-axum/) | API REST de tareas (CRUD completo) | `axum`, `sqlx` (SQLite), manejo de errores HTTP, Docker |
| [`mini-blockchain/`](mini-blockchain/) | Blockchain educativa | `sha2`, proof-of-work, validación de cadena |

Cada uno es un proyecto **completo y funcional**: su propio `Cargo.toml`,
su propio README (qué hace, cómo correrlo, qué conceptos demuestra), tests
que pasan de verdad, y — en el caso de la API — un `Dockerfile`.

```bash
# Verificar los tres de una:
for proyecto in cli-tool api-rest-axum mini-blockchain; do
    (cd "$proyecto" && cargo test)
done
```

## 📖 Proyecto guiado recomendado en paralelo

**[Zero To Production In Rust](https://www.zero2prod.com)** (Luca
Palmieri) — construye una API de producción real con Axum/Actix, tests de
principio a fin, CI, y deploy. No hace falta reproducirlo completo en este
repo, pero seguirlo en paralelo (aunque sea los primeros capítulos) le da
contexto de PRODUCCIÓN a lo que ya construiste acá: qué cambia entre "un
CRUD que funciona" y "un servicio que corre en producción con monitoreo,
logs estructurados y un pipeline de despliegue".

## ✅ Checklist para pasar a la fase 4

- [ ] `textkit`: `cargo test` en verde (19 tests), usado de verdad al menos una vez sobre un proyecto real (no solo los tests).
- [ ] `api-rest-axum`: `cargo test` en verde (9 tests), corrido con `cargo run` y probado manualmente con `curl`, `docker build` corrido al menos una vez localmente.
- [ ] `mini-blockchain`: `cargo test` en verde (14 tests), y podés explicar en voz alta por qué manipular un bloque invalida el resto de la cadena.
- [ ] El CI del repo (`.github/workflows/ci.yml`) corre en verde en un push/PR real (no solo local).
- [ ] Al menos empezado *Zero To Production In Rust* (los primeros 2-3 capítulos).
- [ ] Podés defender cada proyecto en una entrevista: por qué elegiste cada crate, qué harías distinto en producción real (ej. Postgres en vez de SQLite), qué tests faltarían para un caso de uso real.

Si algún punto no está, **no pasar a la fase 4** — y en particular: no
saltar a Solana sin haber internalizado el manejo de errores, ownership y
async de estas fases. La regla de oro del [README principal](../README.md)
aplica con más fuerza acá que en ningún otro lado.
