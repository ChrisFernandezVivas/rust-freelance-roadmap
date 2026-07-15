//! Modelos de datos: la representación de una `Tarea` en la API (Serde,
//! para JSON) y en la base (sqlx::FromRow, para mapear filas de SQLite).
//!
//! Nota de diseño: usamos TRES structs distintos para "una tarea" según el
//! momento del ciclo de vida:
//! - `Tarea`: lo que devolvemos (tiene `id` y `creada_en`, generados por la DB).
//! - `NuevaTarea`: lo que el cliente ENVÍA para crear (solo `titulo`; el resto
//!   lo decide el servidor — el cliente no puede inventarse un `id`).
//! - `ActualizarTarea`: lo que el cliente envía para actualizar, con TODOS
//!   los campos opcionales (`Option<T>`) — un PATCH parcial, no un PUT que
//!   exige mandar el recurso completo.
//!
//! Es el mismo principio que en cualquier API bien diseñada en cualquier
//! lenguaje: el modelo de "entrada" y el de "salida" NO son el mismo tipo,
//! aunque se parezcan. Acá el compilador te obliga a ser explícito sobre
//! esa diferencia en vez de reusar una sola clase con campos opcionales.

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Una tarea tal como vive en la base y se devuelve al cliente.
#[derive(Debug, Serialize, Deserialize, FromRow, PartialEq, Clone)]
pub struct Tarea {
    pub id: i64,
    pub titulo: String,
    pub completada: bool,
    pub creada_en: String,
}

/// Payload para crear una tarea nueva: el cliente solo decide el título.
#[derive(Debug, Deserialize)]
pub struct NuevaTarea {
    pub titulo: String,
}

/// Payload para actualizar una tarea existente. Ambos campos son
/// opcionales: el cliente manda solo lo que quiere cambiar.
#[derive(Debug, Deserialize, Default)]
pub struct ActualizarTarea {
    pub titulo: Option<String>,
    pub completada: Option<bool>,
}
