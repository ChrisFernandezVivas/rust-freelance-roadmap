//! # Ejercicios — Fase 2
//!
//! Cuatro ejercicios de nivel intermedio: mutabilidad interior, threads
//! con channels, y async con Tokio. Misma mecánica que la fase 1:
//! cada módulo trae enunciado + solución de referencia; la forma correcta
//! de ejercitar es **vaciar los cuerpos con `todo!()` y reimplementar**
//! hasta que `cargo test` vuelva a verde.
//!
//! ```bash
//! cargo test          # todo
//! cargo test ej02     # uno solo
//! ```

pub mod ej01_cache_memo;
pub mod ej02_mapreduce;
pub mod ej03_reintentos_async;
pub mod ej04_pipeline_async;
