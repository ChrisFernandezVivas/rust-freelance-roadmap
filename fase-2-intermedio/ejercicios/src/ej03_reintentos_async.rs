//! # Ejercicio 03 — Reintentos con backoff (async + Tokio)
//!
//! **Enunciado**: implementá `con_reintentos`, una función genérica que
//! recibe una operación async que puede fallar y la reintenta con
//! **backoff exponencial** (esperar más entre cada intento) hasta:
//! - que tenga éxito, o
//! - que se agoten los intentos (devuelve el último error).
//!
//! Esto es EXACTAMENTE lo que necesitás en un backend real hablando con
//! una base de datos o una API externa que a veces devuelve 503.
//!
//! **Qué practica**: genéricos sobre closures que devuelven `Future`
//! (`Fn() -> Fut`, `Fut: Future<Output = ...>` — el patrón más retorcido
//! de firma que vas a escribir en Rust async, y vale la pena entenderlo
//! una vez en profundidad), `tokio::time::sleep`, y propagación de errores
//! async.

use std::future::Future;
use std::time::Duration;
use tokio::time::sleep;

/// Reintenta `operacion` hasta `max_intentos` veces con backoff exponencial.
///
/// Firma explicada de afuera hacia adentro:
/// - `F: Fn() -> Fut`        → F es una closure que, LLAMADA, devuelve un Future
///   (no ES un future: la necesitamos poder invocar de nuevo en cada intento,
///   por eso no recibimos directamente un Future ya construido).
/// - `Fut: Future<Output = Result<T, E>>` → ese future, al completarse,
///   produce un Result.
///
/// En C++ el equivalente sería una plantilla que recibe un `std::function`
/// que devuelve una tarea; la diferencia es que acá el compilador verifica
/// TODO esto en tiempo de compilación, tipo por tipo, sin type erasure.
pub async fn con_reintentos<F, Fut, T, E>(
    mut operacion: F,
    max_intentos: u32,
    espera_inicial_ms: u64,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    let mut espera = espera_inicial_ms;

    for intento in 1..=max_intentos {
        match operacion().await {
            Ok(valor) => return Ok(valor),
            Err(e) => {
                if intento == max_intentos {
                    // Se acabaron los intentos: devolvemos el ÚLTIMO error,
                    // no un error genérico "falló todo" — el caller necesita
                    // saber la causa real para decidir qué hacer.
                    return Err(e);
                }
                sleep(Duration::from_millis(espera)).await;
                espera *= 2; // backoff exponencial: 100, 200, 400, 800...
            }
        }
    }
    unreachable!("max_intentos >= 1 garantiza que el for entra al menos una vez")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::pin::Pin;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    /// Simula una operación que falla las primeras `fallos_antes_de_exito`
    /// veces y después tiene éxito. `AtomicU32` porque el contador se
    /// comparte entre llamadas sucesivas del closure (necesita mutación
    /// interior, y acá SÍ puede haber concurrencia real del runtime).
    ///
    /// Nota de tipos: `con_reintentos` acepta cualquier `Fut: Future`, pero
    /// acá en el test necesitamos NOMBRAR el tipo de retorno del closure
    /// para la firma del helper. Un `async move {}` anónimo no tiene un
    /// nombre pronunciable (cada uno es un tipo único generado por el
    /// compilador), así que lo metemos detrás de un
    /// `Pin<Box<dyn Future<...>>>` — el "type erasure" de toda la vida,
    /// como devolver una `std::function<std::future<T>()>` en C++.
    type FuturoReintento = Pin<Box<dyn Future<Output = Result<&'static str, String>>>>;

    fn operacion_inestable(
        fallos_antes_de_exito: u32,
    ) -> (impl FnMut() -> FuturoReintento, Arc<AtomicU32>) {
        let intentos = Arc::new(AtomicU32::new(0));
        let contador = Arc::clone(&intentos);

        let closure = move || -> FuturoReintento {
            let contador = Arc::clone(&contador);
            Box::pin(async move {
                let n = contador.fetch_add(1, Ordering::SeqCst);
                if n < fallos_antes_de_exito {
                    Err(format!("fallo simulado #{}", n + 1))
                } else {
                    Ok("éxito")
                }
            })
        };
        (closure, intentos)
    }

    #[tokio::test]
    async fn exito_al_primer_intento_no_espera_de_mas() {
        let (op, intentos) = operacion_inestable(0);
        let resultado = con_reintentos(op, 5, 1).await;
        assert_eq!(resultado, Ok("éxito"));
        assert_eq!(intentos.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn exito_tras_dos_fallos() {
        let (op, intentos) = operacion_inestable(2);
        let resultado = con_reintentos(op, 5, 1).await;
        assert_eq!(resultado, Ok("éxito"));
        assert_eq!(intentos.load(Ordering::SeqCst), 3); // 2 fallos + 1 éxito
    }

    #[tokio::test]
    async fn agota_intentos_y_devuelve_el_ultimo_error() {
        let (op, intentos) = operacion_inestable(100); // nunca tiene éxito
        let resultado = con_reintentos(op, 3, 1).await;
        assert_eq!(resultado, Err("fallo simulado #3".to_string()));
        assert_eq!(intentos.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn el_backoff_es_exponencial() {
        use tokio::time::Instant;
        let (op, _) = operacion_inestable(3);
        let inicio = Instant::now();
        con_reintentos(op, 5, 20).await.unwrap();
        // esperas: 20 + 40 + 80 = 140ms mínimo entre los 3 fallos
        assert!(inicio.elapsed() >= Duration::from_millis(140));
    }
}
