//! Demo ejecutable del proyecto: `cargo run`
//!
//! Muestra los dos patrones de la lib en acción con salida por consola.

use concurrencia::{contar_en_paralelo, procesar_pool, Trabajo};

fn main() {
    // --- Patrón 1: estado compartido (Arc<Mutex>) ---
    println!("== Contador compartido ==");
    let total = contar_en_paralelo(8, 250_000);
    println!("8 threads × 250k incrementos = {total} (exacto, siempre)\n");

    // --- Patrón 2: paso de mensajes (channels mpsc) ---
    println!("== Pool de workers con channels ==");
    let trabajos: Vec<Trabajo> = (1..=10)
        .map(|i| Trabajo {
            id: i,
            crudo: format!("{}", f64::from(i) * 1.1),
        })
        .collect();

    let mut resultados = procesar_pool(trabajos, 3);
    resultados.sort_by_key(|p| p.id);

    for p in &resultados {
        println!("trabajo {:>2} → {:.2}", p.id, p.valor);
    }
    println!("({} trabajos procesados por 3 workers)", resultados.len());
}
