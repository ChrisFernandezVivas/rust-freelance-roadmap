//! # Concurrencia clásica: threads, `Arc<Mutex>` y channels mpsc
//!
//! La promesa central de Rust: **"fearless concurrency"**. Los data races
//! que en C/C++ se cazan con TSan, printfs y suerte, acá son errores de
//! COMPILACIÓN. Los dos traits que lo hacen posible:
//!
//! - `Send`: el tipo puede MOVERSE a otro thread.
//! - `Sync`: el tipo puede COMPARTIRSE (&T) entre threads.
//!
//! No los implementás a mano: el compilador los deriva y los EXIGE en las
//! firmas de `thread::spawn` y compañía. `Rc` no es Send → usarlo entre
//! threads no compila. `Arc<Mutex<T>>` sí → compila. Fin de la discusión,
//! antes de correr una sola instrucción.

use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

// ============================================================================
// PARTE 1: estado compartido con Arc<Mutex<T>>
// ============================================================================

/// Estadísticas compartidas entre workers.
///
/// La composición `Arc<Mutex<T>>` se lee de afuera hacia adentro:
/// - `Arc` (Atomically Reference Counted): como `Rc` pero con contador
///   atómico → varios threads pueden ser DUEÑOS del mismo dato.
///   Equivalente a `std::shared_ptr` de C++ (cuyo contador sí es atómico).
/// - `Mutex<T>`: y acá la GRAN diferencia con C/C++/Python: el mutex
///   **contiene** al dato. En pthreads o std::mutex, el mutex y el dato
///   están sueltos y la relación "este mutex protege esta variable" vive
///   en un comentario. En Rust es imposible tocar la T sin hacer lock():
///   el compilador convierte la convención en regla.
#[derive(Debug, Default)]
pub struct Estadisticas {
    pub procesadas: u64,
    pub errores: u64,
}

/// Incrementa el contador compartido desde `n_threads` threads, `veces`
/// cada uno. El clásico ejemplo del contador — el "hola mundo" de los
/// data races: en C con `contador++` sin lock, el resultado es aleatorio.
/// Acá es determinista o no compila.
pub fn contar_en_paralelo(n_threads: usize, veces: u64) -> u64 {
    let contador = Arc::new(Mutex::new(0u64));
    let mut handles = Vec::new();

    for _ in 0..n_threads {
        // Cada thread recibe SU clon del Arc (incremento atómico del
        // contador de referencias, no copia del dato).
        let contador = Arc::clone(&contador);

        // `move` obliga al closure a APROPIARSE de `contador` (su clon).
        // Sin move, el closure tomaría una referencia al stack de main —
        // y el thread podría sobrevivir a ese stack. En C++ ese es
        // exactamente el bug de capturar por [&] en un std::thread.
        handles.push(thread::spawn(move || {
            for _ in 0..veces {
                // lock() devuelve Result (Err si otro thread paniqueó con
                // el lock tomado — "mutex envenenado"). El MutexGuard que
                // obtenemos destrabas el mutex SOLO al salir de scope:
                // olvidarse el unlock es imposible (RAII, como
                // std::lock_guard, pero sin poder acceder al dato sin él).
                let mut guardia = contador.lock().expect("mutex envenenado");
                *guardia += 1;
            } // ← unlock automático acá, en cada iteración
        }));
    }

    // join(): esperar a todos. El Result externo es Err si el thread paniqueó.
    for h in handles {
        h.join().expect("un worker paniqueó");
    }

    // Recuperamos el valor final. try_unwrap: si somos el ÚNICO dueño
    // restante del Arc, nos devuelve el Mutex; into_inner lo desarma.
    Arc::try_unwrap(contador)
        .expect("quedan otros dueños del Arc")
        .into_inner()
        .expect("mutex envenenado")
}

// ============================================================================
// PARTE 2: paso de mensajes con channels mpsc
// ============================================================================
// mpsc = Multi-Producer, Single-Consumer. Filosofía opuesta al Mutex:
// "no compartas memoria para comunicarte; comunicate para compartir
// memoria" (robado de Go, que lo robó de Hoare/CSP).
//
// La gracia en Rust: send() MUEVE el valor por el canal (ownership).
// Después de enviar algo, el productor YA NO PUEDE tocarlo — no compila.
// En C++ pasar un puntero por una cola y seguir usándolo del otro lado
// compila perfecto y es una carrera esperando su momento.
// ============================================================================

/// Un trabajo de procesamiento: parsear la lectura cruda de un sensor.
#[derive(Debug)]
pub struct Trabajo {
    pub id: u32,
    pub crudo: String,
}

/// Resultado del procesamiento de un trabajo.
#[derive(Debug, PartialEq)]
pub struct Procesado {
    pub id: u32,
    pub valor: f64,
}

