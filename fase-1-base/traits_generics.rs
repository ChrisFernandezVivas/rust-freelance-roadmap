// ============================================================================
// TRAITS Y GENÉRICOS — el sistema de abstracción de Rust
// ============================================================================
// Compilar y correr:  rustc traits_generics.rs -o /tmp/tg && /tmp/tg
//
// Mapa mental para alguien que viene de C++:
//
//   trait                  ≈ interfaz / clase abstracta pura (pero sin herencia
//                            de estado, y se puede implementar para tipos ajenos)
//   generics <T: Trait>    ≈ templates de C++, PERO verificados en la
//                            declaración (como C++20 concepts), no al instanciar
//   dyn Trait              ≈ funciones virtuales / vtable de C++
//   impl Trait             ≈ "algún tipo concreto que cumple esto" (sin vtable)
//
// La distinción clave (aparece en TODA entrevista de Rust):
//   - Genéricos  → despacho ESTÁTICO (monomorfización): el compilador genera
//     una copia de la función por cada tipo. Cero costo en runtime, binario
//     más grande. Igual que templates de C++.
//   - dyn Trait  → despacho DINÁMICO (vtable): una sola copia de la función,
//     salto indirecto en runtime. Igual que `virtual` en C++.
// ============================================================================

use std::fmt;

fn main() {
    ejemplo_1_trait_basico();
    ejemplo_2_genericos();
    ejemplo_3_dyn_trait();
    ejemplo_4_traits_estandar();
    ejemplo_5_trait_bounds_multiples();
}

// ----------------------------------------------------------------------------
// EJEMPLO 1: definir e implementar un trait
// ----------------------------------------------------------------------------
// Modelamos sensores de un banco de validación (ambiente familiar 😉).
trait Sensor {
    // Método requerido: cada tipo que implemente Sensor DEBE definirlo.
    fn leer(&self) -> f64;

    // Método requerido: nombre del sensor para reportes.
    fn nombre(&self) -> String;

    // Método con implementación por DEFECTO: los implementadores lo heredan
    // gratis, pero pueden sobreescribirlo. Parecido a un método no-puro en
    // una clase abstracta de C++, pero SIN estado compartido: un trait no
    // tiene campos, solo comportamiento.
    fn reporte(&self) -> String {
        format!("[{}] lectura = {:.2}", self.nombre(), self.leer())
    }
}

struct SensorTemperatura {
    celsius: f64,
}

struct SensorVoltaje {
    volts: f64,
}

// "impl Trait for Tipo" — la implementación va SEPARADA de la definición del
// struct. Esto permite algo imposible en C++/Java: implementar TU trait para
// un tipo de una biblioteca ajena (o un trait ajeno para tu tipo).
impl Sensor for SensorTemperatura {
    fn leer(&self) -> f64 {
        self.celsius
    }
    fn nombre(&self) -> String {
        String::from("temp")
    }
}

impl Sensor for SensorVoltaje {
    fn leer(&self) -> f64 {
        self.volts
    }
    fn nombre(&self) -> String {
        String::from("vcore")
    }
    // Sobreescribimos el default para dar más precisión.
    fn reporte(&self) -> String {
        format!("[{}] lectura = {:.4} V", self.nombre(), self.leer())
    }
}

fn ejemplo_1_trait_basico() {
    println!("--- Ejemplo 1: trait básico ---");
    let t = SensorTemperatura { celsius: 71.3 };
    let v = SensorVoltaje { volts: 1.0523 };
    println!("{}", t.reporte()); // usa el default del trait
    println!("{}", v.reporte()); // usa el override
}

// ----------------------------------------------------------------------------
// EJEMPLO 2: genéricos con trait bounds — despacho estático
// ----------------------------------------------------------------------------
// `<S: Sensor>` se lee: "para cualquier tipo S que implemente Sensor".
//
// Diferencia CLAVE con templates de C++: acá el cuerpo de la función se
// verifica contra el bound EN LA DECLARACIÓN. Si adentro llamás a un método
// que Sensor no tiene, el error aparece acá, claro y corto — no las 300
// líneas de vómito de template-instantiation de C++ (pre-concepts).
// ----------------------------------------------------------------------------
fn fuera_de_rango<S: Sensor>(sensor: &S, min: f64, max: f64) -> bool {
    let valor = sensor.leer();
    valor < min || valor > max
}

