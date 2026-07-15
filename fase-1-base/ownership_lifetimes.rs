// ============================================================================
// OWNERSHIP Y LIFETIMES — el corazón de Rust
// ============================================================================
// Compilar y correr:  rustc ownership_lifetimes.rs -o /tmp/ol && /tmp/ol
//
// Si venís de C/C++, este archivo es EL más importante del repo.
// Ownership es la respuesta de Rust a las dos preguntas que en C se responden
// "con disciplina y valgrind":
//
//   1. ¿Quién libera esta memoria?        (en C: convención + free() manual)
//   2. ¿Este puntero sigue siendo válido? (en C: rezá / usa ASAN)
//
// Rust responde las dos EN TIEMPO DE COMPILACIÓN. Cero costo en runtime:
// no hay garbage collector, no hay reference counting implícito.
// ============================================================================

fn main() {
    ejemplo_1_move();
    ejemplo_2_clone_vs_copy();
    ejemplo_3_borrows();
    ejemplo_4_dangling();
    ejemplo_5_lifetimes_explicitos();
    ejemplo_6_lifetimes_en_structs();
}

// ----------------------------------------------------------------------------
// EJEMPLO 1: Move semantics — la asignación TRANSFIERE la propiedad
// ----------------------------------------------------------------------------
fn ejemplo_1_move() {
    println!("--- Ejemplo 1: move ---");

    // `String` es un buffer en el heap (como un char* con malloc en C,
    // o std::string en C++). La variable `a` es la DUEÑA de ese buffer.
    let a = String::from("hola");

    // En C++: `std::string b = a;` hace una COPIA PROFUNDA (caro, implícito).
    // En Rust: esto hace un MOVE. El buffer del heap NO se copia; solo se
    // copian los 3 words del stack (ptr, len, capacity) y `a` queda INVÁLIDA.
    // Es exactamente std::move(a) de C++11... pero obligatorio y verificado:
    // en C++ usar un objeto "moved-from" compila y es UB-ish; acá NO compila.
    let b = a;

    println!("b = {b}");

    // Descomentá la siguiente línea y el compilador te frena:
    //   error[E0382]: borrow of moved value: `a`
    // println!("a = {a}");

    // ¿Por qué? Si `a` y `b` fueran válidas a la vez, al salir del scope
    // AMBAS liberarían el mismo buffer → double free. En C eso es un CVE;
    // en Rust es un error de compilación.
} // <- acá `b` sale de scope y Rust llama a `drop` (el free) AUTOMÁTICAMENTE.
  //    Es RAII de C++ (destructores), pero sin poder olvidarte ni duplicarlo.

// ----------------------------------------------------------------------------
// EJEMPLO 2: Clone vs Copy — cuándo NO hay move
// ----------------------------------------------------------------------------
fn ejemplo_2_clone_vs_copy() {
    println!("--- Ejemplo 2: clone vs copy ---");

    // Los tipos "planos" que viven 100% en el stack (i32, f64, bool, char,
    // tuplas de estos) implementan el trait `Copy`: la asignación copia bits
    // y las dos variables quedan válidas. Igual que `int b = a;` en C.
    let x: i32 = 42;
    let y = x; // copia bit a bit, x sigue viva
    println!("x = {x}, y = {y}"); // ambas válidas, sin problema

    // Para tipos con heap (String, Vec, ...), si de verdad querés una copia
    // profunda, la pedís EXPLÍCITAMENTE con .clone().
    // Filosofía de Rust: lo caro se escribe, no pasa de casualidad.
    // (En C++ la copia profunda es el default silencioso; en Rust es opt-in.)
    let s1 = String::from("dato");
    let s2 = s1.clone(); // ahora sí: dos buffers independientes en el heap
    println!("s1 = {s1}, s2 = {s2}");
}

// ----------------------------------------------------------------------------
// EJEMPLO 3: Borrows — prestar en vez de transferir
// ----------------------------------------------------------------------------
// Regla de oro del borrow checker (memorizala, es TODO el sistema):
//
//   En un momento dado podés tener:
//     - CUALQUIER cantidad de referencias inmutables (&T), O BIEN
//     - EXACTAMENTE UNA referencia mutable (&mut T).
//   Nunca las dos cosas a la vez.
//
// ¿Por qué? Es la definición de data race aplicada a un solo thread:
// "alguien escribe mientras otro lee". C te deja hacerlo y el resultado es
// iterator invalidation, punteros a memoria realocada, etc. Rust lo prohíbe.
// ----------------------------------------------------------------------------
fn ejemplo_3_borrows() {
    println!("--- Ejemplo 3: borrows ---");

    let mut texto = String::from("hola");

    // Préstamo inmutable: como pasar `const char*` en C, pero verificado.
    let len = longitud(&texto);
    println!("longitud de '{texto}' = {len}"); // texto sigue siendo nuestra

    // Préstamo mutable: la función puede modificar, pero mientras exista
    // este préstamo NADIE más puede ni leer ni escribir `texto`.
    agregar_mundo(&mut texto);
    println!("después de mutar: {texto}");

    // Esto NO compila (léelo, es el error más común al empezar):
    //
    //   let r1 = &texto;          // préstamo inmutable vivo...
    //   let r2 = &mut texto;      // error[E0502]: no se puede prestar mutable
    //   println!("{r1}");         // ...porque r1 se usa después
    //
    // El caso real que esto previene: en C++, hacer push_back a un vector
    // mientras iterás sobre él puede realocar el buffer y dejar tu iterador
    // apuntando a memoria liberada. En Rust ese patrón directamente no compila.
}

