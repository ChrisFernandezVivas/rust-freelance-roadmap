//! # Ejercicio 01 — Caché con mutabilidad interior (`RefCell`)
//!
//! **Enunciado**: implementá `Calculadora`, que expone `costo(n)` — una
//! función "cara" de calcular — con memoización transparente:
//!
//! - `costo(&self, n)` toma `&self` INMUTABLE (el caller no necesita saber
//!   que hay un caché adentro: eso es "mutabilidad interior").
//! - La primera vez que se pide un `n`, se calcula y se guarda.
//! - Las siguientes veces sale del caché sin recalcular.
//! - `calculos_reales()` reporta cuántas veces se computó de verdad.
//!
//! **Qué practica**: `RefCell` para mutar detrás de `&self`, y el
//! razonamiento de POR QUÉ es seguro acá (single-thread, borrows cortos
//! que nunca se solapan). En C++ esto sería un `mutable std::map` + la
//! esperanza de que nadie lo toque concurrentemente; acá si un borrow se
//! solapara, panic determinista en el acto.

use std::cell::RefCell;
use std::collections::HashMap;

pub struct Calculadora {
    // DOS celdas separadas a propósito: si metiéramos todo en una sola
    // struct dentro de un RefCell, cualquier lectura del contador
    // bloquearía el caché. Granularidad fina = menos conflictos.
    cache: RefCell<HashMap<u64, u64>>,
    calculos: RefCell<u32>,
}

impl Calculadora {
    pub fn new() -> Self {
        Calculadora {
            cache: RefCell::new(HashMap::new()),
            calculos: RefCell::new(0),
        }
    }

    /// La función "cara" (acá: suma de divisores, O(n)).
    /// Privada: el mundo exterior solo ve `costo`.
    fn computar(&self, n: u64) -> u64 {
        *self.calculos.borrow_mut() += 1;
        (1..=n).filter(|d| n.is_multiple_of(*d)).sum()
    }

    /// Versión memoizada. Fijate el detalle FINO de los borrows:
    /// consultamos el caché y SOLTAMOS el borrow antes de computar,
    /// porque computar() también pide borrows (del contador). Mantener
    /// un borrow del caché mientras computamos funcionaría acá... hasta
    /// que alguien haga computar() recursivo sobre el caché y PANIC.
    /// La disciplina "borrows cortos" es a RefCell lo que "secciones
    /// críticas cortas" es a Mutex.
    pub fn costo(&self, n: u64) -> u64 {
        // Borrow de lectura, corto y en su propio scope:
        if let Some(&v) = self.cache.borrow().get(&n) {
            return v;
        } // ← el borrow muere acá

        let v = self.computar(n);
        self.cache.borrow_mut().insert(n, v); // borrow de escritura, corto
        v
    }

    pub fn calculos_reales(&self) -> u32 {
        *self.calculos.borrow()
    }
}

impl Default for Calculadora {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calcula_bien() {
        let c = Calculadora::new();
        // divisores de 12: 1+2+3+4+6+12 = 28
        assert_eq!(c.costo(12), 28);
        // divisores de 7 (primo): 1+7 = 8
        assert_eq!(c.costo(7), 8);
    }

    #[test]
    fn cachea_de_verdad() {
        let c = Calculadora::new();
        c.costo(100);
        c.costo(100);
        c.costo(100);
        // Tres pedidos, UN solo cálculo real:
        assert_eq!(c.calculos_reales(), 1);
    }

    #[test]
    fn valores_distintos_se_calculan_cada_uno() {
        let c = Calculadora::new();
        c.costo(10);
        c.costo(20);
        c.costo(10); // cacheado
        c.costo(20); // cacheado
        assert_eq!(c.calculos_reales(), 2);
    }

    #[test]
    fn el_caller_solo_necesita_referencia_inmutable() {
        // Este test documenta el CONTRATO: costo() funciona con &self.
        // (Si compila, pasa — el assert es casi decorativo.)
        let c = Calculadora::new();
        let referencia: &Calculadora = &c; // ni mut, ni Box, ni nada
        assert_eq!(referencia.costo(6), 12); // 1+2+3+6
    }
}
