// ============================================================================
// ITERADORES Y CLOSURES — el estilo funcional de Rust (a costo cero)
// ============================================================================
// Compilar y correr:  rustc iterators_closures.rs -o /tmp/ic && /tmp/ic
//
// Si venís de Python: map/filter te van a resultar familiares, PERO acá no
// hay costo de interprete — las cadenas de iteradores compilan al mismo
// código máquina que el loop `for` equivalente escrito a mano ("zero-cost
// abstraction"). El capítulo 13 del Rust Book muestra los benchmarks.
//
// Si venís de C++: un iterador de Rust NO es un puntero disfrazado como en
// C++ (begin/end). Es cualquier tipo que implementa:
//
//     trait Iterator { type Item; fn next(&mut self) -> Option<Self::Item>; }
//
// Una sola operación: "dame el siguiente, o None si terminaste". Todo lo
// demás (map, filter, zip...) son adaptadores construidos sobre next().
// Y el borrow checker garantiza lo que en C++ es tu responsabilidad:
// no hay iterator invalidation posible.
// ============================================================================

fn main() {
    ejemplo_1_closures();
    ejemplo_2_capturas();
    ejemplo_3_adaptadores();
    ejemplo_4_lazy();
    ejemplo_5_pipeline_real();
    ejemplo_6_iterador_propio();
}

// ----------------------------------------------------------------------------
// EJEMPLO 1: closures — funciones anónimas que capturan su entorno
// ----------------------------------------------------------------------------
fn ejemplo_1_closures() {
    println!("--- Ejemplo 1: closures ---");

    // Sintaxis: |parámetros| cuerpo. Los tipos casi siempre se infieren.
    let doble = |x: i32| x * 2;
    println!("doble(21) = {}", doble(21));

    // La gracia vs. un puntero a función de C: el closure CAPTURA variables
    // del scope donde se define. En C harías esto pasando un void* de
    // "contexto" a mano (el clásico `void (*cb)(void*), void* user_data`).
    let factor = 3;
    let triple = |x: i32| x * factor; // captura `factor` por referencia
    println!("triple(14) = {}", triple(14));

    // En C++ esto es [factor](int x) { return x * factor; } — casi igual.
    // La diferencia: en C++ VOS elegís capturar por valor o referencia y si
    // te equivocás (capturar por & algo que muere) es UB. En Rust el
    // compilador infiere la captura mínima necesaria Y verifica lifetimes.
}

// ----------------------------------------------------------------------------
// EJEMPLO 2: los tres "sabores" de captura — Fn, FnMut, FnOnce
// ----------------------------------------------------------------------------
// Rust clasifica los closures según CÓMO usan lo capturado:
//   Fn     → solo lee            (captura por &T)
//   FnMut  → muta                (captura por &mut T)
//   FnOnce → consume/mueve       (captura por valor; solo se puede llamar 1 vez)
// Es ownership aplicado a closures. Estos traits aparecen en las firmas de
// cualquier función que recibe callbacks (y en TODA la API de iteradores).
// ----------------------------------------------------------------------------
fn aplicar<F: Fn(i32) -> i32>(f: F, v: i32) -> i32 {
    f(v)
}

fn ejemplo_2_capturas() {
    println!("--- Ejemplo 2: Fn / FnMut / FnOnce ---");

    // Fn: solo lee `base`.
    let base = 100;
    println!("aplicar = {}", aplicar(|x| x + base, 5));

    // FnMut: muta su entorno (el contador).
    let mut contador = 0;
    let mut incrementar = || {
        contador += 1;
        contador
    };
    incrementar();
    incrementar();
    println!("contador = {}", incrementar()); // 3

    // FnOnce + `move`: el closure se APROPIA de `mensaje` (ownership).
    // Crucial con threads: `thread::spawn` exige `move` porque el thread
    // puede vivir más que el stack frame que creó el closure — exactamente
    // el bug de C++ de capturar por referencia en un std::thread.
    let mensaje = String::from("dato movido al closure");
    let consumir = move || mensaje; // devuelve el String, consumiéndolo
    println!("consumir() = {}", consumir());
    // println!("{mensaje}"); // ← no compila: `mensaje` fue movido
}

