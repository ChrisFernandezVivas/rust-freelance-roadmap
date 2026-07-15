# textkit — buscador y reemplazador de texto en archivos

Un CLI real, con dos subcomandos útiles de verdad (no un "hola mundo"):

- **`buscar`**: encuentra un patrón de texto recursivamente bajo una ruta,
  como un `grep -rn` minimalista (con opción de sensibilidad a mayúsculas).
- **`reemplazar`**: sustituye un patrón por otro texto en todos los archivos
  bajo una ruta, con `--dry-run` para ver el impacto ANTES de tocar nada.

## Cómo correrlo

```bash
cargo build --release

# Buscar "TODO" recursivamente (no distingue mayúsculas por defecto)
cargo run -- buscar TODO ./src

# Buscar distinguiendo mayúsculas
cargo run -- buscar "TODO" ./src --sensible-a-mayus

# Ver qué cambiaría, sin tocar nada
cargo run -- reemplazar "version_vieja" "version_nueva" ./config --dry-run

# Aplicar el reemplazo de verdad
cargo run -- reemplazar "version_vieja" "version_nueva" ./config
```

## Tests

```bash
cargo test
```

- **Tests unitarios** (`src/buscar.rs`, `src/reemplazar.rs`): prueban la
  lógica pura con archivos temporales (`tempfile`), sin pasar por el CLI.
- **Tests de integración** (`tests/integracion.rs`): ejecutan el **binario
  compilado de verdad** con `assert_cmd`, verificando stdout, stderr y
  exit codes — el contrato completo que ve un usuario real.

19 tests en total, todos en verde.

## Qué conceptos de Rust demuestra

- **`clap` con `#[derive(Parser)]`**: parseo de argumentos declarativo,
  con subcomandos (`buscar`/`reemplazar`) verificados en tiempo de
  compilación — un typo en un flag no compila, a diferencia de `argparse`
  de Python donde el error aparece en runtime.
- **`anyhow`**: manejo de errores de aplicación con contexto (`.context()`,
  `anyhow::bail!`) y un solo tipo de error en `main`, sin necesidad de un
  enum de errores custom (eso se justifica en una *librería*; en un
  *binario* casi nunca hace falta que el caller distinga por tipo).
- **Separación lib/bin**: la lógica vive en `src/lib.rs` + módulos; `main.rs`
  solo parsea argumentos y llama a la lib. Permite testear la lógica con
  tests unitarios rápidos Y el binario completo con tests de integración.
- **`std::process::ExitCode`**: salir con código 0/1 de forma idiomática,
  sin `std::process::exit()` a mano (que salta destructores).
- **Manejo de errores por archivo, no por operación completa**: un archivo
  binario o sin permisos se salta con `continue`, en vez de abortar toda la
  búsqueda/reemplazo — el mismo principio de resiliencia que en
  `fase-2-intermedio/concurrencia`.
- **`--dry-run` como patrón de diseño**: cualquier operación destructiva de
  un CLI profesional debería tener una forma de "simular" antes de aplicar.

## Estructura

```
src/
  main.rs         # clap: parseo de argumentos + subcomandos
  lib.rs          # declara los módulos de lógica
  buscar.rs       # lógica de búsqueda + tests unitarios
  reemplazar.rs   # lógica de reemplazo + tests unitarios
tests/
  integracion.rs  # tests end-to-end contra el binario compilado
```

Parte de la [fase 3 del roadmap](../README.md).
