# concurrencia — threads, `Arc<Mutex>` y channels mpsc

Proyecto Cargo funcional que demuestra los **dos modelos de concurrencia
clásica** de Rust, sin ninguna dependencia externa (todo es `std`):

1. **Estado compartido** — `Arc<Mutex<T>>`: el contador multi-thread que en C
   sería un data race, acá es determinista o no compila.
2. **Paso de mensajes** — channels `mpsc`: un pool de workers que consume
   trabajos de un canal y devuelve resultados por otro, con apagado limpio
   por cierre de canal (sin flags, sin señales).

## Cómo correrlo

```bash
cargo run     # demo con salida por consola
cargo test    # 6 tests (contador exacto, pool, casos borde, corruptos)
```

## Qué conceptos de Rust demuestra

- **`Send`/`Sync`**: por qué `Rc` entre threads no compila y `Arc` sí — el
  compilador verifica la seguridad entre threads en los TIPOS.
- **`Arc<Mutex<T>>`**: el mutex *contiene* al dato (imposible tocarlo sin
  `lock()`), a diferencia de pthreads/std::mutex donde la relación
  mutex↔dato es un comentario.
- **RAII del `MutexGuard`**: el unlock es automático al salir de scope, y el
  patrón "sección crítica corta" se ve explícito en el código del pool.
- **Ownership a través de channels**: `send()` MUEVE el valor; usar algo
  después de enviarlo no compila (en C++ pasar un puntero por una cola y
  seguir usándolo es una carrera silenciosa).
- **Apagado por protocolo de canal**: `drop(tx)` → los `recv()` devuelven
  `Err` → los workers salen del loop. Cero variables de "por favor pará".
- **`thread::spawn` + `move` + `join`**: y por qué `move` es obligatorio
  (el thread puede sobrevivir al stack que creó el closure).

## Estructura

```
src/lib.rs    # contar_en_paralelo() y procesar_pool() + tests
src/main.rs   # demo ejecutable
```

Parte de la [fase 2 del roadmap](../README.md). El gemelo async de este
proyecto es [`async-tokio/`](../async-tokio/).
