//! `textkit` — CLI real con subcomandos, construido con `clap` (parseo de
//! argumentos declarativo) y `anyhow` (manejo de errores de aplicación).
//!
//! ```text
//! textkit buscar <patron> <ruta> [--sensible-a-mayus]
//! textkit reemplazar <patron> <reemplazo> <ruta> [--dry-run]
//! ```
//!
//! Comparación rápida con lo que ya conocés:
//! - `clap::Parser` con `#[derive]` ≈ `argparse` de Python, pero el árbol
//!   de subcomandos y flags se verifica en COMPILE TIME (un typo en el
//!   nombre de un campo no compila; en argparse fallaría recién en runtime).
//! - `anyhow::Result` en `main()` ≈ dejar que una excepción de Python
//!   llegue al tope y se imprima con traceback — pero controlado: acá
//!   `main` imprime el error con su cadena de contexto y sale con código 1,
//!   sin pánico ni traceback crudo.

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::ExitCode;
use textkit::{buscar, reemplazar};

/// textkit: buscador y reemplazador de texto en archivos, recursivo.
#[derive(Parser)]
#[command(name = "textkit", version, about)]
struct Cli {
    #[command(subcommand)]
    comando: Comando,
}

#[derive(Subcommand)]
enum Comando {
    /// Busca un patrón de texto recursivamente bajo una ruta.
    Buscar {
        /// Texto a buscar.
        patron: String,
        /// Carpeta o archivo donde buscar.
        ruta: PathBuf,
        /// Distinguir mayúsculas de minúsculas (por defecto, no distingue).
        #[arg(short = 's', long)]
        sensible_a_mayus: bool,
    },
    /// Reemplaza un patrón de texto por otro, recursivamente bajo una ruta.
    Reemplazar {
        /// Texto a buscar.
        patron: String,
        /// Texto de reemplazo.
        reemplazo: String,
        /// Carpeta o archivo donde reemplazar.
        ruta: PathBuf,
        /// Mostrar qué cambiaría, sin modificar ningún archivo.
        #[arg(short = 'n', long)]
        dry_run: bool,
    },
}

fn main() -> ExitCode {
    // Separamos main() (que solo decide el ExitCode) de ejecutar() (que
    // hace el trabajo y puede fallar). Así el manejo de errores queda en
    // UN solo lugar, con el patrón `?` normal, en vez de un main gigante
    // lleno de `match` anidados.
    match ejecutar() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            // El formato "{:#}" de anyhow imprime TODA la cadena de
            // contexto (con .context() en cada capa), no solo el error
            // más interno — fundamental para diagnosticar rápido.
            eprintln!("error: {e:#}");
            ExitCode::FAILURE
        }
    }
}

fn ejecutar() -> Result<()> {
    let cli = Cli::parse();

    match cli.comando {
        Comando::Buscar {
            patron,
            ruta,
            sensible_a_mayus,
        } => {
            buscar::validar_raiz(&ruta)?;
            let resultados = buscar::buscar_en_arbol(&ruta, &patron, sensible_a_mayus)?;

            for c in &resultados {
                println!("{}:{}:{}", c.archivo, c.linea, c.contenido);
            }

            let n_archivos = buscar::contar_archivos_distintos(&resultados);
            eprintln!(
                "{} coincidencia(s) en {} archivo(s)",
                resultados.len(),
                n_archivos
            );
        }

        Comando::Reemplazar {
            patron,
            reemplazo,
            ruta,
            dry_run,
        } => {
            let cambios = reemplazar::reemplazar_en_arbol(&ruta, &patron, &reemplazo, dry_run)?;

            for c in &cambios {
                println!("{}: {} ocurrencia(s)", c.archivo, c.ocurrencias);
            }

            let total = reemplazar::total_ocurrencias(&cambios);
            if dry_run {
                eprintln!(
                    "(dry-run) se reemplazarían {total} ocurrencia(s) en {} archivo(s)",
                    cambios.len()
                );
            } else {
                eprintln!(
                    "se reemplazaron {total} ocurrencia(s) en {} archivo(s)",
                    cambios.len()
                );
            }
        }
    }

    Ok(())
}
