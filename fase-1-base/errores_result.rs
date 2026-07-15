// ============================================================================
// MANEJO DE ERRORES — Result, el operador `?` y errores custom
// ============================================================================
// Compilar y correr:  rustc errores_result.rs -o /tmp/er && /tmp/er
//
// Contraste con lo que ya conocés:
//
//   C:      códigos de retorno + errno. Nada te obliga a chequearlos:
//           `fopen()` devuelve NULL y si no lo mirás, segfault más adelante.
//   C++:    excepciones. Control de flujo invisible: CUALQUIER llamada puede
//           saltar, la firma no te avisa (noexcept es opt-in y parcial).
//   Python: excepciones también; te enterás en runtime, en producción.
//
//   Rust:   el error es PARTE DEL TIPO DE RETORNO.
//
//     enum Result<T, E> { Ok(T), Err(E) }
//
//   No podés acceder al valor sin decidir qué hacer con el error: el
//   compilador te obliga. Es el "chequeá el código de retorno" de C,
//   pero imposible de olvidar.
// ============================================================================

use std::fmt;
use std::num::ParseFloatError;

fn main() {
    ejemplo_1_result_basico();
    ejemplo_2_operador_interrogacion();
    ejemplo_3_error_custom();
    ejemplo_4_option();
}

// ----------------------------------------------------------------------------
// EJEMPLO 1: Result básico — consumirlo con match
// ----------------------------------------------------------------------------
fn dividir(num: f64, den: f64) -> Result<f64, String> {
    if den == 0.0 {
        // En C devolverías -1 o NaN y escribirías errno... y el caller
        // probablemente ni lo mire. Acá el error viaja tipado.
        Err(String::from("división por cero"))
    } else {
        Ok(num / den)
    }
}

fn ejemplo_1_result_basico() {
    println!("--- Ejemplo 1: Result + match ---");

    // `match` sobre un Result es EXHAUSTIVO: si no manejás las dos variantes,
    // no compila. Imposible "olvidarse del caso de error".
    match dividir(10.0, 3.0) {
        Ok(v) => println!("10/3 = {v:.4}"),
        Err(e) => println!("error: {e}"),
    }

    match dividir(1.0, 0.0) {
        Ok(v) => println!("1/0 = {v}"),
        Err(e) => println!("error esperado: {e}"),
    }

    // Atajos útiles (¡conocelos, aparecen en todo código real!):
    let r = dividir(9.0, 3.0);
    println!("is_ok: {}", r.is_ok());
    println!("unwrap_or: {}", dividir(1.0, 0.0).unwrap_or(f64::NAN));
    // .unwrap() extrae el Ok o hace PANIC si es Err. En producción es mala
    // señal salvo en casos donde el error es imposible por construcción;
    // .expect("mensaje") es igual pero documenta por qué "no puede fallar".
}

// ----------------------------------------------------------------------------
// EJEMPLO 2: el operador `?` — propagación de errores sin ruido
// ----------------------------------------------------------------------------
// `expr?` significa:
//   - si expr es Ok(v)  → evalúa a v y seguí ejecutando
//   - si expr es Err(e) → RETURN Err(e.into()) inmediatamente
//
// Es el patrón de C:
//     int rc = hacer_algo();
//     if (rc != 0) return rc;
// ...comprimido en UN carácter, sin poder olvidarlo, y con conversión
// automática del tipo de error vía From/Into (ver ejemplo 3).
// ----------------------------------------------------------------------------

/// Parsea "3.5,2.0" y devuelve la suma de ambos números.
fn sumar_par(entrada: &str) -> Result<f64, ParseFloatError> {
    let mut partes = entrada.split(',');

    // .unwrap_or("") para el ejemplo; parse() devuelve Result y el `?`
    // propaga el error de parseo hacia el caller si algo no es numérico.
    let a: f64 = partes.next().unwrap_or("").trim().parse()?;
    let b: f64 = partes.next().unwrap_or("").trim().parse()?;
    //                                                     ^
    // Sin `?` esto serían 4-6 líneas de match por CADA parse.

    Ok(a + b)
}

fn ejemplo_2_operador_interrogacion() {
    println!("--- Ejemplo 2: operador ? ---");
    println!("sumar_par(\"3.5, 2.0\") = {:?}", sumar_par("3.5, 2.0"));
    println!("sumar_par(\"3.5, hola\") = {:?}", sumar_par("3.5, hola"));
}

