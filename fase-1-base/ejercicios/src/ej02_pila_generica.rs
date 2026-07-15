//! # Ejercicio 02 — Pila genérica con mínimo en O(1) (genéricos + trait bounds)
//!
//! **Enunciado**: implementá `PilaMin<T>`, una pila (stack LIFO) que además
//! de `push`/`pop` puede responder **cuál es el mínimo actual en O(1)**,
//! para cualquier tipo `T` que se pueda ordenar y clonar.
//!
//! Técnica clásica de entrevista: mantener una segunda pila interna con
//! "el mínimo hasta este punto".
//!
//! **Qué practica**:
//! - Genéricos con bounds (`T: Ord + Clone`) — el equivalente verificado
//!   de un template C++ con `requires std::totally_ordered<T>`.
//! - `Option` como contrato de "puede estar vacía" (nada de sentinelas
//!   tipo -1 o NULL como en C).
//! - Ownership: `pop` DEVUELVE el valor (transfiere propiedad al caller),
//!   `minimo` solo lo PRESTA (&T).

/// Pila LIFO con consulta de mínimo en O(1).
///
/// Los bounds van en el `impl` y no en el `struct`: así el struct puede
/// existir para cualquier T, y solo los MÉTODOS que necesitan comparar
/// exigen `Ord`. Es una buena costumbre (bounds mínimos, donde se usan).
#[derive(Debug, Default)]
pub struct PilaMin<T> {
    datos: Vec<T>,
    // Pila paralela: minimos[i] = el mínimo de datos[0..=i].
    // Cuesta memoria extra a cambio de minimo() en O(1) — el clásico
    // trade-off espacio/tiempo.
    minimos: Vec<T>,
}

impl<T: Ord + Clone> PilaMin<T> {
    pub fn new() -> Self {
        PilaMin {
            datos: Vec::new(),
            minimos: Vec::new(),
        }
    }

    /// Apila un valor. `push` toma ownership de `valor`: el caller se lo
    /// entrega a la pila (en C++ sería push_back(std::move(v))).
    pub fn push(&mut self, valor: T) {
        // El nuevo mínimo es el menor entre el valor entrante y el mínimo
        // anterior. Clonamos porque `minimos` necesita su PROPIA copia:
        // no puede guardar una referencia a `datos` (self-referential
        // structs no existen en Rust seguro — ¡y por buenas razones!).
        let nuevo_min = match self.minimos.last() {
            Some(min_actual) if min_actual < &valor => min_actual.clone(),
            _ => valor.clone(),
        };
        self.minimos.push(nuevo_min);
        self.datos.push(valor);
    }

    /// Desapila y DEVUELVE el valor (ownership al caller).
    /// `None` si la pila está vacía — sin sentinelas mágicos.
    pub fn pop(&mut self) -> Option<T> {
        self.minimos.pop(); // las dos pilas siempre están sincronizadas
        self.datos.pop()
    }

    /// El mínimo actual, PRESTADO (&T): mirarlo no lo saca de la pila.
    pub fn minimo(&self) -> Option<&T> {
        self.minimos.last()
    }

    /// El tope actual, prestado.
    pub fn tope(&self) -> Option<&T> {
        self.datos.last()
    }

    pub fn len(&self) -> usize {
        self.datos.len()
    }

    pub fn is_empty(&self) -> bool {
        self.datos.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_pop_lifo() {
        let mut p = PilaMin::new();
        p.push(1);
        p.push(2);
        p.push(3);
        assert_eq!(p.pop(), Some(3));
        assert_eq!(p.pop(), Some(2));
        assert_eq!(p.pop(), Some(1));
        assert_eq!(p.pop(), None);
    }

    #[test]
    fn minimo_se_actualiza_al_apilar() {
        let mut p = PilaMin::new();
        p.push(5);
        assert_eq!(p.minimo(), Some(&5));
        p.push(3);
        assert_eq!(p.minimo(), Some(&3));
        p.push(7); // 7 no cambia el mínimo
        assert_eq!(p.minimo(), Some(&3));
    }

    #[test]
    fn minimo_se_restaura_al_desapilar() {
        let mut p = PilaMin::new();
        p.push(5);
        p.push(1); // mínimo pasa a 1
        p.push(9);
        assert_eq!(p.minimo(), Some(&1));
        p.pop(); // sale 9, mínimo sigue 1
        assert_eq!(p.minimo(), Some(&1));
        p.pop(); // sale 1, mínimo VUELVE a 5 — acá fallan las
                 // implementaciones ingenuas que guardan un solo mínimo
        assert_eq!(p.minimo(), Some(&5));
    }

    #[test]
    fn funciona_con_strings() {
        // Genéricos de verdad: el MISMO código sirve para String.
        let mut p = PilaMin::new();
        p.push(String::from("pera"));
        p.push(String::from("banana"));
        p.push(String::from("kiwi"));
        assert_eq!(p.minimo().map(|s| s.as_str()), Some("banana"));
    }

    #[test]
    fn vacia() {
        let p: PilaMin<i32> = PilaMin::new();
        assert!(p.is_empty());
        assert_eq!(p.minimo(), None);
        assert_eq!(p.tope(), None);
    }
}
