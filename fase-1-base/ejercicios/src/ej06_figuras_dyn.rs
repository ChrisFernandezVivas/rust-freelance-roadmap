//! # Ejercicio 06 — Reporte polimórfico (traits + dyn + despacho dinámico)
//!
//! **Enunciado**: modelá instrumentos de medición heterogéneos con un trait
//! `Instrumento`, guardalos MEZCLADOS en una sola colección, e implementá:
//!
//! 1. `costo_total`: suma el costo de calibración de todos.
//! 2. `mas_caro`: referencia al instrumento más caro (¡sin clonar!).
//! 3. `resumen`: un String con una línea por instrumento.
//!
//! **Qué practica**: `Box<dyn Trait>` (vtables, como `virtual` en C++ pero
//! opt-in), colecciones heterogéneas, y devolver `Option<&dyn Trait>` —
//! prestar un objeto detrás de una vtable.

/// El "contrato" que todo instrumento cumple.
/// En C++: clase base abstracta con métodos virtuales puros y un virtual
/// destructor (que SIEMPRE hay que acordarse de escribir). Acá el drop
/// correcto a través del Box<dyn> está garantizado por el lenguaje.
pub trait Instrumento {
    fn nombre(&self) -> &str;
    /// Costo de calibración anual en USD.
    fn costo_calibracion(&self) -> f64;
    /// Método default: los tipos pueden sobreescribirlo si quieren.
    fn descripcion(&self) -> String {
        format!(
            "{} (calibración: ${:.2}/año)",
            self.nombre(),
            self.costo_calibracion()
        )
    }
}

pub struct Osciloscopio {
    pub modelo: String,
    pub ancho_banda_ghz: f64,
}

pub struct FuenteAlimentacion {
    pub modelo: String,
    pub canales: u8,
}

pub struct CamaraTermica {
    pub modelo: String,
}

impl Instrumento for Osciloscopio {
    fn nombre(&self) -> &str {
        &self.modelo
    }
    fn costo_calibracion(&self) -> f64 {
        // Los osciloscopios de más ancho de banda cuestan más de calibrar.
        800.0 + self.ancho_banda_ghz * 150.0
    }
    fn descripcion(&self) -> String {
        format!(
            "Osciloscopio {} ({} GHz)",
            self.modelo, self.ancho_banda_ghz
        )
    }
}

impl Instrumento for FuenteAlimentacion {
    fn nombre(&self) -> &str {
        &self.modelo
    }
    fn costo_calibracion(&self) -> f64 {
        120.0 * f64::from(self.canales)
    }
}

impl Instrumento for CamaraTermica {
    fn nombre(&self) -> &str {
        &self.modelo
    }
    fn costo_calibracion(&self) -> f64 {
        450.0
    }
}

/// Suma el costo de todo el banco de instrumentos.
/// `&[Box<dyn Instrumento>]` = slice prestado de punteros con vtable.
/// Cada llamada a costo_calibracion() es un salto indirecto — el costo de
/// la flexibilidad. Si TODOS fueran del mismo tipo usaríamos genéricos y
/// el compilador inlinearía todo (despacho estático).
pub fn costo_total(banco: &[Box<dyn Instrumento>]) -> f64 {
    banco.iter().map(|i| i.costo_calibracion()).sum()
}

/// El instrumento más caro, PRESTADO. Fijate el tipo de retorno:
/// `Option<&dyn Instrumento>` — una referencia al contenido del Box,
/// sin mover ni clonar nada. `as_ref()` convierte &Box<dyn T> → &dyn T.
pub fn mas_caro(banco: &[Box<dyn Instrumento>]) -> Option<&dyn Instrumento> {
    banco
        .iter()
        // max_by con total_cmp porque f64 no es Ord (ver ej04).
        .max_by(|a, b| a.costo_calibracion().total_cmp(&b.costo_calibracion()))
        .map(|caja| caja.as_ref())
}

/// Una línea por instrumento, usando descripcion() (default u override).
pub fn resumen(banco: &[Box<dyn Instrumento>]) -> String {
    banco
        .iter()
        .map(|i| i.descripcion())
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: arma un banco heterogéneo. Devuelve Vec de Box<dyn ...>:
    /// tres TIPOS CONCRETOS distintos en la misma colección.
    fn banco_de_prueba() -> Vec<Box<dyn Instrumento>> {
        vec![
            Box::new(Osciloscopio {
                modelo: "MSO64".into(),
                ancho_banda_ghz: 8.0,
            }),
            Box::new(FuenteAlimentacion {
                modelo: "N6705C".into(),
                canales: 4,
            }),
            Box::new(CamaraTermica {
                modelo: "FLIR-A70".into(),
            }),
        ]
    }

    #[test]
    fn suma_costos_heterogeneos() {
        let banco = banco_de_prueba();
        // 800 + 8*150 = 2000; 120*4 = 480; 450 → total 2930
        assert!((costo_total(&banco) - 2930.0).abs() < 1e-9);
    }

    #[test]
    fn encuentra_el_mas_caro() {
        let banco = banco_de_prueba();
        let caro = mas_caro(&banco).expect("banco no vacío");
        assert_eq!(caro.nombre(), "MSO64");
    }

    #[test]
    fn banco_vacio() {
        let banco: Vec<Box<dyn Instrumento>> = Vec::new();
        assert_eq!(costo_total(&banco), 0.0);
        assert!(mas_caro(&banco).is_none());
    }

    #[test]
    fn resumen_usa_default_y_override() {
        let banco = banco_de_prueba();
        let r = resumen(&banco);
        // El osciloscopio sobreescribe descripcion():
        assert!(r.contains("Osciloscopio MSO64 (8 GHz)"));
        // La fuente usa el default del trait:
        assert!(r.contains("N6705C (calibración: $480.00/año)"));
        assert_eq!(r.lines().count(), 3);
    }
}
