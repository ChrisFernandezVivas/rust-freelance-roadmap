//! Lógica del subcomando `reemplazar`: sustituir un patrón por otro texto
//! en todos los archivos bajo una ruta, con soporte de `--dry-run`
//! (mostrar qué cambiaría, sin tocar nada — la opción que salva vidas).

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Un cambio que se hizo (o que se HARÍA, en modo dry-run).
#[derive(Debug, PartialEq)]
pub struct Cambio {
    pub archivo: String,
    pub ocurrencias: usize,
}

/// Reemplaza todas las ocurrencias de `patron` por `reemplazo` en los
/// archivos bajo `raiz`. Si `dry_run` es true, NO escribe nada: solo
/// calcula qué archivos cambiarían y cuántas ocurrencias tiene cada uno.
///
/// Este flag es el patrón profesional para cualquier operación destructiva
/// de un CLI: dejar que el usuario vea el "blast radius" antes de comprometerse.
pub fn reemplazar_en_arbol(
    raiz: &Path,
    patron: &str,
    reemplazo: &str,
    dry_run: bool,
) -> Result<Vec<Cambio>> {
    if patron.is_empty() {
        // Reemplazar "" por algo insertaría el reemplazo en CADA posición
        // de cada archivo — un desastre silencioso. Lo rechazamos temprano
        // con un mensaje claro, en vez de dejar que el usuario lo descubra
        // mirando un archivo destruido.
        anyhow::bail!("el patrón de búsqueda no puede ser vacío");
    }

    let mut cambios = Vec::new();

    for entrada in WalkDir::new(raiz).into_iter().filter_map(|e| e.ok()) {
        if !entrada.file_type().is_file() {
            continue;
        }

        let contenido = match fs::read_to_string(entrada.path()) {
            Ok(c) => c,
            Err(_) => continue, // probablemente binario: lo saltamos
        };

        let ocurrencias = contenido.matches(patron).count();
        if ocurrencias == 0 {
            continue;
        }

        if !dry_run {
            let nuevo_contenido = contenido.replace(patron, reemplazo);
            fs::write(entrada.path(), nuevo_contenido)
                .with_context(|| format!("no se pudo escribir '{}'", entrada.path().display()))?;
        }

        cambios.push(Cambio {
            archivo: entrada.path().display().to_string(),
            ocurrencias,
        });
    }

    Ok(cambios)
}

/// Total de ocurrencias reemplazadas (o que se reemplazarían) en todo el árbol.
pub fn total_ocurrencias(cambios: &[Cambio]) -> usize {
    cambios.iter().map(|c| c.ocurrencias).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn reemplaza_de_verdad_por_defecto() {
        let dir = tempdir().unwrap();
        let archivo = dir.path().join("a.txt");
        fs::write(&archivo, "hola mundo, mundo cruel").unwrap();

        let cambios = reemplazar_en_arbol(dir.path(), "mundo", "planeta", false).unwrap();

        assert_eq!(
            cambios,
            vec![Cambio {
                archivo: archivo.display().to_string(),
                ocurrencias: 2
            }]
        );
        assert_eq!(
            fs::read_to_string(&archivo).unwrap(),
            "hola planeta, planeta cruel"
        );
    }

    #[test]
    fn dry_run_no_modifica_nada() {
        let dir = tempdir().unwrap();
        let archivo = dir.path().join("a.txt");
        let original = "el gato duerme";
        fs::write(&archivo, original).unwrap();

        let cambios = reemplazar_en_arbol(dir.path(), "gato", "perro", true).unwrap();

        assert_eq!(cambios.len(), 1);
        assert_eq!(cambios[0].ocurrencias, 1);
        // El archivo NO debe haber cambiado en disco:
        assert_eq!(fs::read_to_string(&archivo).unwrap(), original);
    }

    #[test]
    fn patron_vacio_es_error() {
        let dir = tempdir().unwrap();
        let resultado = reemplazar_en_arbol(dir.path(), "", "x", false);
        assert!(resultado.is_err());
    }

    #[test]
    fn archivos_sin_coincidencias_no_aparecen_en_el_reporte() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), "nada relevante").unwrap();
        fs::write(dir.path().join("b.txt"), "esto tiene manzana").unwrap();

        let cambios = reemplazar_en_arbol(dir.path(), "manzana", "pera", true).unwrap();
        assert_eq!(cambios.len(), 1);
        assert!(cambios[0].archivo.ends_with("b.txt"));
    }

    #[test]
    fn total_ocurrencias_suma_todos_los_archivos() {
        let cambios = vec![
            Cambio {
                archivo: "a".into(),
                ocurrencias: 3,
            },
            Cambio {
                archivo: "b".into(),
                ocurrencias: 5,
            },
        ];
        assert_eq!(total_ocurrencias(&cambios), 8);
    }
}
