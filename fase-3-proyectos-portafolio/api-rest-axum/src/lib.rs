//! # api-rest-axum — API REST de tareas con Axum + SQLx
//!
//! La lib expone `construir_app(pool)`, que arma el `Router` completo.
//! Separarlo de `main.rs` permite que los tests de integración (en
//! `tests/`) construyan la MISMA app que corre en producción, pero
//! apuntando a una base de datos SQLite en memoria — sin levantar un
//! servidor HTTP real ni pegarle por la red (ver `tests/integracion.rs`).

pub mod db;
pub mod error;
pub mod modelos;
pub mod rutas;

use axum::routing::{get, post};
use axum::Router;
use sqlx::SqlitePool;

/// Arma el router de la API, con el pool de conexiones como estado
/// compartido (`with_state`). Axum inyecta ese estado en cada handler que
/// lo pida vía el extractor `State<SqlitePool>` — sin variables globales,
/// sin singletons: el estado viaja explícito en el tipo del Router.
pub fn construir_app(pool: SqlitePool) -> Router {
    Router::new()
        .route("/salud", get(rutas::salud))
        .route(
            "/tareas",
            post(rutas::crear_tarea).get(rutas::listar_tareas),
        )
        .route(
            "/tareas/{id}",
            get(rutas::obtener_tarea)
                .put(rutas::actualizar_tarea)
                .delete(rutas::eliminar_tarea),
        )
        .with_state(pool)
}
