//! Configuración de la conexión a la base de datos.
//!
//! ## Por qué SQLite y no Postgres en este repo
//!
//! En producción real, este servicio usaría **Postgres** (transacciones
//! multi-usuario más robustas, tipos más ricos, réplicas...). Acá usamos
//! **SQLite vía sqlx** para que el proyecto **compile y corra sin
//! infraestructura externa**: sin Docker Compose, sin levantar un server de
//! Postgres, sin credenciales. Es una decisión deliberada para un repo de
//! portafolio/estudio: `cargo test` y `cargo run` funcionan en cualquier
//! máquina con Rust instalado, punto.
//!
//! Migrar a Postgres en un proyecto real sería, en gran parte, cambiar el
//! feature de sqlx (`sqlite` → `postgres`) y el tipo de pool — la mayoría
//! del SQL (sintaxis estándar) y TODO el resto del código (handlers,
//! modelos, error handling) queda igual. Esa portabilidad es una de las
//! razones por las que sqlx es una buena elección para un backend real.

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::str::FromStr;

/// Crea el pool de conexiones y corre las migraciones pendientes.
///
/// `url` puede ser:
/// - `"sqlite::memory:"` para tests (base efímera, vive solo en RAM).
/// - `"sqlite:tareas.db"` para uso real (archivo persistente en disco).
///
/// `create_if_missing(true)`: para que el archivo se cree solo la primera
/// vez, sin necesitar un paso manual de "inicializar la base" (algo que en
/// Postgres/MySQL normalmente sí requiere un comando aparte).
pub async fn crear_pool(url: &str) -> Result<SqlitePool, sqlx::Error> {
    let opciones = SqliteConnectOptions::from_str(url)?.create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        // Un solo writer a la vez es una limitación conocida de SQLite;
        // para este proyecto (portafolio, no alta concurrencia real) alcanza
        // de sobra. max_connections bajo evita contención innecesaria.
        .max_connections(5)
        .connect_with(opciones)
        .await?;

    // sqlx::migrate! embebe el contenido de migrations/ EN EL BINARIO en
    // tiempo de compilación (no necesita leer archivos en runtime, y por
    // eso no requiere una DB conectada al COMPILAR — a diferencia de las
    // macros sqlx::query! que sí verifican contra una DB real).
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
