//! # textkit — buscador y reemplazador de texto en archivos
//!
//! La lógica vive en una lib (`buscar`, `reemplazar`) separada del binario
//! (`main.rs`, que solo parsea argumentos con `clap` y llama a estas
//! funciones). Esto permite testear la lógica con `cargo test` normal
//! (units tests rápidos) Y con tests de integración en `tests/` que
//! ejercitan el binario compilado de punta a punta.

pub mod buscar;
pub mod reemplazar;
