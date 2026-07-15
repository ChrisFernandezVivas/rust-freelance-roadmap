//! Demo ejecutable: `cargo run` — mina una cadena de ejemplo, la muestra,
//! y luego demuestra en vivo cómo la manipulación de un bloque la invalida.

use mini_blockchain::Cadena;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

fn ahora() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("el reloj retrocedió")
        .as_secs()
}

fn main() {
    let dificultad = 4; // 4 ceros hexadecimales iniciales
    println!("== Minando una cadena con dificultad {dificultad} ==\n");

    let mut cadena = Cadena::nueva(dificultad);

    for (i, data) in [
        "Alice paga 10 a Bob",
        "Bob paga 3 a Carol",
        "Carol paga 7 a Alice",
    ]
    .into_iter()
    .enumerate()
    {
        let inicio = Instant::now();
        cadena.agregar_bloque(data.to_string(), ahora());
        let bloque = &cadena.bloques[i + 1];
        println!(
            "bloque {} minado en {:?} (nonce={}): {}",
            bloque.index,
            inicio.elapsed(),
            bloque.nonce,
            bloque.hash
        );
    }

    println!("\ncadena válida: {}", cadena.es_valida());

    println!("\n== Manipulando el bloque 1 (sin re-minar) ==");
    cadena.bloques[1].data = "Alice paga 10000 a Bob".to_string();
    println!(
        "cadena válida después de la manipulación: {}",
        cadena.es_valida()
    );

    // Bonus: los bloques derivan Serialize/Deserialize (serde), así que
    // la cadena se puede persistir/transmitir como JSON sin escribir
    // ningún parser a mano — el mismo patrón visto con Serde en el
    // to-do list de persistencia que ya construiste antes de este repo.
    println!("\n== Bloque génesis como JSON ==");
    println!(
        "{}",
        serde_json::to_string_pretty(&cadena.bloques[0]).expect("serialización infalible")
    );
}
