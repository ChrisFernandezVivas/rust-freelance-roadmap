//! Tests de integración: ejercitan la API completa (router de Axum +
//! handlers + SQLx contra una SQLite REAL en memoria) sin levantar un
//! socket TCP. `tower::ServiceExt::oneshot` envía UN request al `Router`
//! directamente, como lo haría el servidor HTTP real, pero en memoria: más
//! rápido y más determinístico en CI que abrir un puerto y usar un cliente
//! HTTP de verdad.
//!
//! Cada test crea SU PROPIA base en memoria (`sqlite::memory:`), así que
//! son independientes entre sí (no comparten estado, se pueden correr en
//! paralelo sin pisarse — `cargo test` los paraleliza por defecto).

use api_rest_axum::{construir_app, db, modelos::Tarea};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt; // trae el método `.oneshot()`

/// Arma una app fresca con una base SQLite en memoria y las migraciones
/// ya aplicadas — el equivalente de "levantar un servidor de prueba" en
/// otros frameworks, pero sin proceso ni puerto real.
async fn app_de_prueba() -> axum::Router {
    let pool = db::crear_pool("sqlite::memory:")
        .await
        .expect("no se pudo crear el pool de prueba");
    construir_app(pool)
}

/// Helper: arma un request JSON con método y body.
fn request_json(metodo: &str, uri: &str, cuerpo: Option<Value>) -> Request<Body> {
    let body = match cuerpo {
        Some(v) => Body::from(v.to_string()),
        None => Body::empty(),
    };
    Request::builder()
        .method(metodo)
        .uri(uri)
        .header("content-type", "application/json")
        .body(body)
        .unwrap()
}

/// Helper: extrae y parsea el body de una respuesta como JSON.
async fn body_json(response: axum::response::Response) -> Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn salud_responde_200() {
    let app = app_de_prueba().await;
    let respuesta = app
        .oneshot(request_json("GET", "/salud", None))
        .await
        .unwrap();
    assert_eq!(respuesta.status(), StatusCode::OK);
}

#[tokio::test]
async fn crear_y_listar_tarea() {
    let app = app_de_prueba().await;

    let respuesta = app
        .clone()
        .oneshot(request_json(
            "POST",
            "/tareas",
            Some(json!({ "titulo": "comprar café" })),
        ))
        .await
        .unwrap();
    assert_eq!(respuesta.status(), StatusCode::CREATED);

    let creada: Tarea = serde_json::from_value(body_json(respuesta).await).unwrap();
    assert_eq!(creada.titulo, "comprar café");
    assert!(!creada.completada); // por defecto, no completada
    assert!(creada.id > 0);

    let respuesta = app
        .oneshot(request_json("GET", "/tareas", None))
        .await
        .unwrap();
    assert_eq!(respuesta.status(), StatusCode::OK);
    let lista: Vec<Tarea> = serde_json::from_value(body_json(respuesta).await).unwrap();
    assert_eq!(lista.len(), 1);
    assert_eq!(lista[0].titulo, "comprar café");
}

#[tokio::test]
async fn crear_con_titulo_vacio_es_400() {
    let app = app_de_prueba().await;
    let respuesta = app
        .oneshot(request_json(
            "POST",
            "/tareas",
            Some(json!({ "titulo": "   " })),
        ))
        .await
        .unwrap();
    assert_eq!(respuesta.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn obtener_tarea_inexistente_es_404() {
    let app = app_de_prueba().await;
    let respuesta = app
        .oneshot(request_json("GET", "/tareas/999", None))
        .await
        .unwrap();
    assert_eq!(respuesta.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn actualizar_tarea_cambia_solo_lo_enviado() {
    let app = app_de_prueba().await;

    let respuesta = app
        .clone()
        .oneshot(request_json(
            "POST",
            "/tareas",
            Some(json!({ "titulo": "lavar el auto" })),
        ))
        .await
        .unwrap();
    let creada: Tarea = serde_json::from_value(body_json(respuesta).await).unwrap();

    // Solo mandamos `completada`: el título NO debería cambiar (PATCH parcial).
    let respuesta = app
        .clone()
        .oneshot(request_json(
            "PUT",
            &format!("/tareas/{}", creada.id),
            Some(json!({ "completada": true })),
        ))
        .await
        .unwrap();
    assert_eq!(respuesta.status(), StatusCode::OK);

    let actualizada: Tarea = serde_json::from_value(body_json(respuesta).await).unwrap();
    assert_eq!(actualizada.titulo, "lavar el auto"); // sin cambios
    assert!(actualizada.completada); // sí cambió

    // Y ahora cambiamos solo el título:
    let respuesta = app
        .oneshot(request_json(
            "PUT",
            &format!("/tareas/{}", creada.id),
            Some(json!({ "titulo": "lavar el auto (de nuevo)" })),
        ))
        .await
        .unwrap();
    let actualizada: Tarea = serde_json::from_value(body_json(respuesta).await).unwrap();
    assert_eq!(actualizada.titulo, "lavar el auto (de nuevo)");
    assert!(actualizada.completada); // se mantiene de la actualización anterior
}

#[tokio::test]
async fn actualizar_tarea_inexistente_es_404() {
    let app = app_de_prueba().await;
    let respuesta = app
        .oneshot(request_json(
            "PUT",
            "/tareas/12345",
            Some(json!({ "completada": true })),
        ))
        .await
        .unwrap();
    assert_eq!(respuesta.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn eliminar_tarea_y_verificar_que_desaparece() {
    let app = app_de_prueba().await;

    let respuesta = app
        .clone()
        .oneshot(request_json(
            "POST",
            "/tareas",
            Some(json!({ "titulo": "temporal" })),
        ))
        .await
        .unwrap();
    let creada: Tarea = serde_json::from_value(body_json(respuesta).await).unwrap();

    let respuesta = app
        .clone()
        .oneshot(request_json(
            "DELETE",
            &format!("/tareas/{}", creada.id),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(respuesta.status(), StatusCode::NO_CONTENT);

    let respuesta = app
        .oneshot(request_json("GET", &format!("/tareas/{}", creada.id), None))
        .await
        .unwrap();
    assert_eq!(respuesta.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn eliminar_tarea_inexistente_es_404() {
    let app = app_de_prueba().await;
    let respuesta = app
        .oneshot(request_json("DELETE", "/tareas/54321", None))
        .await
        .unwrap();
    assert_eq!(respuesta.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn listar_sin_tareas_devuelve_array_vacio() {
    let app = app_de_prueba().await;
    let respuesta = app
        .oneshot(request_json("GET", "/tareas", None))
        .await
        .unwrap();
    let lista: Vec<Tarea> = serde_json::from_value(body_json(respuesta).await).unwrap();
    assert!(lista.is_empty());
}
