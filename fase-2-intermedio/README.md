# Fase 2 — Intermedio real

> ⏱️ Tiempo estimado: **2–3 meses**. Es la fase más larga y la más importante:
> acá se construye el vocabulario técnico (`Send`/`Sync`, `Arc<Mutex>`,
> `Rc<RefCell>`, `tokio::spawn`/`select!`) que vas a necesitar defender en
> cualquier entrevista o llamada con un cliente freelance.

Requisito: haber completado el [checklist de la fase 1](../fase-1-base/README.md#-checklist-para-pasar-a-la-fase-2).

## 📂 Contenido de esta carpeta

| Ítem | Tema | Cómo correrlo |
|---|---|---|
| [`smart_pointers.rs`](smart_pointers.rs) | `Box`, `Rc`, `RefCell` — cuándo usar cada uno, comparado con `unique_ptr`/`shared_ptr` de C++ | `rustc smart_pointers.rs -o /tmp/sp && /tmp/sp` |
| [`concurrencia/`](concurrencia/) | Proyecto Cargo: threads, `Arc<Mutex>`, channels `mpsc` | `cd concurrencia && cargo run && cargo test` |
| [`async-tokio/`](async-tokio/) | Proyecto Cargo: `tokio::spawn`, `tokio::sync::Mutex`, `select!` | `cd async-tokio && cargo run && cargo test` |
| [`ejercicios/`](ejercicios/) | 4 ejercicios de nivel intermedio con tests | `cd ejercicios && cargo test` |

### Los ejercicios

`RefCell` con memoización (mutabilidad interior detrás de `&self`),
un mini MapReduce con `thread::scope` + channels (el gemelo educativo de
`concurrencia/`), reintentos con backoff exponencial en async (el patrón que
vas a usar contra CUALQUIER API externa poco confiable), y un pipeline async
con `Semaphore` para limitar concurrencia (scraping/clientes HTTP masivos sin
tumbar nada).

## 📚 Plan de estudio de la fase

1. **Leer "Rust for Rustaceans"** de Jon Gjengset (No Starch Press). El libro
   de referencia para pasar de "conozco la sintaxis" a "entiendo el modelo
   de memoria, los traits `Send`/`Sync`, y por qué el compilador decide lo
   que decide".
2. **Ver "Crust of Rust"** (canal de YouTube de Jon Gjengset,
   <https://www.youtube.com/@jonhoo>) — streams largos reimplementando
   partes de la stdlib desde cero (lifetimes, `Rc`, iteradores, channels).
   Complemento audiovisual perfecto del libro.
3. **Dominar en profundidad**:
   - `Send` / `Sync`: qué tipos pueden cruzar threads y por qué (¡el
     compilador te lo dice, no hace falta memorizar reglas de memoria!).
   - `Arc<Mutex<T>>`: estado compartido entre threads, sin data races.
   - Channels (`mpsc`): paso de mensajes como alternativa al estado compartido.
   - `Box` / `Rc` / `RefCell`: ownership único, compartido, y mutabilidad
     interior — y sus costos reales (heap alloc, contador, verificación runtime).
   - Async con Tokio: `tokio::spawn`, `tokio::sync::Mutex`, `select!` — y
     la regla de oro (nunca bloquear el thread dentro de una `async fn`).

## ✅ Checklist para pasar a la fase 3

- [ ] "Rust for Rustaceans" leído (al menos los capítulos de tipos, traits y concurrencia).
- [ ] Al menos 5 videos de "Crust of Rust" vistos (prioridad: el de lifetimes, el de `Rc`/`RefCell`, el de iteradores).
- [ ] `smart_pointers.rs` leído y corrido — podés explicar de memoria cuándo usar `Box` vs `Rc` vs `Rc<RefCell>`.
- [ ] `concurrencia/`: `cargo test` en verde, y podés explicar por qué el mutex "contiene" al dato en vez de estar suelto al lado.
- [ ] `async-tokio/`: `cargo test` en verde, y podés explicar la diferencia entre threads (CPU-bound) y tasks async (IO-bound).
- [ ] Los 4 ejercicios de `ejercicios/` resueltos desde `todo!()`.
- [ ] Podés explicar en voz alta: (a) qué es `Send`/`Sync` y por qué `Rc` no es `Send`, (b) cuándo `std::sync::Mutex` no alcanza y hace falta `tokio::sync::Mutex`, (c) qué hace `select!` con el future que "pierde" la carrera.

Si algún punto no está, **no pasar a la fase 3** — la regla de oro del
[README principal](../README.md).