fn ejemplo_2_genericos() {
    println!("--- Ejemplo 2: genéricos (despacho estático) ---");
    let t = SensorTemperatura { celsius: 105.0 };

    // Al compilar, Rust genera una versión especializada
    // `fuera_de_rango::<SensorTemperatura>` — MONOMORFIZACIÓN.
    // Es exactamente lo que hace un template de C++: la llamada es directa,
    // inlineable, sin salto por vtable. Costo en runtime: cero.
    let alarma = fuera_de_rango(&t, 0.0, 100.0);
    println!("temp fuera de rango: {alarma}");
}

// ----------------------------------------------------------------------------
// EJEMPLO 3: dyn Trait — despacho dinámico (vtables)
// ----------------------------------------------------------------------------
// ¿Cuándo NO alcanza con genéricos? Cuando necesitás una colección
// HETEROGÉNEA: un Vec con sensores de distintos tipos concretos a la vez.
// Un Vec<T> genérico fija UN solo T; para mezclar tipos necesitás borrar el
// tipo concreto ("type erasure") detrás de un puntero: `Box<dyn Sensor>`.
//
// En C++ sería: std::vector<std::unique_ptr<SensorBase>> con métodos virtual.
// Mismo mecanismo, mismo costo: cada llamada pasa por la vtable.
// La diferencia: en Rust ELEGÍS vtable solo donde la necesitás; `virtual`
// no se te filtra por toda la jerarquía porque no HAY jerarquía.
// ----------------------------------------------------------------------------
fn ejemplo_3_dyn_trait() {
    println!("--- Ejemplo 3: dyn Trait (despacho dinámico) ---");

    // Box = puntero al heap con ownership (como unique_ptr).
    // dyn Sensor = "algún tipo que implementa Sensor, resuelto en runtime".
    let tablero: Vec<Box<dyn Sensor>> = vec![
        Box::new(SensorTemperatura { celsius: 68.9 }),
        Box::new(SensorVoltaje { volts: 1.0498 }),
        Box::new(SensorTemperatura { celsius: 72.1 }),
    ];

    for sensor in &tablero {
        // Esta llamada es un salto indirecto vía vtable (fat pointer:
        // puntero al dato + puntero a la tabla de métodos).
        println!("{}", sensor.reporte());
    }
}

// ----------------------------------------------------------------------------
// EJEMPLO 4: traits estándar — derive y operadores
// ----------------------------------------------------------------------------
// La stdlib define traits que "enchufan" tu tipo al lenguaje:
//   Debug    → formateo {:?} (para desarrolladores)
//   Display  → formateo {}   (para usuarios)      ≈ operator<< de C++
//   Clone    → copia profunda explícita           ≈ constructor de copia
//   PartialEq→ operador ==                        ≈ operator==
//   Default  → valor por defecto                  ≈ constructor por defecto
//
// `#[derive(...)]` genera la implementación automáticamente — como el
// `= default` de C++ pero para mucho más, y sin escribir boilerplate.
// ----------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq)]
struct Medicion {
    canal: u8,
    valor: f64,
}

// Display no se puede derivar: el formato "para humanos" es decisión tuya.
impl fmt::Display for Medicion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "canal {} → {:.3}", self.canal, self.valor)
    }
}

fn ejemplo_4_traits_estandar() {
    println!("--- Ejemplo 4: traits estándar ---");
    let m1 = Medicion { canal: 3, valor: 0.99871 };
    let m2 = m1.clone();

    println!("Debug:   {m1:?}"); // gracias a #[derive(Debug)]
    println!("Display: {m1}"); // gracias a nuestro impl Display
    println!("m1 == m2: {}", m1 == m2); // gracias a #[derive(PartialEq)]
}

// ----------------------------------------------------------------------------
// EJEMPLO 5: bounds múltiples y `where`
// ----------------------------------------------------------------------------
// Un parámetro puede exigir VARIOS traits a la vez con `+`.
// Cuando los bounds se ponen largos, `where` los saca de la firma
// (puro azúcar sintáctico, misma semántica).
// ----------------------------------------------------------------------------
fn describir_todo<T>(items: &[T]) -> String
where
    T: fmt::Display + Clone, // T debe poder imprimirse Y clonarse
{
    items
        .iter()
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join(" | ")
}

fn ejemplo_5_trait_bounds_multiples() {
    println!("--- Ejemplo 5: bounds múltiples ---");
    let mediciones = vec![
        Medicion { canal: 0, valor: 1.05 },
        Medicion { canal: 1, valor: 0.98 },
    ];
    println!("{}", describir_todo(&mediciones));

    // Bonus: la MISMA función sirve para i32, porque i32 también cumple
    // Display + Clone. Una sola definición, N monomorfizaciones.
    println!("{}", describir_todo(&[10, 20, 30]));
}
