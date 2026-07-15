# 📚 Recursos

Todos los links, libros, cursos y repos de referencia del roadmap, en un solo lugar.

---

## 📖 Libros y guías fundamentales

| Recurso | Link | Para qué |
|---|---|---|
| **The Rust Book** | <https://doc.rust-lang.org/book/> | La referencia oficial. Capítulos clave: 4 (ownership), 10 (traits/generics/lifetimes), 13 (iterators/closures), 16 (concurrencia). |
| **Rustlings** | <https://github.com/rust-lang/rustlings> | Ejercicios cortos para fijar sintaxis y conceptos. Terminarlo completo es parte de la fase 1. |
| **Exercism — track de Rust** | <https://exercism.org/tracks/rust> | Ejercicios con *mentoría humana gratis*: la diferencia entre código que compila y código idiomático. |
| **100 Exercises to Learn Rust** (Mainmatter) | <https://github.com/mainmatter/100-exercises-to-learn-rust> | Curso guiado por ejercicios, más estructurado que Rustlings. También en <https://rust-exercises.com>. |
| **Rust for Rustaceans** (Jon Gjengset, No Starch Press) | <https://nostarch.com/rust-rustaceans> | EL libro de nivel intermedio. Lectura obligada de la fase 2. |
| **Zero To Production In Rust** (Luca Palmieri) | <https://www.zero2prod.com> | Construir una API de producción real con Axum/Actix, tests, CI, deploy. Guía de referencia para la fase 3. |

## 🎬 Video

| Recurso | Link | Para qué |
|---|---|---|
| **Crust of Rust** (Jon Gjengset) | <https://www.youtube.com/@jonhoo> | Streams largos donde implementa partes de la stdlib desde cero (lifetimes, channels, `Rc`, async). Complemento perfecto de *Rust for Rustaceans*. |

## 🔍 Repos de GitHub para estudiar código ajeno

Leer código bien escrito es la forma más rápida de pasar de "compila" a "es idiomático".

| Repo | Link | Qué mirar |
|---|---|---|
| **tokio-rs/axum** | <https://github.com/tokio-rs/axum> | La carpeta [`examples/`](https://github.com/tokio-rs/axum/tree/main/examples): mini-apps completas (auth, websockets, testing, SQLx…). Oro puro para la fase 3. |
| **BurntSushi/ripgrep** | <https://github.com/BurntSushi/ripgrep> | "El CLI mejor escrito del ecosistema". Manejo de errores, performance, estructura de crates. |
| **rust-unofficial/awesome-rust** | <https://github.com/rust-unofficial/awesome-rust> | Índice general de crates y proyectos por categoría. |
| **rust-lang/rustlings** | <https://github.com/rust-lang/rustlings> | Además de hacer los ejercicios: **mirar los PRs** para aprender cómo se revisa código Rust en un proyecto real. |
| **solana-developers/program-examples** | <https://github.com/solana-developers/program-examples> | Patrones reales de programas de Solana (nativo y Anchor). Referencia principal de la fase 4. |

## ⛓️ Solana / Web3 (fase 4)

| Recurso | Link | Para qué |
|---|---|---|
| **Solana Playground** | <https://beta.solpg.io> | IDE en el navegador: escribir, compilar y desplegar programas en devnet **sin instalar nada**. |
| **Anchor Book** | <https://book.anchor-lang.com> | Documentación oficial del framework Anchor. |
| **Cursos oficiales de Solana** | <https://solana.com/developers/courses> | Cursos gratuitos y estructurados del equipo de Solana Foundation. |
| **RareSkills — Solana tutorial** | <https://www.rareskills.io> | Tutorial que explica *el fondo* (cuentas, rent, CPI, seguridad), no solo el copy-paste. Buscar su curso/tutorial de Solana. |

---

## 🧭 Cómo usar estos recursos

- **Fase 1** → The Rust Book + Rustlings + Exercism / 100-exercises.
- **Fase 2** → Rust for Rustaceans + Crust of Rust.
- **Fase 3** → zero2prod + examples/ de axum + ripgrep como referencia de calidad.
- **Fase 4** → Solana Playground + Anchor Book + program-examples + RareSkills.

No intentar consumir todo en paralelo: cada fase tiene sus recursos, y la regla de oro
del [README principal](../README.md) aplica también acá.