// ----------------------------------------------------------------------------
// EJEMPLO 3: errores custom con Display + From — el patrón profesional
// ----------------------------------------------------------------------------
// En una app real una función puede fallar por MOTIVOS DISTINTOS (parseo,
// validación, IO...). El patrón idiomático: un enum con una variante por
// causa. El caller puede hacer match por causa — muchísimo más rico que
// el `int` de errno o que capturar std::exception genérica.
// ----------------------------------------------------------------------------
#[derive(Debug)]
enum ErrorMedicion {
    /// El texto no era un número (guardamos el error original adentro).
    ParseoInvalido(ParseFloatError),
    /// Número válido pero físicamente imposible para nuestro rig.
    FueraDeRango { valor: f64, max: f64 },
}

// Display: cómo se muestra el error a un humano.
// (El trait std::error::Error requiere Debug + Display; con estos dos impl
// nuestro tipo se integra con todo el ecosistema de errores de Rust.)
impl fmt::Display for ErrorMedicion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorMedicion::ParseoInvalido(e) => write!(f, "no es un número: {e}"),
            ErrorMedicion::FueraDeRango { valor, max } => {
                write!(f, "{valor} excede el máximo físico de {max}")
            }
        }
    }
}

impl std::error::Error for ErrorMedicion {}

// From: la MAGIA detrás de `?`. Al escribir `parse()?` dentro de una función
// que devuelve Result<_, ErrorMedicion>, el `?` llama a este From para
// convertir ParseFloatError → ErrorMedicion automáticamente.
// Es lo que las crates `anyhow`/`thiserror` automatizan (las usamos en la
// fase 3); acá lo hacemos a mano para entender QUÉ generan.
impl From<ParseFloatError> for ErrorMedicion {
    fn from(e: ParseFloatError) -> Self {
        ErrorMedicion::ParseoInvalido(e)
    }
}

/// Parsea una lectura de voltaje y valida que sea físicamente posible.
fn parsear_voltaje(texto: &str) -> Result<f64, ErrorMedicion> {
    const MAX_FISICO: f64 = 5.0;

    // parse() falla con ParseFloatError; `?` + From lo convierte solo.
    let v: f64 = texto.trim().parse()?;

    if v > MAX_FISICO {
        return Err(ErrorMedicion::FueraDeRango { valor: v, max: MAX_FISICO });
    }
    Ok(v)
}

fn ejemplo_3_error_custom() {
    println!("--- Ejemplo 3: errores custom ---");

    for entrada in ["1.05", "abc", "12.7"] {
        match parsear_voltaje(entrada) {
            Ok(v) => println!("'{entrada}' → {v} V ✓"),
            // Podemos reaccionar DISTINTO según la causa — probá hacer esto
            // con errno o con catch(...) y me contás.
            Err(ErrorMedicion::ParseoInvalido(_)) => {
                println!("'{entrada}' → entrada corrupta, pedir de nuevo")
            }
            Err(e @ ErrorMedicion::FueraDeRango { .. }) => {
                println!("'{entrada}' → ALARMA: {e}")
            }
        }
    }
}

// ----------------------------------------------------------------------------
// EJEMPLO 4: Option — la ausencia de valor, sin NULL
// ----------------------------------------------------------------------------
// enum Option<T> { Some(T), None }
//
// Rust NO tiene null. Donde en C devolverías NULL (y el caller olvida
// chequear → segfault), acá devolvés Option y el compilador OBLIGA a
// considerar el caso None. Es std::optional de C++17, pero sin poder
// derreferenciarlo sin chequear.
// `?` también funciona con Option (propaga el None).
// ----------------------------------------------------------------------------
fn buscar_canal(nombre: &str, canales: &[(&str, u8)]) -> Option<u8> {
    // find devuelve Option<&(&str, u8)>; map lo transforma si es Some.
    canales.iter().find(|(n, _)| *n == nombre).map(|(_, id)| *id)
}

fn ejemplo_4_option() {
    println!("--- Ejemplo 4: Option ---");
    let canales = [("vcore", 0u8), ("vddq", 1), ("temp0", 7)];

    match buscar_canal("vddq", &canales) {
        Some(id) => println!("vddq está en el canal {id}"),
        None => println!("vddq no existe"),
    }

    // Combinadores: el estilo funcional evita el match cuando es trivial.
    let id = buscar_canal("no_existe", &canales).unwrap_or(255);
    println!("canal por defecto: {id}");
}