// `&String` = "te presto el valor para leer". No hay transferencia de ownership.
fn longitud(s: &String) -> usize {
    s.len()
}

// `&mut String` = "te presto el valor para escribir, devolvémelo cuando termines".
fn agregar_mundo(s: &mut String) {
    s.push_str(", mundo");
}

// ----------------------------------------------------------------------------
// EJEMPLO 4: punteros colgantes (dangling pointers) — el clásico de C
// ----------------------------------------------------------------------------
fn ejemplo_4_dangling() {
    println!("--- Ejemplo 4: no hay dangling pointers ---");

    // En C, esto compila y explota (o peor: "funciona" hasta que no):
    //
    //   char* dame_saludo(void) {
    //       char buf[16] = "hola";
    //       return buf;            // ⚠️ devuelve puntero a stack muerto
    //   }
    //
    // El equivalente en Rust NO COMPILA:
    //
    //   fn dame_saludo<'a>() -> &'a String {
    //       let s = String::from("hola");
    //       &s   // error[E0515]: cannot return reference to local variable
    //   }        // `s` muere acá; la referencia apuntaría a memoria liberada
    //
    // La solución idiomática: devolver el VALOR (transferir ownership).
    // El move es barato (3 words), no copia el heap.
    let saludo = dame_saludo();
    println!("{saludo}");
}

fn dame_saludo() -> String {
    let s = String::from("hola desde el heap");
    s // move: el ownership sale de la función hacia el caller. Sin copia.
}

// ----------------------------------------------------------------------------
// EJEMPLO 5: lifetimes explícitos — cuando el compilador necesita ayuda
// ----------------------------------------------------------------------------
// Un lifetime NO crea nada en runtime: es una ANOTACIÓN que describe una
// relación que ya existe: "esta referencia no puede vivir más que aquella".
// Pensalo como documentación verificada por el compilador sobre la validez
// de los punteros — lo que en C va en un comentario ("el caller debe
// asegurarse de que buf viva mientras...") acá es parte del tipo.
// ----------------------------------------------------------------------------
fn ejemplo_5_lifetimes_explicitos() {
    println!("--- Ejemplo 5: lifetimes explícitos ---");

    let s1 = String::from("cadena larga");
    let resultado;
    {
        let s2 = String::from("corta");
        // `mas_larga` devuelve una referencia atada a AMBAS entradas:
        // el resultado vive como máximo lo que viva la MÁS CORTA de las dos.
        resultado = mas_larga(s1.as_str(), s2.as_str());
        println!("la más larga es: {resultado}");
    } // s2 muere acá → `resultado` ya no puede usarse después de esta llave.

    // Descomentá esto y no compila: `resultado` podría apuntar a s2 (muerta).
    // println!("{resultado}");
}

// ¿Por qué hace falta `'a` acá? Porque el compilador solo mira la FIRMA
// (no el cuerpo) para razonar sobre las llamadas. Sin la anotación no puede
// saber si el retorno apunta a `x`, a `y`, o a ambas.
// `'a` dice: "el retorno vive mientras vivan x E y a la vez".
fn mas_larga<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}

// ----------------------------------------------------------------------------
// EJEMPLO 6: lifetimes en structs — guardar referencias
// ----------------------------------------------------------------------------
// Un struct que guarda una referencia debe declarar el lifetime: le dice al
// compilador "este struct NO puede sobrevivir al dato que referencia".
// En C++ guardarías un puntero/referencia y cruzarías los dedos para que el
// objeto original no muera antes (el clásico use-after-free con callbacks).
// ----------------------------------------------------------------------------
struct Extracto<'a> {
    // `parte` apunta DENTRO de un String ajeno. No es dueño de nada:
    // es un &str "prestado", y el struct entero hereda esa restricción.
    parte: &'a str,
}

impl<'a> Extracto<'a> {
    fn primera_palabra(texto: &'a str) -> Extracto<'a> {
        let palabra = texto.split_whitespace().next().unwrap_or("");
        Extracto { parte: palabra }
    }
}

fn ejemplo_6_lifetimes_en_structs() {
    println!("--- Ejemplo 6: lifetimes en structs ---");

    let novela = String::from("Llamadme Ismael. Hace unos años...");
    let extracto = Extracto::primera_palabra(&novela);
    println!("primera palabra: {}", extracto.parte);

    // Si intentaras hacer esto, no compila:
    //
    //   let extracto;
    //   {
    //       let novela = String::from("texto corto");
    //       extracto = Extracto::primera_palabra(&novela);
    //   } // novela muere acá
    //   println!("{}", extracto.parte); // error: `novela` no vive lo suficiente
    //
    // En C++ el equivalente (guardar string_view de un string temporal)
    // compila sin quejarse y lee memoria liberada.
}
