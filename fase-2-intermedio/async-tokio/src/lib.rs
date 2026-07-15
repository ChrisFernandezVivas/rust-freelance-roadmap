//! # Async con Tokio: `spawn`, `tokio::sync` y `select!`
//!
//! ## Threads vs. async — ¿cuándo cada uno?
//!
//! El proyecto hermano `concurrencia/` usa threads del sistema operativo:
//! ideales para trabajo **CPU-bound** (cada thread usa un core de verdad).
//! Async/Tokio es para trabajo **IO-bound**: miles de operaciones que pasan
//! el 99% del tiempo ESPERANDO (sockets, discos, timers).
//!
//! - Un thread del SO cuesta ~MB de stack y un context switch del kernel.
//! - Una task de Tokio cuesta ~cientos de bytes y un salto de función.
//!   Podés tener MILLONES de tasks; con threads, miles ya duele.
//!
//! ## El modelo, comparado con lo que conocés
//!
//! - **Python asyncio**: la sintaxis async/await es casi idéntica. La
//!   diferencia: no hay GIL ni intérprete — el runtime de Tokio ejecuta
//!   tasks en un pool de threads reales, en paralelo de verdad.
//! - **C/C++**: async de Rust compila cada `async fn` a una **máquina de
//!   estados** (un enum con una variante por cada punto de `await`). Es lo
//!   que escribirías a mano con epoll + callbacks + structs de contexto,
//!   pero generado por el compilador y sin poder equivocarte de estado.
//!
//! Una `async fn` NO ejecuta nada al llamarla: devuelve un `Future` inerte
//! (lazy, como los iteradores). Alguien tiene que hacerle `.await` o
//! `spawn`earla para que avance.

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot, Mutex};
use tokio::time::sleep;

// ============================================================================
// PARTE 1: tokio::spawn — lanzar tasks concurrentes
// ============================================================================

/// Simula consultar un sensor remoto: tarda `demora_ms` y devuelve un valor.
/// El `.await` en el sleep CEDE el control al runtime: mientras este sensor
/// "espera", el mismo thread ejecuta otras tasks. Eso es cooperative
/// scheduling — nadie bloquea a nadie (¡por eso JAMÁS se usa
/// std::thread::sleep dentro de código async: ese sí bloquea el thread!).
pub async fn leer_sensor_remoto(id: u32, demora_ms: u64) -> f64 {
    sleep(Duration::from_millis(demora_ms)).await;
    // Valor sintético determinista para poder testear.
    f64::from(id) * 1.5
}

/// Lee `n` sensores EN PARALELO y devuelve la suma de sus lecturas.
///
/// La versión secuencial tardaría n × demora; esta tarda ~1 × demora,
/// porque todas las esperas se solapan. Este patrón (spawn N + join N)
/// es el 80% del async que vas a escribir en backend.
pub async fn leer_todos(n: u32, demora_ms: u64) -> f64 {
    let mut handles = Vec::new();

    for id in 0..n {
        // tokio::spawn es el gemelo async de thread::spawn:
        // - recibe un Future (no un closure) y lo ejecuta "en background"
        // - devuelve un JoinHandle que también es un Future
        // - exige 'static + Send, igual que los threads: la task puede
        //   sobrevivir a esta función y migrar de thread entre awaits.
        handles.push(tokio::spawn(leer_sensor_remoto(id, demora_ms)));
    }

    let mut suma = 0.0;
    for h in handles {
        // .await del JoinHandle espera a que la task termine.
        // Devuelve Result: Err solo si la task PANIQUEÓ (como join()).
        suma += h.await.expect("una task paniqueó");
    }
    suma
}

// ============================================================================
// PARTE 2: tokio::sync::Mutex — estado compartido entre tasks
// ============================================================================
// ¿Por qué NO usar std::sync::Mutex acá? Porque su lock() BLOQUEA EL THREAD
// entero, y con él a todas las demás tasks de ese thread (y si el runtime
// tiene pocos threads: deadlock potencial). El Mutex de tokio es async:
// lock().await SUSPENDE la task y libera el thread para otras.
//
// Regla práctica (de la propia doc de tokio):
// - Sección crítica corta y SIN awaits adentro → std::sync::Mutex está bien.
// - Necesitás mantener el lock A TRAVÉS de un .await → tokio::sync::Mutex.
// ============================================================================

/// Registro de eventos compartido entre tasks (el dato vive DENTRO del
/// mutex, igual que en el proyecto de threads — misma filosofía, otro lock).
#[derive(Debug, Default)]
pub struct Registro {
    pub eventos: Vec<String>,
}

/// Lanza `n` tasks que escriben en un registro compartido y espera a todas.
pub async fn registrar_en_paralelo(n: u32) -> Arc<Mutex<Registro>> {
    let registro = Arc::new(Mutex::new(Registro::default()));

    let mut handles = Vec::new();
    for i in 0..n {
        let registro = Arc::clone(&registro); // mismo patrón que con threads
        handles.push(tokio::spawn(async move {
            // Simulamos algo de trabajo async antes de escribir:
            sleep(Duration::from_millis(1)).await;
            // lock().await: si otra task tiene el lock, nos suspendemos
            // (el thread queda libre). Nada de spinear ni bloquear.
            let mut guardia = registro.lock().await;
            guardia.eventos.push(format!("task {i} reportó"));
        }));
    }

    for h in handles {
        h.await.expect("una task paniqueó");
    }
    registro
}