// ----------------------------------------------------------------------------
// EJEMPLO 3: adaptadores fundamentales — map, filter, collect
// ----------------------------------------------------------------------------
fn ejemplo_3_adaptadores() {
    println!("--- Ejemplo 3: map / filter / collect ---");

    let lecturas: Vec<f64> = vec![0.98, 1.02, 1.15, 0.99, 1.30, 1.01];

    // Python:  [round(x*1000) for x in lecturas if x <= 1.1]
    // Rust:
    let en_rango_mv: Vec<i64> = lecturas
        .iter() //                      itera por referencia (&f64)
        .filter(|v| **v <= 1.1) //      descarta outliers
        .map(|v| (v * 1000.0).round() as i64) // volts → milivolts
        .collect(); //                  materializa en un Vec

    println!("lecturas en rango (mV): {en_rango_mv:?}");

    // Reducciones directas (no reinventes el loop):
    let suma: f64 = lecturas.iter().sum();
    let max = lecturas.iter().cloned().fold(f64::MIN, f64::max);
    println!("promedio = {:.3}, max = {max}", suma / lecturas.len() as f64);

    // Los tres modos de iterar (¡otra vez ownership!):
    //   .iter()      → &T       (prestado, solo lectura)     ← el más común
    //   .iter_mut()  → &mut T   (prestado, modificable in-place)
    //   .into_iter() → T        (consume la colección, te da los valores)
    let mut valores = vec![1, 2, 3];
    valores.iter_mut().for_each(|v| *v *= 10);
    println!("mutados in-place: {valores:?}");
}

// ----------------------------------------------------------------------------
// EJEMPLO 4: los iteradores son LAZY (perezosos)
// ----------------------------------------------------------------------------
// Encadenar adaptadores NO ejecuta nada: construye una "receta" (un tipo que
// envuelve al iterador anterior). Solo un CONSUMIDOR (collect, sum, for,
// next...) tira de la cadena y ejecuta. Igual que los generadores de Python,
// pero resuelto en compile time: el optimizador fusiona toda la cadena en
// un único loop sin allocaciones intermedias.
// ----------------------------------------------------------------------------
fn ejemplo_4_lazy() {
    println!("--- Ejemplo 4: lazy evaluation ---");

    let mut evaluados = 0;

    let resultado: Vec<i32> = (1..=1000) // rango de 1000 elementos
        .map(|x| {
            evaluados += 1; // contamos cuántas veces corre el map
            x * x
        })
        .take(5) // ...pero solo pedimos 5
        .collect();

    println!("resultado: {resultado:?}");
    // Como take(5) corta la demanda, el map corrió SOLO 5 veces, no 1000.
    // En Python con listas (no generadores) habrías computado las 1000.
    println!("el map se evaluó {evaluados} veces (¡no 1000!)");
}

// ----------------------------------------------------------------------------
// EJEMPLO 5: pipeline realista — parsear un log de mediciones
// ----------------------------------------------------------------------------
fn ejemplo_5_pipeline_real() {
    println!("--- Ejemplo 5: pipeline real ---");

    // Simulamos un log tipo CSV con líneas corruptas (la vida real).
    let log = "\
vcore,1.048
vddq,1.201
CORRUPTO
temp0,68.5
vcore,abc
vcore,1.052";

    // Objetivo: promedio de las lecturas válidas de vcore.
    // Fijate cómo filter_map maneja el "puede fallar" SIN abortar el
    // pipeline: parse() devuelve Result, .ok() lo vuelve Option, y
    // filter_map descarta los None. Errores manejados, cero excepciones.
    let lecturas_vcore: Vec<f64> = log
        .lines()
        .filter_map(|linea| {
            let (nombre, valor) = linea.split_once(',')?; // None si no hay coma
            if nombre != "vcore" {
                return None;
            }
            valor.trim().parse::<f64>().ok() // None si no parsea
        })
        .collect();

    let promedio: f64 = lecturas_vcore.iter().sum::<f64>() / lecturas_vcore.len() as f64;
    println!("lecturas válidas de vcore: {lecturas_vcore:?}");
    println!("promedio = {promedio:.4} V");
}

// ----------------------------------------------------------------------------
// EJEMPLO 6: implementar TU propio iterador
// ----------------------------------------------------------------------------
// Solo hay que implementar next(). A cambio, tu tipo gana GRATIS los ~70
// métodos de Iterator (map, filter, zip, take, skip...) porque todos tienen
// implementación por defecto sobre next(). Este es el poder de los traits
// con métodos default que vimos en traits_generics.rs.
// ----------------------------------------------------------------------------
struct Fibonacci {
    actual: u64,
    siguiente: u64,
}

impl Iterator for Fibonacci {
    type Item = u64; // "tipo asociado": el tipo que produce el iterador

    fn next(&mut self) -> Option<u64> {
        let valor = self.actual;
        // Nota: sin checked_add esto haría overflow con muchos elementos;
        // u64 aguanta hasta fib(93), suficiente para el ejemplo.
        self.actual = self.siguiente;
        self.siguiente = valor + self.siguiente;
        Some(valor) // infinito: nunca devolvemos None (take() lo corta)
    }
}

fn ejemplo_6_iterador_propio() {
    println!("--- Ejemplo 6: iterador propio ---");

    let fib = Fibonacci { actual: 0, siguiente: 1 };

    // Nuestro iterador infinito, usado con adaptadores estándar:
    let pares: Vec<u64> = fib
        .filter(|n| n % 2 == 0) // solo pares
        .take(6) // los primeros 6
        .collect();

    println!("primeros 6 fibonacci pares: {pares:?}");
}
