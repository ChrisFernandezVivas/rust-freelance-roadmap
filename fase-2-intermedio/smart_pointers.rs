// ============================================================================
// SMART POINTERS — Box, Rc, RefCell: cuándo usar cada uno
// ============================================================================
// Compilar y correr:  rustc smart_pointers.rs -o /tmp/sp && /tmp/sp
//
// Tabla de equivalencias mental para alguien que viene de C++:
//
//   Rust            C++                     Diferencia clave
//   ─────────────── ─────────────────────── ─────────────────────────────────
//   Box<T>          std::unique_ptr<T>      Igual idea; imposible usar tras move
//   Rc<T>           std::shared_ptr<T>      SOLO single-thread (no atómico)
//   Arc<T>          std::shared_ptr<T>      Multi-thread (contador atómico)
//   RefCell<T>      — (no existe)           Borrow checking EN RUNTIME
//   Rc<RefCell<T>>  shared_ptr<T> mutable   Compartido Y mutable, verificado
//
// Regla de decisión rápida (memorizala):
//   ¿Un solo dueño, dato en heap?            → Box<T>
//   ¿Varios dueños, solo lectura, 1 thread?  → Rc<T>
//   ¿Varios dueños Y mutación, 1 thread?     → Rc<RefCell<T>>
//   ¿Varios threads?                         → Arc<T> / Arc<Mutex<T>> (fase 2: concurrencia/)
// ============================================================================

use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    ejemplo_1_box();
    ejemplo_2_box_recursivo();
    ejemplo_3_rc();
    ejemplo_4_refcell();
    ejemplo_5_rc_refcell();
}

// ----------------------------------------------------------------------------
// EJEMPLO 1: Box<T> — un dueño, dato en el heap
// ----------------------------------------------------------------------------
// Box es EL smart pointer básico: malloc + free automático + un solo dueño.
// Exactamente unique_ptr de C++, con una mejora: en C++ podés usar un
// unique_ptr después de moverlo (compila, revienta en runtime con nullptr);
// en Rust usar un Box movido NO COMPILA.
//
// ¿Cuándo usarlo?
//   a) Tipos grandes que no querés copiar por el stack.
//   b) Tipos recursivos (ejemplo 2) — el caso donde es OBLIGATORIO.
//   c) Trait objects: Box<dyn Trait> (visto en fase 1).
// ----------------------------------------------------------------------------
fn ejemplo_1_box() {
    println!("--- Ejemplo 1: Box ---");

    // 8 bytes en el stack (el puntero); el i32 vive en el heap.
    // Para un i32 solo es tonto, claro — es el ejemplo mínimo.
    let en_heap: Box<i32> = Box::new(42);

    // Deref automático: se usa como si fuera el valor (como operator* de
    // unique_ptr, pero casi siempre implícito gracias al trait Deref).
    println!("valor = {}, valor + 1 = {}", en_heap, *en_heap + 1);
} // drop automático: free del heap acá. Sin fugas, sin double-free posible.

// ----------------------------------------------------------------------------
// EJEMPLO 2: Box para tipos recursivos — el caso obligatorio
// ----------------------------------------------------------------------------
// Esto NO compila:
//
//     enum Lista { Nodo(i32, Lista), Fin }   // error[E0072]
//
// ¿Por qué? El compilador necesita saber el TAMAÑO de Lista, y una Lista
// que contiene una Lista que contiene una Lista... es infinita.
// En C harías `struct nodo { int v; struct nodo *sig; };` — el puntero
// rompe la recursión porque su tamaño es fijo (8 bytes). Box es eso:
// ----------------------------------------------------------------------------
#[derive(Debug)]
enum Lista {
    // Box<Lista> mide 8 bytes, sea cual sea el largo de la lista. Resuelto.
    Nodo(i32, Box<Lista>),
    Fin,
}

fn ejemplo_2_box_recursivo() {
    println!("--- Ejemplo 2: Box recursivo ---");
    use Lista::{Fin, Nodo};

    // 1 → 2 → 3 → Fin (cada Nodo vive en el heap, enlazado por Box)
    let lista = Nodo(1, Box::new(Nodo(2, Box::new(Nodo(3, Box::new(Fin))))));

    // Recorremos con referencias (sin consumir la lista):
    let mut actual = &lista;
    while let Nodo(valor, siguiente) = actual {
        print!("{valor} → ");
        actual = siguiente;
    }
    println!("Fin");
}

