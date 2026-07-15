# Fase 1 — Base sólida

> ⏱️ Tiempo estimado: **2–4 semanas** (es un repaso profundo, no un curso desde cero).

El objetivo de esta fase NO es "aprender sintaxis" (eso ya está). Es consolidar
los cuatro conceptos que **separan a un junior de un intermedio** en Rust —
los que aparecen en toda entrevista y en todo code review:

1. **Ownership y lifetimes** (por qué el borrow checker rechaza tu código, y por qué tiene razón)
2. **Traits y genéricos** (despacho estático vs. dinámico — la decisión de diseño más frecuente)
3. **Manejo de errores** (`Result`, `?`, errores custom — cero `unwrap()` en código serio)
4. **Iteradores y closures** (código idiomático vs. "C escrito en Rust")

## 📂 Contenido de esta carpeta

| Archivo | Tema | Cómo correrlo |
|---|---|---|
| [`ownership_lifetimes.rs`](ownership_lifetimes.rs) | Move, borrows, dangling pointers, lifetimes explícitos y en structs | `rustc ownership_lifetimes.rs -o /tmp/ol && /tmp/ol` |
| [`traits_generics.rs`](traits_generics.rs) | Traits, genéricos, monomorfización, `dyn Trait`, derive | `rustc traits_generics.rs -o /tmp/tg && /tmp/tg` |
| [`errores_result.rs`](errores_result.rs) | `Result`, `?`, errores custom con `Display`/`From`, `Option` | `rustc errores_result.rs -o /tmp/er && /tmp/er` |
| [`iterators_closures.rs`](iterators_closures.rs) | Closures (Fn/FnMut/FnOnce), adaptadores, laziness, iteradores propios | `rustc iterators_closures.rs -o /tmp/ic && /tmp/ic` |
| [`ejercicios/`](ejercicios/) | 6 ejercicios con tests, estilo Rustlings pero más difíciles | `cd ejercicios && cargo test` |

Los `.rs` están comentados línea por línea **explicando el porqué**, con
comparaciones directas a C/C++ (ownership vs. malloc/free, borrow checker vs.
punteros colgantes, traits vs. interfaces/vtables, closures vs. callbacks con
`void*`). Leelos con el compilador al lado: varios tienen bloques comentados
que NO compilan a propósito — descomentá y mirá el error.

### Los ejercicios

En [`ejercicios/`](ejercicios/) hay 6 ejercicios pensados para alguien que ya
sabe la sintaxis: frecuencias de palabras (HashMap + ownership), pila genérica
con mínimo O(1) (bounds), parser de configuración (errores custom + `?`),
pipeline de sensores (iteradores), búsqueda sin copiar (lifetimes en serio) y
reporte polimórfico (`Box<dyn Trait>`).

Vienen con solución de referencia para que el CI verifique el repo; la forma
correcta de ejercitar es **borrar los cuerpos, poner `todo!()` y reimplementar
hasta que `cargo test` vuelva a pasar**.

## 📚 Plan de estudio de la fase

1. **Terminar Rustlings completo** — <https://github.com/rust-lang/rustlings>.
   Sin saltear los de lifetimes ni los de smart pointers.
2. **The Rust Book en paralelo** — <https://doc.rust-lang.org/book/>, con
   énfasis especial en los capítulos que separan juniors de intermedios:
   - **Cap. 4** — Ownership (leerlo dos veces si hace falta)
   - **Cap. 10** — Traits, genéricos y lifetimes
   - **Cap. 13** — Iterators y closures
   - **Cap. 16** — Concurrencia (prepara la fase 2)
3. **Complementar con ejercicios corregidos**: el track de Rust de
   **Exercism** (<https://exercism.org/tracks/rust>) o los
   **100 Exercises to Learn Rust** de Mainmatter
   (<https://github.com/mainmatter/100-exercises-to-learn-rust>).
   El objetivo acá es escribir código **idiomático**, no solo que compile —
   la mentoría de Exercism te marca la diferencia.

## ✅ Checklist para pasar a la fase 2

- [ ] Rustlings 100% completado.
- [ ] Capítulos 4, 10, 13 y 16 del Rust Book leídos (y sus ejemplos tipeados, no copiados).
- [ ] Los 4 archivos `.rs` de esta carpeta leídos, corridos, y sus bloques "no compila" verificados contra el compilador.
- [ ] Los 6 ejercicios de `ejercicios/` reimplementados desde `todo!()` con todos los tests en verde.
- [ ] Al menos 10 ejercicios de Exercism (o 30 de 100-exercises) resueltos.
- [ ] Podés explicar en voz alta, sin mirar: (a) por qué Rust no necesita GC, (b) la regla de los borrows (&T vs &mut T), (c) cuándo usar genéricos vs `dyn Trait`, (d) qué hace `?` exactamente.

Si algún punto no está, **no pasar a la fase 2** — la regla de oro del
[README principal](../README.md).
