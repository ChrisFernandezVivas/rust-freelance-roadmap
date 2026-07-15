# ejercicios (fase 2)

Cuatro ejercicios de nivel intermedio: mutabilidad interior, threads con
`thread::scope`, y async con Tokio (backoff y control de concurrencia).

## Cómo correrlos

```bash
cargo test              # los 19 tests, todos en verde
cargo test ej03         # solo un ejercicio
```

## Ejercicios

| Módulo | Tema | Qué demuestra |
|---|---|---|
| `ej01_cache_memo` | `RefCell` | Memoización detrás de `&self` — mutabilidad interior real, con borrows cortos y sin solapamientos |
| `ej02_mapreduce` | `thread::scope` + `mpsc` | MapReduce casero: partición de trabajo, threads que piden prestado del stack (sin `'static`), merge por channel |
| `ej03_reintentos_async` | Tokio + genéricos | Reintentos con backoff exponencial — el patrón real contra APIs poco confiables; firma `F: FnMut() -> Fut, Fut: Future<...>` |
| `ej04_pipeline_async` | `tokio::sync::Semaphore` | Límite de concurrencia en un pipeline async — scraping/clientes HTTP sin saturar nada |

Igual que en la fase 1: la forma correcta de ejercitar es vaciar los cuerpos
con `todo!()` y reimplementar hasta que los tests vuelvan a pasar.

Parte de la [fase 2 del roadmap](../README.md).
