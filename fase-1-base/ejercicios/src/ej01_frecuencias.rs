//! # Ejercicio 01 — Frecuencias de palabras (ownership + HashMap)
//!
//! **Enunciado**: implementá `frecuencias`, que recibe un texto y devuelve
//! un `HashMap<String, usize>` con cuántas veces aparece cada palabra.
//! Reglas:
//! - Insensible a mayúsculas ("Hola" y "hola" son la misma palabra).
//! - Ignorar signos de puntuación pegados a la palabra (",", ".", "!", "?", ";", ":", "¡", "¿").
//! - Las palabras vacías (resultado de limpiar solo puntuación) no cuentan.
//!
//! **Qué practica**: ownership al construir Strings nuevos desde &str
//! prestados, y la API `entry()` de HashMap — el patrón idiomático para
//! "insertar o actualizar" en UNA sola búsqueda de hash (en C++ harías
//! `map[key]++`, que también es una sola búsqueda, pero en Rust el acceso
//! indexado a un HashMap con clave inexistente no existe: te obliga a
//! decidir qué pasa si la clave no está).

use std::collections::HashMap;

/// Cuenta la frecuencia de cada palabra normalizada del texto.
pub fn frecuencias(texto: &str) -> HashMap<String, usize> {
    let mut mapa = HashMap::new();

    for palabra in texto.split_whitespace() {
        // trim_matches limpia la puntuación de AMBOS extremos sin alocar:
        // devuelve un &str que apunta adentro de `palabra` (¡lifetimes!).
        // Ojo: incluimos ¡ y ¿ — en español la puntuación también abre.
        let limpia = palabra.trim_matches(|c: char| ",.!?;:¡¿".contains(c));

        if limpia.is_empty() {
            continue;
        }

        // Recién acá alocamos: to_lowercase crea un String nuevo, y el
        // HashMap se vuelve DUEÑO de esa clave (por eso HashMap<String, _>
        // y no HashMap<&str, _>: si guardáramos &str, el mapa no podría
        // sobrevivir al texto de entrada).
        let clave = limpia.to_lowercase();

        // entry(): si la clave existe devuelve su entrada; si no, la crea
        // con or_insert(0). Una sola búsqueda, sin unwrap, sin doble lookup.
        *mapa.entry(clave).or_insert(0) += 1;
    }

    mapa
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cuenta_basica() {
        let f = frecuencias("hola mundo hola");
        assert_eq!(f.get("hola"), Some(&2));
        assert_eq!(f.get("mundo"), Some(&1));
    }

    #[test]
    fn insensible_a_mayusculas() {
        let f = frecuencias("Rust rust RUST");
        assert_eq!(f.get("rust"), Some(&3));
        assert_eq!(f.len(), 1);
    }

    #[test]
    fn ignora_puntuacion() {
        let f = frecuencias("¡vamos! rust, rust. rust; ¿rust?");
        // "rust," "rust." "rust;" y "¿rust?" cuentan como la misma palabra.
        assert_eq!(f.get("rust"), Some(&4));
        assert_eq!(f.get("vamos"), Some(&1));
    }

    #[test]
    fn texto_vacio() {
        assert!(frecuencias("").is_empty());
        assert!(frecuencias("   \n\t  ").is_empty());
    }

    #[test]
    fn puntuacion_sola_no_cuenta() {
        let f = frecuencias("hola ... !!! mundo");
        assert_eq!(f.len(), 2);
        assert_eq!(f.get("hola"), Some(&1));
        assert_eq!(f.get("mundo"), Some(&1));
    }
}
