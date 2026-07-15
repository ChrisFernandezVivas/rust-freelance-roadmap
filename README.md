# 🦀 Rust Freelance Roadmap

Plan de estudio + portafolio de código para pasar de nivel **principiante-intermedio**
a **intermedio-avanzado** en Rust, con un objetivo concreto: **trabajar freelance**
haciendo backend de alto rendimiento y, más adelante, programas de Solana con Anchor (Web3).

Este repo no es solo un plan: **es el portafolio en sí**. Cada fase tiene código real,
comentado en español, con tests que pasan y CI que lo verifica.

> **Contexto del autor**: vengo de C/C++/Python (post-silicon validation), ya trabajé
> con Rustlings, The Rust Book, Serde, threads básicos y async/await con Tokio básico.
> Por eso los ejemplos comparan constantemente contra C/C++ — es la forma más rápida
> de entender *por qué* Rust hace las cosas como las hace.

---

## 🎯 Objetivo

**Freelance en Rust**, en dos frentes:

1. **Backend de alto rendimiento** (Axum/Actix, APIs REST, servicios async con Tokio).
2. **Web3/Solana** (programas on-chain con Anchor) — el nicho mejor pagado, pero también
   el que menos perdona errores.

---

## ⚠️ Regla de oro

**No avanzar de fase sin completar el checklist de la fase anterior.**

Y en particular: **no saltar a Solana antes de terminar la fase 3.** Anchor abstrae
muchísimo (cuentas, serialización, validación), y sin dominar lifetimes, ownership y
traits se termina copiando código de tutoriales sin entenderlo. En freelance crypto
eso no es un problema académico: **un bug en un programa on-chain cuesta plata real
de terceros**, y no hay "deploy de hotfix" que devuelva fondos drenados.

---

## 💰 Nichos freelance de Rust y tarifas aproximadas

| Nicho | Descripción | Tarifa aprox. (USD/hr) |
|---|---|---|
| **Blockchain / Web3 (Solana)** | Programas on-chain con Anchor, integraciones, auditoría básica | **$60–150+** (el mejor pagado) |
| **Backend de alto rendimiento** | APIs con Axum/Actix, servicios async, migraciones desde Python/Node por performance | $50–100 |
| **CLI tools / automatización** | Herramientas internas, tooling de DevOps, migración de scripts | $40–80 |
| **Embedded / IoT** | Firmware, `no_std`, drivers | $50–100 |
| **WASM** | Módulos de alto rendimiento para la web, plugins | $50–90 |

*(Tarifas orientativas de mercado; varían según experiencia demostrable y país del cliente.)*

---

## 🧲 Qué hace que te contraten

Un cliente freelance **no contrata una enciclopedia de ejemplos**. Contrata a alguien
que pueda mostrar **2-3 proyectos profundos, publicados en GitHub**, con:

- ✅ Tests que pasan (unitarios + integración)
- ✅ Un README claro que explica qué hace y cómo correrlo
- ✅ CI configurado (fmt + clippy + tests en cada push)
- ✅ Código idiomático, no "C escrito en Rust"

Este repo está estructurado para terminar exactamente en eso: los tres proyectos de la
**fase 3** son el portafolio; las fases 1 y 2 son el camino para poder escribirlos
(y defenderlos en una entrevista o llamada con un cliente).

---

## 🗺️ Fases

| Fase | Contenido | Tiempo estimado |
|---|---|---|
| [**Fase 1 — Base sólida**](fase-1-base/) | Repaso profundo: ownership, lifetimes, traits, genéricos, `Result`/`?`, iteradores, closures. Rustlings + The Rust Book. | 2–4 semanas |
| [**Fase 2 — Intermedio real**](fase-2-intermedio/) | Concurrencia (`Arc<Mutex>`, channels), smart pointers (`Box`/`Rc`/`RefCell`), async con Tokio (`spawn`, `select!`). *Rust for Rustaceans* + *Crust of Rust*. | 2–3 meses |
| [**Fase 3 — Proyectos de portafolio**](fase-3-proyectos-portafolio/) | 3 proyectos completos y funcionales: CLI con clap, API REST con Axum + SQLx, mini-blockchain educativa. | 2–3 meses |
| [**Fase 4 — Solana / Anchor**](fase-4-solana-anchor/) | Programas on-chain con Anchor. Solana Playground, Anchor Book, program-examples. | Después de la fase 3 |

Además: [**recursos/**](recursos/) — todos los links, libros y repos de referencia en un solo lugar.

---

## 🔧 Cómo usar este repo

```bash
# Los archivos .rs sueltos de fase 1 y 2 compilan standalone:
rustc fase-1-base/ownership_lifetimes.rs -o /tmp/demo && /tmp/demo

# Los proyectos Cargo se corren como cualquier proyecto:
cd fase-1-base/ejercicios
cargo test          # los ejercicios se "resuelven" haciendo pasar los tests

cd ../../fase-3-proyectos-portafolio/mini-blockchain
cargo run
cargo test
```

El CI del repo (`.github/workflows/ci.yml`) corre `cargo fmt --check`,
`cargo clippy` y `cargo test` sobre todos los proyectos en cada push.