// ============================================================================
// PARTE 3: select! — esperar VARIAS cosas a la vez, ganar con la primera
// ============================================================================
// select! es el superpoder de async que los threads no tienen barato:
// "esperá A y B y C; lo primero que termine gana, cancelá el resto".
// Con threads esto es doloroso (¿cómo matás un thread bloqueado?); acá los
// futures perdedores simplemente se DROPEAN (cancelación por drop — gratis
// porque un future suspendido es solo un struct en memoria).
// ============================================================================

#[derive(Debug, PartialEq)]
pub enum Lectura {
    /// El sensor respondió a tiempo.
    Valor(f64),
    /// El sensor tardó más que el timeout.
    Timeout,
}

/// Lee un sensor con un timeout, implementado A MANO con select! para ver
/// el mecanismo (en producción usarías tokio::time::timeout, que es esto).
pub async fn leer_con_timeout(id: u32, demora_ms: u64, timeout_ms: u64) -> Lectura {
    tokio::select! {
        // Rama 1: la lectura del sensor.
        valor = leer_sensor_remoto(id, demora_ms) => Lectura::Valor(valor),
        // Rama 2: el timeout. Si esta "gana", el future de la rama 1 se
        // dropea → la lectura queda cancelada. Sin flags, sin kill().
        _ = sleep(Duration::from_millis(timeout_ms)) => Lectura::Timeout,
    }
}

/// Worker con apagado limpio (graceful shutdown) — EL patrón de servicio:
///
/// - procesa trabajos que llegan por un canal mpsc (async: recv().await)
/// - hasta que llegue la señal por el canal oneshot de apagado
///
/// Devuelve cuántos trabajos procesó. Fijate que el loop entero es UN
/// select!: "o llega trabajo, o llega la orden de apagar" — la forma
/// idiomática de estructurar cualquier servicio de larga vida en Tokio
/// (servidores HTTP, consumidores de colas, bots...).
pub async fn worker_con_apagado(
    mut trabajos: mpsc::Receiver<f64>,
    mut apagado: oneshot::Receiver<()>,
) -> u64 {
    let mut procesados = 0u64;

    loop {
        tokio::select! {
            // Rama trabajo: recv() devuelve None si TODOS los Senders
            // murieron — el mismo protocolo de cierre que en mpsc de std.
            trabajo = trabajos.recv() => {
                match trabajo {
                    Some(_valor) => procesados += 1,
                    None => break, // canal cerrado: no habrá más trabajo
                }
            }
            // Rama apagado: la señal puede llegar en cualquier momento,
            // incluso con trabajos aún encolados (apagado "abrupto pero
            // limpio"; drenar la cola primero sería la variante graceful).
            _ = &mut apagado => break,
        }
    }
    procesados
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[tokio::test] arma un runtime por test — el equivalente de
    // #[test] para funciones async. Sin esto, no hay quién ejecute awaits.
    #[tokio::test]
    async fn spawn_lee_en_paralelo() {
        // 5 sensores: 0*1.5 + 1*1.5 + 2*1.5 + 3*1.5 + 4*1.5 = 15.0
        let suma = leer_todos(5, 10).await;
        assert!((suma - 15.0).abs() < 1e-9);
    }

    #[tokio::test]
    async fn el_paralelismo_solapa_las_esperas() {
        use tokio::time::Instant;
        // 8 sensores de 50 ms cada uno: secuencial serían 400 ms.
        // En paralelo debe tardar ~50 ms (dejamos margen generoso para CI).
        let inicio = Instant::now();
        leer_todos(8, 50).await;
        let transcurrido = inicio.elapsed();
        assert!(
            transcurrido < Duration::from_millis(300),
            "tardó {transcurrido:?}: las lecturas no se solaparon"
        );
    }

    #[tokio::test]
    async fn mutex_async_no_pierde_eventos() {
        let registro = registrar_en_paralelo(50).await;
        assert_eq!(registro.lock().await.eventos.len(), 50);
    }

    #[tokio::test]
    async fn select_devuelve_valor_si_llega_a_tiempo() {
        // demora 10 ms < timeout 200 ms → gana la lectura
        let r = leer_con_timeout(4, 10, 200).await;
        assert_eq!(r, Lectura::Valor(6.0));
    }

    #[tokio::test]
    async fn select_corta_por_timeout() {
        // demora 200 ms > timeout 10 ms → gana el timeout y la lectura
        // se cancela (su future se dropea sin terminar).
        let r = leer_con_timeout(4, 200, 10).await;
        assert_eq!(r, Lectura::Timeout);
    }

    #[tokio::test]
    async fn worker_procesa_y_termina_al_cerrar_canal() {
        let (tx, rx) = mpsc::channel(16);
        let (_tx_apagado, rx_apagado) = oneshot::channel();

        let worker = tokio::spawn(worker_con_apagado(rx, rx_apagado));

        for i in 0..7 {
            tx.send(f64::from(i)).await.unwrap();
        }
        drop(tx); // cerrar el canal = "no hay más trabajo"

        assert_eq!(worker.await.unwrap(), 7);
    }

    #[tokio::test]
    async fn worker_obedece_la_senal_de_apagado() {
        let (tx, rx) = mpsc::channel(16);
        let (tx_apagado, rx_apagado) = oneshot::channel();

        let worker = tokio::spawn(worker_con_apagado(rx, rx_apagado));

        tx.send(1.0).await.unwrap();
        tx.send(2.0).await.unwrap();
        // Le damos tiempo a procesar y mandamos la señal de apagado
        // SIN cerrar el canal de trabajos (tx sigue vivo):
        sleep(Duration::from_millis(50)).await;
        tx_apagado.send(()).unwrap();

        let procesados = worker.await.unwrap();
        assert_eq!(procesados, 2); // terminó por la señal, no por el canal
        drop(tx);
    }
}
