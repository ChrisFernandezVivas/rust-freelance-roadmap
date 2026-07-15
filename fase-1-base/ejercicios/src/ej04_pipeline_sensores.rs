//! # Ejercicio 04 — Pipeline de sensores (iteradores + closures)
//!
//! **Enunciado**: dado un log de lecturas crudas (`"nombre:valor"` separadas
//! por `;`), implementá con UNA sola cadena de iteradores (sin `for`, sin
//! `Vec` intermedios más que el `collect` final):
//!
//! 1. `lecturas_validas`: parsea y descarta entradas corruptas.
//! 2. `promedio_movil`: media móvil de ventana N sobre una serie
//!    (usando `windows()`).
//! 3. `top_n`: los N nombres con lectura más alta, ordenados descendente.
//!
//! **Qué practica**: `filter_map`, `windows`, `sort_by`, closures que
//! capturan parámetros, y pensar en pipelines en vez de loops con estado
//! mutable — el cambio de mentalidad C → Rust idiomático.

/// Parsea un log tipo `"vcore:1.05;vddq:abc;temp:68.5"` descartando lo corrupto.
pub fn lecturas_validas(log: &str) -> Vec<(String, f64)> {
    log.split(';')
        // filter_map = filter + map en un paso: el closure devuelve
        // Option, y los None desaparecen del stream. Es el manejo de
        // errores "por elemento": una entrada corrupta no aborta el resto
        // (comparalo con una excepción de Python cortando todo el loop).
        .filter_map(|entrada| {
            let (nombre, valor) = entrada.split_once(':')?;
            let valor: f64 = valor.trim().parse().ok()?;
            // Rechazamos NaN/inf: parse() los acepta ("NaN" parsea!) pero
            // para un sensor físico son basura.
            if !valor.is_finite() {
                return None;
            }
            Some((nombre.trim().to_string(), valor))
        })
        .collect()
}

/// Media móvil de ventana `n`. Devuelve un Vec con len() = serie.len() - n + 1.
/// Si la serie es más corta que la ventana, devuelve vacío.
pub fn promedio_movil(serie: &[f64], n: usize) -> Vec<f64> {
    if n == 0 {
        return Vec::new(); // ventana 0 no tiene sentido; sin panic.
    }
    serie
        // windows(n) itera slices solapados de largo n: [a,b,c], [b,c,d]...
        // Son PRÉSTAMOS (&[f64]) del buffer original: cero copias.
        // En C esto serían aritmética de punteros y rezos por no pasarte
        // del final; acá los límites los garantiza el iterador.
        .windows(n)
        .map(|ventana| ventana.iter().sum::<f64>() / n as f64)
        .collect()
}

/// Los `n` sensores con lectura más alta, de mayor a menor.
pub fn top_n(lecturas: &[(String, f64)], n: usize) -> Vec<(String, f64)> {
    let mut ordenadas: Vec<(String, f64)> = lecturas.to_vec();
    // f64 NO implementa Ord (por culpa de NaN: NaN != NaN rompe el orden
    // total). Por eso sort() directo no compila y usamos sort_by con
    // total_cmp — Rust te hace explícito un problema que en C/C++ con
    // qsort/std::sort queda silenciosamente indefinido con NaNs.
    ordenadas.sort_by(|a, b| b.1.total_cmp(&a.1));
    ordenadas.truncate(n);
    ordenadas
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descarta_corruptas() {
        let v = lecturas_validas("vcore:1.05;vddq:abc;;temp:68.5;sinvalor");
        assert_eq!(
            v,
            vec![("vcore".to_string(), 1.05), ("temp".to_string(), 68.5)]
        );
    }

    #[test]
    fn rechaza_nan_e_infinito() {
        let v = lecturas_validas("a:NaN;b:inf;c:1.0");
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].0, "c");
    }

    #[test]
    fn media_movil_ventana_3() {
        let serie = [1.0, 2.0, 3.0, 4.0, 5.0];
        let m = promedio_movil(&serie, 3);
        assert_eq!(m, vec![2.0, 3.0, 4.0]);
    }

    #[test]
    fn media_movil_casos_borde() {
        assert!(promedio_movil(&[1.0, 2.0], 3).is_empty()); // serie < ventana
        assert!(promedio_movil(&[1.0, 2.0], 0).is_empty()); // ventana 0
        assert_eq!(promedio_movil(&[7.0], 1), vec![7.0]); // ventana 1 = identidad
    }

    #[test]
    fn top_2() {
        let lecturas = vec![
            ("temp0".to_string(), 68.5),
            ("temp1".to_string(), 91.2),
            ("temp2".to_string(), 45.0),
            ("temp3".to_string(), 71.3),
        ];
        let top = top_n(&lecturas, 2);
        assert_eq!(top[0].0, "temp1");
        assert_eq!(top[1].0, "temp3");
        assert_eq!(top.len(), 2);
    }

    #[test]
    fn top_n_mayor_que_lista() {
        let lecturas = vec![("a".to_string(), 1.0)];
        assert_eq!(top_n(&lecturas, 10).len(), 1); // truncate no explota
    }

    #[test]
    fn pipeline_completo() {
        // Integración de las tres funciones, como en un caso real.
        let log = "t:10.0;t:20.0;x:mal;t:30.0;t:40.0";
        let lecturas = lecturas_validas(log);
        let valores: Vec<f64> = lecturas.iter().map(|(_, v)| *v).collect();
        assert_eq!(promedio_movil(&valores, 2), vec![15.0, 25.0, 35.0]);
    }
}
