//! # Ejercicio 05 — Búsqueda sin copiar (lifetimes en serio)
//!
//! **Enunciado**: implementá un buscador de líneas que devuelve REFERENCIAS
//! al texto original (cero copias), no Strings nuevos:
//!
//! 1. `lineas_con`: todas las líneas que contienen un patrón → `Vec<&str>`.
//! 2. `linea_mas_larga`: la línea más larga entre DOS textos → `&str`
//!    (acá el lifetime se pone interesante: ¿a cuál texto apunta el retorno?).
//! 3. `Resaltado`: un struct que guarda los fragmentos antes/después de la
//!    primera ocurrencia del patrón — un struct CON lifetime.
//!
//! **Qué practica**: la diferencia `&str` vs `String` (prestar vs poseer),
//! anotaciones de lifetime en funciones con múltiples entradas, y structs
//! con lifetime. Es EXACTAMENTE lo que hace ripgrep para ser rápido:
//! devolver vistas al buffer, no copias.
//!
//! Analogía C++: devolver `std::string_view` a un buffer ajeno. La
//! diferencia es que en C++ nada impide que el buffer muera antes que la
//! vista (dangling view = UB silencioso); acá el compilador lo prohíbe.

/// Devuelve las líneas de `texto` que contienen `patron`.
///
/// El lifetime dice algo importante: el retorno está atado a `texto`
/// (de ahí salen los &str), pero NO a `patron` — por eso son lifetimes
/// SEPARADOS ('a y 'b). Si usáramos uno solo, el caller no podría hacer
/// esto (que es perfectamente válido):
///
/// ```
/// use ejercicios_fase_1::ej05_lifetimes_busqueda::lineas_con;
/// let texto = String::from("hola\nchau");
/// let resultado;
/// {
///     let patron = String::from("hola"); // patrón de vida corta
///     resultado = lineas_con(&texto, &patron);
/// } // patron muere acá...
/// assert_eq!(resultado, vec!["hola"]); // ...y el resultado sigue vivo ✓
/// ```
// clippy sugiere elidir 'b (tiene razón: `patron: &str` bastaría porque 'b
// no aparece en el retorno). Lo dejamos EXPLÍCITO a propósito con fines
// didácticos: queremos que se VEA que son dos lifetimes independientes.
#[allow(clippy::needless_lifetimes)]
pub fn lineas_con<'a, 'b>(texto: &'a str, patron: &'b str) -> Vec<&'a str> {
    texto
        .lines()
        .filter(|linea| linea.contains(patron))
        .collect()
}

/// La línea más larga entre dos textos.
///
/// Acá SÍ va un único lifetime: el retorno puede apuntar a CUALQUIERA de
/// los dos, así que debe vivir mientras vivan ambos. El compilador no
/// puede adivinarlo mirando la firma — por eso la anotación es obligatoria
/// (probá sacarla: error E0106, "missing lifetime specifier").
pub fn linea_mas_larga<'a>(texto_a: &'a str, texto_b: &'a str) -> &'a str {
    texto_a
        .lines()
        .chain(texto_b.lines()) // encadena ambos iteradores
        .max_by_key(|linea| linea.len())
        .unwrap_or("") // ambos textos vacíos → línea vacía
}

/// Vista de una línea partida alrededor de la primera ocurrencia del patrón.
/// Los tres campos son PRÉSTAMOS del texto original: este struct pesa
/// 3 fat-pointers y no aloca nada. El lifetime `'a` en el struct declara:
/// "un Resaltado no puede sobrevivir al texto del que nació".
#[derive(Debug, PartialEq)]
pub struct Resaltado<'a> {
    pub antes: &'a str,
    pub coincidencia: &'a str,
    pub despues: &'a str,
}

impl<'a> Resaltado<'a> {
    /// Busca la primera ocurrencia de `patron` en `linea`.
    pub fn buscar(linea: &'a str, patron: &str) -> Option<Resaltado<'a>> {
        // find devuelve el índice en BYTES. Los slices de &str validan que
        // cortes en límites UTF-8 (si no, panic) — en C cortarías un
        // carácter multibyte por la mitad sin enterarte.
        let inicio = linea.find(patron)?;
        let fin = inicio + patron.len();
        Some(Resaltado {
            antes: &linea[..inicio],
            coincidencia: &linea[inicio..fin],
            despues: &linea[fin..],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encuentra_lineas() {
        let log = "INFO: arranque\nERROR: voltaje fuera de rango\nINFO: fin\nERROR: timeout";
        let errores = lineas_con(log, "ERROR");
        assert_eq!(
            errores,
            vec!["ERROR: voltaje fuera de rango", "ERROR: timeout"]
        );
    }

    #[test]
    fn sin_coincidencias() {
        assert!(lineas_con("a\nb\nc", "z").is_empty());
    }

    #[test]
    fn las_referencias_apuntan_al_original() {
        let texto = String::from("una línea con dato");
        let resultado = lineas_con(&texto, "dato");
        // Verificamos que es la MISMA memoria (mismo puntero), no una copia:
        assert_eq!(resultado[0].as_ptr(), texto.as_ptr());
    }

    #[test]
    fn mas_larga_entre_dos_textos() {
        let a = "corta\nla línea más larga de todas";
        let b = "media\nchica";
        assert_eq!(linea_mas_larga(a, b), "la línea más larga de todas");
        // Y al revés (el retorno puede venir del segundo argumento):
        assert_eq!(linea_mas_larga(b, a), "la línea más larga de todas");
    }

    #[test]
    fn mas_larga_con_vacios() {
        assert_eq!(linea_mas_larga("", ""), "");
        assert_eq!(linea_mas_larga("solo", ""), "solo");
    }

    #[test]
    fn resaltado_parte_en_tres() {
        let r = Resaltado::buscar("voltaje vcore alto", "vcore").unwrap();
        assert_eq!(r.antes, "voltaje ");
        assert_eq!(r.coincidencia, "vcore");
        assert_eq!(r.despues, " alto");
    }

    #[test]
    fn resaltado_sin_match() {
        assert_eq!(Resaltado::buscar("nada por aquí", "vcore"), None);
    }

    #[test]
    fn resaltado_al_inicio_y_al_final() {
        let r = Resaltado::buscar("vcore", "vcore").unwrap();
        assert_eq!(r.antes, "");
        assert_eq!(r.despues, "");
    }
}
