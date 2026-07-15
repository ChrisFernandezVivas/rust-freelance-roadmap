//! # mini-blockchain — blockchain educativa
//!
//! Implementa los mecanismos centrales de una blockchain simplificada:
//! bloques encadenados por hash SHA-256, proof-of-work con dificultad
//! ajustable, y validación de la cadena completa (incluida la detección
//! de manipulación de bloques intermedios).
//!
//! **Advertencia de alcance** (ver `fase-4-solana-anchor/README.md` para
//! el detalle): esto es un ejercicio EDUCATIVO para entender el mecanismo.
//! El mercado freelance real de Web3 en Rust está en programas de Solana
//! con Anchor, no en escribir blockchains desde cero.

pub mod bloque;
pub mod cadena;

pub use bloque::Bloque;
pub use cadena::Cadena;
