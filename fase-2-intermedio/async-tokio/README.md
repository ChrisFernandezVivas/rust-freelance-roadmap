# async-tokio — `tokio::spawn`, `tokio::sync` y `select!`

Proyecto Cargo funcional que demuestra los tres pilares del async en Rust
con Tokio, el gemelo IO-bound del proyecto [`concurrencia/`](../concurrencia/)
(que es CPU-bound con threads del SO).

1. **`tokio::spawn`** — lanzar N lecturas "remotas" en paralelo: 8 sensores de
   100 ms tardan ~100 ms en total, no 800 (las esperas se solapan).
2. **`tokio::sync::Mutex`** — estado compartido entre tasks, y POR QUÉ no se
   usa `std::sync::Mutex` cuando el lock cruza un `.await`.
3. **`select!`** — timeout implementado a mano (carrera lectura vs. reloj,
   con cancelación por drop del perdedor) y el patrón **worker con graceful
   shutdown** (`mpsc` para trabajos + `oneshot` para la señal de apagado),
   la columna vertebral de cualquier servicio de larga vida en Tokio.

## Cómo correrlo

```bash
cargo run     # demo con tiempos reales por consola
cargo test    # 7 tests async (#[tokio::test])
```

## Qué conceptos de Rust demuestra

- **Futures lazy**: llamar una `async fn` no ejecuta nada; devuelve una
  máquina de estados que alguien debe `.await`ear o `spawn`ear.
- **Scheduling cooperativo**: `sleep().await` cede el thread a otras tasks
  (y por qué `std::thread::sleep` dentro de async es un pecado capital).
- **`tokio::spawn` + `JoinHandle`**: mismo contrato mental que
  `thread::spawn`/`join`, con tasks de cientos de bytes en vez de threads
  de megabytes.
- **`Arc<tokio::sync::Mutex<T>>`**: el mismo patrón de `concurrencia/`,
  con `lock().await` que suspende en vez de bloquear.
- **Cancelación por drop**: la rama perdedora de un `select!` simplemente
  se dropea — la respuesta de Rust al "¿cómo mato un thread bloqueado?".
- **Apagado limpio por protocolo de canales**: cierre de `mpsc` = fin del
  trabajo; `oneshot` = orden de apagado. Sin flags atómicos artesanales.

## Estructura

```
src/lib.rs    # leer_todos, registrar_en_paralelo, leer_con_timeout,
              # worker_con_apagado + 7 tests
src/main.rs   # demo ejecutable con tiempos medidos
```

Parte de la [fase 2 del roadmap](../README.md).
