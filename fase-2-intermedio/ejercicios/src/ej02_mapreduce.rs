//! # Ejercicio 02 — MapReduce casero (threads + channels)
//!
//! **Enunciado**: implementá `contar_palabras_paralelo(texto, n_threads)`:
//! partí el texto en `n_threads` bloques de líneas, contá palabras en cada
//! bloque EN SU PROPIO THREAD (map), y combiná los HashMaps parciales en
//! uno final (reduce). El resultado debe ser idéntico al conteo secuencial.
//!
//! Restricciones:
//! - Los threads NO comparten estado mutable: cada uno cuenta lo suyo y
//!   manda su HashMap parcial por un channel (¡ownership viajando!).
//! - Prohibido `Arc<Mutex<HashMap>>` global: eso serializa el trabajo y
//!   convierte el "paralelo" en secuencial con extra pasos.
//!
//! **Qué practica**: partición de trabajo, `thread::scope` (¡threads que
//! PIDEN PRESTADO del stack!), channels para recolectar, y el merge final.

use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;

/// Conteo secuencial de referencia (la "verdad" contra la que se compara).
pub fn contar_palabras(texto: &str) -> HashMap<String, usize> {
    let mut mapa = HashMap::new();
    for palabra in texto.split_whitespace() {
        *mapa.entry(palabra.to_lowercase()).or_insert(0) += 1;
    }
    mapa
}

/// Versión paralela con map (por bloques) + reduce (merge de parciales).
pub fn contar_palabras_paralelo(texto: &str, n_threads: usize) -> HashMap<String, usize> {
    let lineas: Vec<&str> = texto.lines().collect();
    if lineas.is_empty() || n_threads == 0 {
        return HashMap::new();
    }

    // div_ceil: bloques de tamaño parejo (el último puede quedar corto).
    let tam_bloque = lineas.len().div_ceil(n_threads);

    let (tx, rx) = mpsc::channel::<HashMap<String, usize>>();

    // thread::scope (Rust 1.63+): threads que GARANTIZAN terminar antes de
    // que esta función retorne. Eso le permite al compilador aceptar que
    // los threads tomen PRESTADOS los &str de `lineas` (que apuntan a
    // `texto`) sin exigir 'static ni clonar el texto entero.
    //
    // Con thread::spawn "normal" esto NO compilaría: spawn exige 'static
    // porque el thread podría sobrevivir al caller. scope acota las vidas
    // — es la solución verificada al problema que en C++ resolvés
    // "prometiendo" que harás join antes de que muera el buffer.
    thread::scope(|s| {
        for bloque in lineas.chunks(tam_bloque) {
            let tx = tx.clone();
            s.spawn(move || {
                // MAP: cada thread cuenta su bloque en SU HashMap local.
                // Nada compartido → nada que sincronizar → escala lineal.
                let mut parcial: HashMap<String, usize> = HashMap::new();
                for linea in bloque {
                    for palabra in linea.split_whitespace() {
                        *parcial.entry(palabra.to_lowercase()).or_insert(0) += 1;
                    }
                }
                // El HashMap parcial SE MUEVE por el canal (ownership):
                // cero copias, cero locks compartidos.
                tx.send(parcial).expect("el receptor murió");
            });
        }
        drop(tx); // soltar el Sender original → rx sabrá cuándo terminar

        // REDUCE: fusionamos los parciales a medida que llegan.
        // rx.iter() termina solo cuando el último worker suelta su Sender.
        let mut total: HashMap<String, usize> = HashMap::new();
        for parcial in rx.iter() {
            for (palabra, cuenta) in parcial {
                *total.entry(palabra).or_insert(0) += cuenta;
            }
        }
        total
    }) // el valor del closure de scope es el valor de retorno
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEXTO: &str = "\
el rápido zorro marrón salta
sobre el perro perezoso
el zorro es rápido
y el perro no";

    #[test]
    fn paralelo_coincide_con_secuencial() {
        let esperado = contar_palabras(TEXTO);
        for n in [1, 2, 3, 8] {
            assert_eq!(
                contar_palabras_paralelo(TEXTO, n),
                esperado,
                "difiere con {n} threads"
            );
        }
    }

    #[test]
    fn cuenta_correcta_puntual() {
        let resultado = contar_palabras_paralelo(TEXTO, 2);
        assert_eq!(resultado.get("el"), Some(&4));
        assert_eq!(resultado.get("zorro"), Some(&2));
        assert_eq!(resultado.get("perezoso"), Some(&1));
    }

    #[test]
    fn texto_vacio() {
        assert!(contar_palabras_paralelo("", 4).is_empty());
    }

    #[test]
    fn mas_threads_que_lineas() {
        // 4 líneas, 32 threads: chunks() reparte lo que hay y listo.
        let esperado = contar_palabras(TEXTO);
        assert_eq!(contar_palabras_paralelo(TEXTO, 32), esperado);
    }

    #[test]
    fn cero_threads_no_paniquea() {
        assert!(contar_palabras_paralelo(TEXTO, 0).is_empty());
    }

    #[test]
    fn texto_grande() {
        // Un texto sintético más grande para que el paralelismo trabaje.
        let grande: String = (0..500)
            .map(|i| format!("linea {} con palabras repetidas {}\n", i, i % 7))
            .collect();
        let esperado = contar_palabras(&grande);
        assert_eq!(contar_palabras_paralelo(&grande, 8), esperado);
    }
}
