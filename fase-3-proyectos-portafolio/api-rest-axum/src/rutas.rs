//! Handlers HTTP: el CRUD completo del recurso `tareas`.
//!
//! Fijate el patrón que se repite en cada handler: extractors de Axum
//! (`State`, `Path`, `Json`) declaran QUÉ necesita el handler, y Axum se
//! encarga de parsearlo desde el request antes de llamar a la función. Es
//! inyección de dependencias vía TIPOS, sin contenedor de DI ni reflection
//! (compará con Spring/FastAPI, donde esto se resuelve con anotaciones e
//! introspección en runtime).

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use sqlx::SqlitePool;

use crate::error::ErrorApi;
use crate::modelos::{ActualizarTarea, NuevaTarea, Tarea};

/// GET /salud — chequeo de vida, para balanceadores de carga / Docker HEALTHCHECK.
pub async fn salud() -> impl IntoResponse {
    StatusCode::OK
}

/// POST /tareas — crea una tarea nueva.
pub async fn crear_tarea(
    State(pool): State<SqlitePool>,
    Json(nueva): Json<NuevaTarea>,
) -> Result<(StatusCode, Json<Tarea>), ErrorApi> {
    let titulo = nueva.titulo.trim();
    if titulo.is_empty() {
        // Validación de negocio ANTES de tocar la base: rechazamos rápido
        // y con un mensaje claro, en vez de dejar que una constraint SQL
        // falle más abajo con un error críptico.
        return Err(ErrorApi::SolicitudInvalida(
            "el título no puede estar vacío".into(),
        ));
    }

    // query_as! (con macro) validaría esto contra la DB en COMPILE TIME,
    // pero requeriría una base disponible durante `cargo build` (o un
    // archivo .sqlx cacheado). Usamos la variante EN RUNTIME (`query_as`,
    // sin `!`) a propósito: el proyecto compila en cualquier máquina sin
    // pasos extra — el trade-off correcto para un repo de portafolio/CI.
    let tarea = sqlx::query_as::<_, Tarea>(
        "INSERT INTO tareas (titulo) VALUES (?) RETURNING id, titulo, completada, creada_en",
    )
    .bind(titulo)
    .fetch_one(&pool)
    .await?; // el `?` usa el From<sqlx::Error> de ErrorApi (fase 1, ejercicio 3)

    Ok((StatusCode::CREATED, Json(tarea)))
}

/// GET /tareas — lista todas las tareas, más nuevas primero.
pub async fn listar_tareas(State(pool): State<SqlitePool>) -> Result<Json<Vec<Tarea>>, ErrorApi> {
    let tareas = sqlx::query_as::<_, Tarea>(
        "SELECT id, titulo, completada, creada_en FROM tareas ORDER BY id DESC",
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(tareas))
}

/// GET /tareas/{id} — obtiene una tarea puntual.
pub async fn obtener_tarea(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<Tarea>, ErrorApi> {
    let tarea = sqlx::query_as::<_, Tarea>(
        "SELECT id, titulo, completada, creada_en FROM tareas WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await?
    // fetch_optional devuelve Option<Tarea>; ok_or convierte el None
    // en NUESTRO error de dominio (404), no en un error de sqlx.
    .ok_or(ErrorApi::NoEncontrada(id))?;

    Ok(Json(tarea))
}

/// PUT /tareas/{id} — actualiza parcialmente una tarea (título y/o estado).
pub async fn actualizar_tarea(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(cambios): Json<ActualizarTarea>,
) -> Result<Json<Tarea>, ErrorApi> {
    // Primero confirmamos que exista (para devolver 404 preciso en vez de
    // "0 filas afectadas" silencioso, que dejaría al cliente adivinando).
    let actual = obtener_tarea_o_404(&pool, id).await?;

    let nuevo_titulo = cambios.titulo.unwrap_or(actual.titulo);
    if nuevo_titulo.trim().is_empty() {
        return Err(ErrorApi::SolicitudInvalida(
            "el título no puede estar vacío".into(),
        ));
    }
    let nueva_completada = cambios.completada.unwrap_or(actual.completada);

    let tarea = sqlx::query_as::<_, Tarea>(
        "UPDATE tareas SET titulo = ?, completada = ? WHERE id = ? \
         RETURNING id, titulo, completada, creada_en",
    )
    .bind(nuevo_titulo)
    .bind(nueva_completada)
    .bind(id)
    .fetch_one(&pool)
    .await?;

    Ok(Json(tarea))
}

/// DELETE /tareas/{id} — elimina una tarea.
pub async fn eliminar_tarea(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ErrorApi> {
    let resultado = sqlx::query("DELETE FROM tareas WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?;

    if resultado.rows_affected() == 0 {
        return Err(ErrorApi::NoEncontrada(id));
    }

    // 204 No Content: éxito, sin cuerpo — la convención REST para DELETE.
    Ok(StatusCode::NO_CONTENT)
}

/// Helper interno compartido por `actualizar_tarea`: trae la tarea o 404.
async fn obtener_tarea_o_404(pool: &SqlitePool, id: i64) -> Result<Tarea, ErrorApi> {
    sqlx::query_as::<_, Tarea>("SELECT id, titulo, completada, creada_en FROM tareas WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(ErrorApi::NoEncontrada(id))
}
