//! Error unificado de la API + su conversión a respuesta HTTP.
//!
//! En una librería usaríamos un enum de error "puro" (como en
//! `fase-1-base/errores_result.rs`). Acá agregamos UNA responsabilidad más,
//! propia de una API web: cada variante sabe traducirse a un
//! `(StatusCode, JSON)` — el equivalente de un exception handler global de
//! Express/FastAPI, pero resuelto con el sistema de tipos: `IntoResponse`
//! es un trait que Axum reconoce automáticamente en cualquier valor que
//! devuelva un handler.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum ErrorApi {
    /// El recurso pedido no existe. 404, sin exponer detalles internos.
    #[error("tarea {0} no encontrada")]
    NoEncontrada(i64),

    /// El payload es inválido (por ejemplo, título vacío). 400.
    #[error("solicitud inválida: {0}")]
    SolicitudInvalida(String),

    /// Cualquier error de la base de datos. Lo envolvemos con `#[from]`
    /// para que `?` lo convierta automáticamente (mismo mecanismo que
    /// vimos en `fase-1-base/errores_result.rs` con `From`).
    /// 500: nunca exponemos el mensaje crudo de sqlx al cliente (podría
    /// filtrar detalles del esquema); solo lo logueamos con `tracing`.
    #[error("error de base de datos")]
    BaseDeDatos(#[from] sqlx::Error),
}

impl IntoResponse for ErrorApi {
    fn into_response(self) -> Response {
        let (status, mensaje_publico) = match &self {
            ErrorApi::NoEncontrada(_) => (StatusCode::NOT_FOUND, self.to_string()),
            ErrorApi::SolicitudInvalida(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ErrorApi::BaseDeDatos(e) => {
                // Logueamos el error REAL (con detalle) para debugging,
                // pero el cliente solo ve un mensaje genérico. Filtrar
                // detalles internos de la DB a un cliente es un descuido
                // de seguridad común — acá el tipo nos obliga a decidirlo
                // conscientemente en un solo lugar.
                tracing::error!(error = %e, "fallo de base de datos");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "error interno".to_string(),
                )
            }
        };

        (status, Json(json!({ "error": mensaje_publico }))).into_response()
    }
}
