//! Tests de integración: ejecutan el BINARIO COMPILADO de verdad (no
//! llaman a funciones internas), igual que lo haría un usuario desde la
//! terminal. `assert_cmd::Command::cargo_bin` encuentra el binario que
//! `cargo test` acaba de compilar; no hace falta instalar nada aparte.
//!
//! Esta es la diferencia clave con los tests unitarios de `src/`: ahí
//! probamos la LÓGICA; acá probamos el CONTRATO completo del CLI (parseo
//! de argumentos, exit codes, stdout/stderr) — lo que de verdad ve un
//! usuario o un script de CI que invoque `textkit`.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn buscar_encuentra_coincidencia_y_reporta_resumen() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("nota.txt"),
        "primera linea\nRust es genial\nultima",
    )
    .unwrap();

    Command::cargo_bin("textkit")
        .unwrap()
        .args(["buscar", "rust", dir.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Rust es genial"))
        .stderr(predicate::str::contains(
            "1 coincidencia(s) en 1 archivo(s)",
        ));
}

#[test]
fn buscar_sensible_a_mayusculas_no_encuentra_nada() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("nota.txt"), "rust en minusculas").unwrap();

    Command::cargo_bin("textkit")
        .unwrap()
        .args([
            "buscar",
            "Rust",
            dir.path().to_str().unwrap(),
            "--sensible-a-mayus",
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("0 coincidencia(s)"));
}

#[test]
fn buscar_en_ruta_inexistente_falla_con_exit_code_1() {
    Command::cargo_bin("textkit")
        .unwrap()
        .args(["buscar", "x", "/ruta/inventada/que/no/existe"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("error:"));
}

#[test]
fn reemplazar_modifica_el_archivo_en_disco() {
    let dir = tempdir().unwrap();
    let archivo = dir.path().join("config.txt");
    fs::write(&archivo, "entorno=desarrollo").unwrap();

    Command::cargo_bin("textkit")
        .unwrap()
        .args([
            "reemplazar",
            "desarrollo",
            "produccion",
            dir.path().to_str().unwrap(),
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("se reemplazaron 1 ocurrencia"));

    assert_eq!(fs::read_to_string(&archivo).unwrap(), "entorno=produccion");
}

#[test]
fn reemplazar_con_dry_run_no_toca_el_archivo() {
    let dir = tempdir().unwrap();
    let archivo = dir.path().join("config.txt");
    fs::write(&archivo, "entorno=desarrollo").unwrap();

    Command::cargo_bin("textkit")
        .unwrap()
        .args([
            "reemplazar",
            "desarrollo",
            "produccion",
            dir.path().to_str().unwrap(),
            "--dry-run",
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("(dry-run)"));

    // El contenido NO cambió:
    assert_eq!(fs::read_to_string(&archivo).unwrap(), "entorno=desarrollo");
}

#[test]
fn reemplazar_con_patron_vacio_falla() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("a.txt"), "algo").unwrap();

    Command::cargo_bin("textkit")
        .unwrap()
        .args(["reemplazar", "", "x", dir.path().to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("no puede ser vacío"));
}

#[test]
fn sin_subcomando_muestra_ayuda_y_falla() {
    Command::cargo_bin("textkit").unwrap().assert().failure();
}

#[test]
fn flag_de_version_funciona() {
    Command::cargo_bin("textkit")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("textkit"));
}
