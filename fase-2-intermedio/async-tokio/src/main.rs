//! Demo ejecutable: `cargo run`
//!
//! Muestra los tres patrones de la lib con tiempos reales por consola.

use async_tokio::{leer_con_timeout, leer_todos, registrar_en_paralelo, Lectura};
use tokio::time::Instant;

// #[tokio::main] transforma este main en:
//   fn main() { Runtime::new().block_on(async { ... }) }
// Es decir: arma el runtime multi-thread y le entrega el future raíz.
#[tokio::main]
async fn main() {
    println!("== spawn: 8 sensores de 100 ms en paralelo ==");
    let inicio = Instant::now();
    let suma = leer_todos(8, 100).await;
    println!(
        "suma = {suma}, tardó {:?} (secuencial serían ~800 ms)\n",
        inicio.elapsed()
    );

    println!("== tokio::sync::Mutex: 20 tasks escriben un registro ==");
    let registro = registrar_en_paralelo(20).await;
    println!(
        "eventos registrados: {}\n",
        registro.lock().await.eventos.len()
    );

    println!("== select!: carrera lectura vs timeout ==");
    match leer_con_timeout(7, 20, 100).await {
        Lectura::Valor(v) => println!("sensor rápido → valor {v}"),
        Lectura::Timeout => println!("sensor rápido → timeout (!?)"),
    }
    match leer_con_timeout(7, 500, 100).await {
        Lectura::Valor(v) => println!("sensor lento → valor {v} (!?)"),
        Lectura::Timeout => println!("sensor lento → timeout (cancelado por select!)"),
    }
}
