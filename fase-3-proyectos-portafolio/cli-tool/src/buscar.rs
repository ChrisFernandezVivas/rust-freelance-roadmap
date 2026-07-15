//! Lógica del subcomando `buscar`: encontrar un patrón de texto en archivos,
//! recursivamente, y reportar archivo:línea:contenido — como un `grep -rn`
//! minimalista.

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Una coincidencia encontrada: dónde y qué.
#[derive(Debug, PartialEq)]
pub struct Coincidencia {
    pub archivo: String,
    pub linea: usize,
    pub contenido: String,
}

/// Busca `patron` en todos los archivos de texto bajo `raiz`, recursivamente.
///
/// Devuelve `Result<Vec<_>>`: la función SÍ puede fallar (ruta inexistente,
/// sin permisos), y a diferencia de un `unwrap()` en cada paso, dejamos que
/// el caller (en `main.rs`) decida cómo mostrar el error con contexto.
pub fn buscar_en_arbol(
    raiz: &Path,
    patron: &str,
    sensible_a_mayus: bool,
) -> Result<Vec<Coincidencia>> {
    if !raiz.exists() {
        // anyhow::bail! construye un error ad-hoc con contexto — el
        // equivalente de `throw std::runtime_error(...)` de C++, pero sin
        // pagar el costo de excepciones y con el error viajando en el tipo.
        anyhow::bail!("la ruta '{}' no existe", raiz.display());
    }

    let patron_comparar = if sensible_a_mayus {
        patron.to_string()
    } else {
        patron.to_lowercase()
    };

    let mut resultados = Vec::new();

    // WalkDir::new(raiz) itera TODO el árbol; filter_map descarta entradas
    // con error de permisos en vez de abortar la búsqueda completa (una
    // carpeta sin acceso no debería tumbar todo el comando).
    for entrada in WalkDir::new(raiz).into_iter().filter_map(|e| e.ok()) {
        if !entrada.file_type().is_file() {
            continue;
        }

        // Saltamos archivos que claramente no son texto (heurística simple
        // por extensión, evita gastar tiempo intentando leer binarios).
        if es_binario_probable(entrada.path()) {
            continue;
        }

        // Leemos el archivo entero. Si no es UTF-8 válido (probablemente
        // binario disfrazado), lo saltamos en vez de fallar todo el comando
        // — igual filosofía que WalkDir arriba: un archivo problemático no
        // debe abortar la búsqueda completa.
        let contenido = match fs::read_to_string(entrada.path()) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for (i, linea) in contenido.lines().enumerate() {
            let linea_comparar = if sensible_a_mayus {
                linea.to_string()
            } else {
                linea.to_lowercase()
            };

            if linea_comparar.contains(&patron_comparar) {
                resultados.push(Coincidencia {
                    archivo: entrada.path().display().to_string(),
                    linea: i + 1, // 1-indexado: así lo espera un humano/editor
                    contenido: linea.to_string(),
                });
            }
        }
    }

    Ok(resultados)
}

/// Heurística barata: extensiones comunes de binarios/artefactos que no
/// tiene sentido buscar como texto.
fn es_binario_probable(ruta: &Path) -> bool {
    matches!(
        ruta.extension().and_then(|e| e.to_str()),
        Some(
            "png"
                | "jpg"
                | "jpeg"
                | "gif"
                | "ico"
                | "pdf"
                | "zip"
                | "exe"
                | "so"
                | "dylib"
                | "o"
                | "rlib"
        )
    )
}

/// Cuenta cuántos archivos DISTINTOS tienen al menos una coincidencia —
/// útil para el resumen final (`buscar` reporta "N coincidencias en M archivos").
pub fn contar_archivos_distintos(coincidencias: &[Coincidencia]) -> usize {
    use std::collections::HashSet;
    coincidencias
        .iter()
        .map(|c| c.archivo.as_str())
        .collect::<HashSet<_>>()
        .len()
}

/// Helper usado por `main.rs`: valida que la ruta exista y sea legible
/// antes de arrancar, para dar un mensaje de error claro con `Context`.
pub fn validar_raiz(raiz: &Path) -> Result<()> {
    fs::metadata(raiz).with_context(|| format!("no se pudo acceder a '{}'", raiz.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn encuentra_coincidencias_en_un_archivo() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("a.txt"),
            "hola mundo\nRust es genial\notra linea",
        )
        .unwrap();

        let resultados = buscar_en_arbol(dir.path(), "rust", false).unwrap();
        assert_eq!(resultados.len(), 1);
        assert_eq!(resultados[0].linea, 2);
        assert_eq!(resultados[0].contenido, "Rust es genial");
    }

    #[test]
    fn respeta_sensibilidad_a_mayusculas() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), "Rust\nrust\nRUST").unwrap();

        assert_eq!(buscar_en_arbol(dir.path(), "rust", false).unwrap().len(), 3);
        assert_eq!(buscar_en_arbol(dir.path(), "rust", true).unwrap().len(), 1);
    }

    #[test]
    fn busca_recursivamente_en_subcarpetas() {
        let dir = tempdir().unwrap();
        fs::create_dir(dir.path().join("sub")).unwrap();
        fs::write(dir.path().join("raiz.txt"), "manzana").unwrap();
        fs::write(dir.path().join("sub/hoja.txt"), "manzana verde").unwrap();

        let resultados = buscar_en_arbol(dir.path(), "manzana", false).unwrap();
        assert_eq!(resultados.len(), 2);
    }

    #[test]
    fn ruta_inexistente_devuelve_error() {
        let resultado = buscar_en_arbol(Path::new("/ruta/que/no/existe/jamas"), "x", false);
        assert!(resultado.is_err());
    }

    #[test]
    fn sin_coincidencias_devuelve_vacio() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), "nada que ver").unwrap();
        assert!(buscar_en_arbol(dir.path(), "inexistente", false)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn cuenta_archivos_distintos_correctamente() {
        let coincidencias = vec![
            Coincidencia {
                archivo: "a.txt".into(),
                linea: 1,
                contenido: "x".into(),
            },
            Coincidencia {
                archivo: "a.txt".into(),
                linea: 2,
                contenido: "x".into(),
            },
            Coincidencia {
                archivo: "b.txt".into(),
                linea: 1,
                contenido: "x".into(),
            },
        ];
        assert_eq!(contar_archivos_distintos(&coincidencias), 2);
    }
}
