-- Tabla de tareas: el recurso CRUD de este ejemplo.
-- SQLite no tiene un tipo BOOLEAN nativo: se guarda como INTEGER (0/1).
-- sqlx decodifica esa columna directamente a `bool` en Rust sin esfuerzo.
CREATE TABLE IF NOT EXISTS tareas (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    titulo TEXT NOT NULL,
    completada INTEGER NOT NULL DEFAULT 0,
    creada_en TEXT NOT NULL DEFAULT (datetime('now'))
);