/// Procesa trabajos en paralelo con un pool de workers y DOS channels:
///
/// ```text
///                 channel de trabajos            channel de resultados
///   productor ──► (1 receptor compartido  ──► [worker × n] ──► (n emisores,
///                  vía Arc<Mutex<Receiver>>)                    1 receptor)
/// ```
///
/// - Trabajos → workers: mpsc es single-consumer, así que el Receiver se
///   comparte entre workers con `Arc<Mutex<Receiver>>` (patrón estándar
///   de pool; los crates como crossbeam traen canales multi-consumer).
/// - Workers → resultados: acá mpsc brilla — MUCHOS productores (un Sender
///   clonado por worker) y UN consumidor (esta función).
pub fn procesar_pool(trabajos: Vec<Trabajo>, n_workers: usize) -> Vec<Procesado> {
    let (tx_trabajo, rx_trabajo) = mpsc::channel::<Trabajo>();
    let (tx_result, rx_result) = mpsc::channel::<Procesado>();

    let rx_trabajo = Arc::new(Mutex::new(rx_trabajo));

    let mut handles = Vec::new();
    for _ in 0..n_workers {
        let rx = Arc::clone(&rx_trabajo);
        let tx = tx_result.clone(); // mpsc: clonar el Sender es la forma "MP"

        handles.push(thread::spawn(move || {
            loop {
                // Tomamos el lock SOLO para sacar un trabajo, y lo soltamos
                // ANTES de procesar (el bloque interior limita el scope del
                // guard). Regla de oro: secciones críticas cortas.
                let trabajo = { rx.lock().expect("mutex envenenado").recv() };

                match trabajo {
                    // recv() devuelve Err cuando TODOS los Senders murieron:
                    // ese es el protocolo de apagado, sin flags ni señales.
                    Err(_) => break,
                    Ok(t) => {
                        // Entrada corrupta → la salteamos (en producción
                        // la mandaríamos a un canal de errores).
                        if let Ok(valor) = t.crudo.trim().parse::<f64>() {
                            // send() MUEVE el Procesado al canal. Si el
                            // receptor murió, send falla; acá lo ignoramos
                            // porque implica que ya nadie espera resultados.
                            let _ = tx.send(Procesado { id: t.id, valor });
                        }
                    }
                }
            }
        }));
    }

    // Encolamos todos los trabajos...
    for t in trabajos {
        tx_trabajo
            .send(t)
            .expect("los workers murieron antes de tiempo");
    }

    // ...y CERRAMOS los emisores originales (drop explícito). Esto es lo
    // que hace que los recv() de arriba devuelvan Err y los workers salgan
    // del loop. Sin estos drops, deadlock: todos esperando eternamente.
    drop(tx_trabajo);
    drop(tx_result); // los workers tienen sus clones; este era el nuestro

    // Drenamos resultados hasta que el último worker suelte su Sender.
    let resultados: Vec<Procesado> = rx_result.iter().collect();

    for h in handles {
        h.join().expect("un worker paniqueó");
    }

    resultados
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn el_contador_no_pierde_incrementos() {
        // 8 threads × 10_000 incrementos. Sin Mutex (en C, contador++ a
        // secas) esto daría un número distinto en cada corrida.
        assert_eq!(contar_en_paralelo(8, 10_000), 80_000);
    }

    #[test]
    fn contador_con_un_solo_thread() {
        assert_eq!(contar_en_paralelo(1, 5), 5);
    }

    #[test]
    fn el_pool_procesa_todo() {
        let trabajos: Vec<Trabajo> = (0..100)
            .map(|i| Trabajo {
                id: i,
                crudo: format!("{}.5", i),
            })
            .collect();

        let mut resultados = procesar_pool(trabajos, 4);

        // Los resultados llegan en CUALQUIER orden (4 workers compitiendo):
        // ordenar antes de comparar es parte de la lección.
        resultados.sort_by_key(|p| p.id);

        assert_eq!(resultados.len(), 100);
        assert_eq!(resultados[7], Procesado { id: 7, valor: 7.5 });
        assert_eq!(
            resultados[99],
            Procesado {
                id: 99,
                valor: 99.5
            }
        );
    }

    #[test]
    fn el_pool_descarta_entradas_corruptas() {
        let trabajos = vec![
            Trabajo {
                id: 1,
                crudo: "1.0".into(),
            },
            Trabajo {
                id: 2,
                crudo: "no soy un número".into(),
            },
            Trabajo {
                id: 3,
                crudo: "3.0".into(),
            },
        ];

        let mut resultados = procesar_pool(trabajos, 2);
        resultados.sort_by_key(|p| p.id);

        assert_eq!(resultados.len(), 2);
        assert_eq!(resultados[0].id, 1);
        assert_eq!(resultados[1].id, 3);
    }

    #[test]
    fn pool_con_mas_workers_que_trabajos() {
        // Caso borde: workers ociosos deben terminar igual (sin deadlock).
        let trabajos = vec![Trabajo {
            id: 1,
            crudo: "2.5".into(),
        }];
        let resultados = procesar_pool(trabajos, 8);
        assert_eq!(resultados, vec![Procesado { id: 1, valor: 2.5 }]);
    }

    #[test]
    fn pool_sin_trabajos() {
        let resultados = procesar_pool(Vec::new(), 4);
        assert!(resultados.is_empty());
    }
}
