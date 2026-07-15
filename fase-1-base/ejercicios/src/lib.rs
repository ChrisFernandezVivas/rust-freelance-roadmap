//! # Ejercicios — Fase 1
//!
//! Seis ejercicios estilo Rustlings pero MÁS difíciles, pensados para alguien
//! que ya conoce la sintaxis básica y quiere consolidar los conceptos que
//! separan a un junior de un intermedio.
//!
//! ## Cómo usarlos
//!
//! Cada módulo incluye:
//! 1. El **enunciado** en el doc-comment del módulo.
//! 2. Una **solución de referencia** (para que `cargo test` pase y el CI
//!    verifique que todo el repo compila).
//!
//! La forma correcta de ejercitar: **borrá el cuerpo de las funciones**,
//! reemplazalo por `todo!()`, y volvé a implementarlas hasta que
//! `cargo test` pase de nuevo. Después compará con la referencia.
//!
//! ```bash
//! cargo test                 # correr todo
//! cargo test ej03            # correr solo un ejercicio
//! ```

pub mod ej01_frecuencias;
pub mod ej02_pila_generica;
pub mod ej03_parser_config;
pub mod ej04_pipeline_sensores;
pub mod ej05_lifetimes_busqueda;
pub mod ej06_figuras_dyn;
