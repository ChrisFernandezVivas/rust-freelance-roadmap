//! # Ejercicio 04 — Pipeline async con límite de concurrencia
//!
//! **Enunciado**: implementá `procesar_con_limite`, que recibe una lista de
//! "URLs" (strings) y las procesa de forma async, pero **sin superar
//! `max_concurrentes` tareas en vuelo a la vez** — el patrón real de
//! cualquier scraper/cliente HTTP masivo que no quiere tumbar el servidor
//! remoto (ni el propio) abriendo 10.000 conexiones simultáneas.
//!
//! **Qué practica**: `tokio::sync::Semaphore` (control de concurrencia,
//! el async-equivalente de limitar threads en un pool), `join_all` para
//! esperar muchas tasks a la vez, y por qué esto NO es lo mismo que
//! `spawn` sin límite (que sería "lanzar todo y que Dios reparta").

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::time::sleep;

/// Simula descargar una URL: tarda un tiempo fijo y devuelve su longitud.
async fn descargar(url: &str) -> usize {
    sleep(Duration::from_millis(20)).await;
    url.len()
}

/// Procesa todas las `urls` concurrentemente, pero limitando cuántas
/// descargas están EN VUELO a la vez a `max_concurrentes`.
///
/// El mecanismo: un `Semaphore` con `max_concurrentes` permisos. Cada task
/// debe `acquire()` un permiso ANTES de trabajar y lo suelta (drop del
/// guard) al terminar. Si no hay permisos libres, `acquire().await`
/// suspende la task (no bloquea el thread) hasta que alguien libere uno.
///
/// Es el mismo concepto que un pool de N threads en C/C++ (sem_wait /
/// sem_post de POSIX), pero sin threads del SO: el "esperar mi turno" es
/// asíncrono y no cuesta memoria de stack por cada URL en cola.
pub async fn procesar_con_limite(urls: Vec<String>, max_concurrentes: usize) -> Vec<usize> {
    let semaforo = Arc::new(Semaphore::new(max_concurrentes));
    // Para poder VERIFICAR en los tests que el límite se respetó de
    // verdad (no solo confiar en que "debería andar"): llevamos la cuenta
    // de cuántas tasks están activas AHORA MISMO, y el máximo histórico.
    let activas = Arc::new(AtomicUsize::new(0));
    let pico_maximo = Arc::new(AtomicUsize::new(0));

    let mut handles = Vec::new();
    for url in urls {
        let semaforo = Arc::clone(&semaforo);
        let activas = Arc::clone(&activas);
        let pico_maximo = Arc::clone(&pico_maximo);

        handles.push(tokio::spawn(async move {
            // acquire_owned: el permiso viaja CON la task (no con una
            // referencia prestada), necesario porque esta task es 'static
            // (spawneada). Se libera automáticamente al final del scope.
            let _permiso = semaforo.acquire_owned().await.expect("semáforo cerrado");

            let ahora = activas.fetch_add(1, Ordering::SeqCst) + 1;
            pico_maximo.fetch_max(ahora, Ordering::SeqCst);

            let resultado = descargar(&url).await;

            activas.fetch_sub(1, Ordering::SeqCst);
            resultado
        })); // ← _permiso se dropea acá, liberando el turno para el próximo
    }

    let mut resultados = Vec::with_capacity(handles.len());
    for h in handles {
        resultados.push(h.await.expect("una task paniqueó"));
    }

    // Guardamos el pico en un campo "oculto" accesible solo para tests
    // sería más limpio devolverlo, pero mantenemos la firma simple para
    // el caso de uso real; los tests usan la variante instrumentada de
    // abajo.
    resultados
}

/// Variante que además devuelve el pico de concurrencia observado —
/// existe para poder ESCRIBIR UN TEST que de verdad demuestre que el
/// límite se respetó (no alcanza con que el resultado final sea correcto).
pub async fn procesar_con_limite_instrumentado(
    urls: Vec<String>,
    max_concurrentes: usize,
) -> (Vec<usize>, usize) {
    let semaforo = Arc::new(Semaphore::new(max_concurrentes));
    let activas = Arc::new(AtomicUsize::new(0));
    let pico_maximo = Arc::new(AtomicUsize::new(0));

    let mut handles = Vec::new();
    for url in urls {
        let semaforo = Arc::clone(&semaforo);
        let activas = Arc::clone(&activas);
        let pico_maximo = Arc::clone(&pico_maximo);

        handles.push(tokio::spawn(async move {
            let _permiso = semaforo.acquire_owned().await.expect("semáforo cerrado");
            let ahora = activas.fetch_add(1, Ordering::SeqCst) + 1;
            pico_maximo.fetch_max(ahora, Ordering::SeqCst);
            let resultado = descargar(&url).await;
            activas.fetch_sub(1, Ordering::SeqCst);
            resultado
        }));
    }

    let mut resultados = Vec::with_capacity(handles.len());
    for h in handles {
        resultados.push(h.await.expect("una task paniqueó"));
    }

    let pico = pico_maximo.load(Ordering::SeqCst);
    (resultados, pico)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn procesa_todas_las_urls() {
        let urls = vec!["a".into(), "bb".into(), "ccc".into()];
        let resultados = procesar_con_limite(urls, 2).await;
        assert_eq!(resultados, vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn respeta_el_limite_de_concurrencia() {
        let urls: Vec<String> = (0..20).map(|i| "x".repeat(i + 1)).collect();
        let (_, pico) = procesar_con_limite_instrumentado(urls, 4).await;
        // El pico NUNCA debe superar el límite pedido:
        assert!(pico <= 4, "pico de concurrencia fue {pico}, límite era 4");
        // Y con 20 tareas y descargas no instantáneas, debería haberse
        // usado el límite completo al menos una vez (si no, el test de
        // arriba sería vacuamente verdadero).
        assert_eq!(pico, 4);
    }

    #[tokio::test]
    async fn lista_vacia() {
        let resultados = procesar_con_limite(Vec::new(), 5).await;
        assert!(resultados.is_empty());
    }

    #[tokio::test]
    async fn limite_mayor_que_cantidad_de_urls() {
        let urls = vec!["uno".into(), "dos".into()];
        let (resultados, pico) = procesar_con_limite_instrumentado(urls, 100).await;
        assert_eq!(resultados, vec![3, 3]);
        assert!(pico <= 2);
    }

    #[tokio::test]
    async fn el_paralelismo_real_reduce_el_tiempo_total() {
        use tokio::time::Instant;
        // 8 URLs de 20ms cada una, límite de concurrencia 8 → deberían
        // solaparse casi todas: tiempo total cercano a 20ms, no 160ms.
        let urls: Vec<String> = (0..8).map(|i| format!("url-{i}")).collect();
        let inicio = Instant::now();
        procesar_con_limite(urls, 8).await;
        assert!(inicio.elapsed() < Duration::from_millis(150));
    }
}
