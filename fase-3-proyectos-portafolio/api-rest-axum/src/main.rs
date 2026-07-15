//! Binario del servicio: lee configuración, crea el pool, arma la app
//! (vía la lib) y sirve HTTP con Tokio.

use api_rest_axum::{construir_app, db};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // tracing_subscriber: logging estructurado (nivel, spans). Reemplaza
    // los `println!` de debug por algo que en producción se puede filtrar
    // por nivel (info/warn/error) y correlacionar entre requests.
    tracing_subscriber::fmt::init();

    // DATABASE_URL configurable por entorno (12-factor app); si no está
    // seteada, usamos un archivo local — cero configuración para probar.
    let url_db =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:tareas.db?mode=rwc".to_string());

    let pool = db::crear_pool(&url_db).await?;
    tracing::info!(url = %url_db, "conectado a la base de datos, migraciones aplicadas");

    let app = construir_app(pool);

    let direccion = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(direccion).await?;
    tracing::info!(direccion, "servidor escuchando");

    axum::serve(listener, app).await?;

    Ok(())
}