// ----------------------------------------------------------------------------
// EJEMPLO 3: Rc<T> — VARIOS dueños (reference counting), solo lectura
// ----------------------------------------------------------------------------
// A veces "un solo dueño" no alcanza: dos estructuras necesitan COMPARTIR
// el mismo dato y ninguna es claramente "la dueña" (grafos, cachés,
// configuración compartida). Rc = shared_ptr de C++ con dos diferencias:
//
//   1. El contador NO es atómico → más barato, pero SOLO single-thread.
//      (El compilador lo garantiza: Rc no es Send. Si lo intentás mover a
//      un thread, no compila. En C++ elegir mal es una carrera silenciosa.)
//   2. Rc te da acceso SOLO LECTURA al contenido. Para mutar: ejemplo 5.
// ----------------------------------------------------------------------------
fn ejemplo_3_rc() {
    println!("--- Ejemplo 3: Rc ---");

    let config = Rc::new(String::from("modo=verbose;reintentos=3"));
    println!("refs iniciales: {}", Rc::strong_count(&config)); // 1

    // Rc::clone NO copia el String: solo incrementa el contador.
    // (Se escribe Rc::clone(&x) y no x.clone() por convención: deja claro
    // en el código que es un incremento barato, no una copia profunda.)
    let lector_a = Rc::clone(&config);
    {
        let lector_b = Rc::clone(&config);
        println!("con dos lectores extra: {}", Rc::strong_count(&config)); // 3
        println!("lector_b lee: {lector_b}");
    } // lector_b muere → contador baja a 2

    println!("tras el scope: {}", Rc::strong_count(&config)); // 2
    println!("lector_a lee: {lector_a}");
} // contador llega a 0 → RECIÉN ahí se libera el String. Como shared_ptr.

// ----------------------------------------------------------------------------
// EJEMPLO 4: RefCell<T> — borrow checking movido a RUNTIME
// ----------------------------------------------------------------------------
// Las reglas de préstamo de la fase 1 (muchos &T XOR un &mut T) se verifican
// en compile time. Pero a veces VOS sabés que un patrón es seguro y el
// compilador no puede probarlo (mutación detrás de una referencia
// compartida: cachés, contadores internos, memoización).
//
// RefCell dice: "verificá las MISMAS reglas, pero en runtime".
//   .borrow()     → como &T    (panic si hay un borrow_mut activo)
//   .borrow_mut() → como &mut T (panic si hay CUALQUIER otro borrow activo)
//
// No hay equivalente en C++: allá la mutabilidad interior es `mutable` +
// disciplina. Acá las violaciones se detectan SIEMPRE (panic determinista),
// no son UB que aparece cada tanto.
// ----------------------------------------------------------------------------
struct Termometro {
    // Mutabilidad interior: leer() toma &self (¡inmutable!) pero necesita
    // registrar cuántas veces se leyó. RefCell habilita exactamente eso.
    lecturas: RefCell<u32>,
    celsius: f64,
}

impl Termometro {
    fn leer(&self) -> f64 {
        // Mutamos a través de &self — con Mutex sería lock(); acá es
        // borrow_mut(), sin costo de sincronización (single-thread).
        *self.lecturas.borrow_mut() += 1;
        self.celsius
    }
}

fn ejemplo_4_refcell() {
    println!("--- Ejemplo 4: RefCell ---");

    let t = Termometro { lecturas: RefCell::new(0), celsius: 23.5 };
    t.leer();
    t.leer();
    t.leer();
    println!("temperatura: {} °C, leída {} veces", t.celsius, t.lecturas.borrow());

    // La violación clásica (dos borrow_mut a la vez) PANIQUEA en runtime:
    //
    //   let celda = RefCell::new(5);
    //   let a = celda.borrow_mut();
    //   let b = celda.borrow_mut(); // panic: already mutably borrowed
    //
    // Trade-off honesto: cambiaste un error de compilación por uno de
    // runtime. Por eso RefCell se usa POCO y encapsulado — no es una
    // salida de emergencia del borrow checker.
}

// ----------------------------------------------------------------------------
// EJEMPLO 5: Rc<RefCell<T>> — compartido Y mutable (el combo)
// ----------------------------------------------------------------------------
// Rc da la propiedad compartida, RefCell da la mutabilidad. Juntos:
// varios dueños que pueden mutar el mismo dato, en un solo thread.
// Es el equivalente semántico de shared_ptr<T> no-const en C++... pero
// con las reglas de aliasing verificadas en runtime en vez de "confiamos".
//
// Su gemelo multi-thread es Arc<Mutex<T>> (proyecto concurrencia/):
//   Rc  : RefCell  ::  Arc : Mutex
// ----------------------------------------------------------------------------
fn ejemplo_5_rc_refcell() {
    println!("--- Ejemplo 5: Rc<RefCell> ---");

    // Un log compartido entre varios "módulos" de una app single-thread.
    let log: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));

    // Dos módulos comparten la propiedad del MISMO Vec:
    let modulo_red = Rc::clone(&log);
    let modulo_disco = Rc::clone(&log);

    modulo_red.borrow_mut().push(String::from("red: conectado"));
    modulo_disco.borrow_mut().push(String::from("disco: 4 archivos escritos"));
    log.borrow_mut().push(String::from("main: apagando"));

    println!("log final (visto desde cualquier handle):");
    for linea in log.borrow().iter() {
        println!("  {linea}");
    }

    // Nota final: si Rc<RefCell> aparece POR TODOS LADOS en tu diseño,
    // suele ser señal de que estás forzando un diseño de "objetos que se
    // apuntan entre sí" estilo OOP. En Rust casi siempre hay un dueño
    // claro esperando ser encontrado (o un índice/id en vez de puntero).
}
