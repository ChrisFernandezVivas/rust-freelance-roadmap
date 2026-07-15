//! # Ejercicio 03 — Parser de configuración (Result + errores custom + `?`)
//!
//! **Enunciado**: implementá `parsear_config`, que lee un texto con formato
//! `clave = valor` (una por línea) y devuelve un struct `Config` validado.
//! Reglas:
//! - Líneas vacías o que empiezan con `#` se ignoran (comentarios).
//! - Claves requeridas: `puerto` (u16, > 0), `host` (no vacío),
//!   `reintentos` (u8).
//! - CADA error debe reportar la causa exacta y la línea donde ocurrió
//!   (¡nada de "config inválida" genérico! el usuario de tu CLI te lo
//!   agradece, y el cliente que te contrata también).
//!
//! **Qué practica**: enums de error con datos, `impl Display`,
//! `impl From` para que `?` convierta errores automáticamente, y la
//! diferencia entre errores recuperables (Result) y bugs (panic).

use std::fmt;

#[derive(Debug, PartialEq)]
pub struct Config {
    pub host: String,
    pub puerto: u16,
    pub reintentos: u8,
}

/// Un enum de error con DATOS en cada variante: el caller puede hacer
/// match por causa y el mensaje incluye contexto. Esto es lo que la crate
/// `thiserror` genera por vos con macros — acá lo escribimos a mano una
/// vez para saber exactamente qué automatiza.
#[derive(Debug, PartialEq)]
pub enum ErrorConfig {
    /// La línea no tiene el formato `clave = valor`.
    FormatoInvalido { linea: usize, contenido: String },
    /// La clave no es una de las conocidas.
    ClaveDesconocida { linea: usize, clave: String },
    /// El valor no se pudo convertir al tipo esperado.
    ValorInvalido {
        linea: usize,
        clave: String,
        valor: String,
    },
    /// Terminó el archivo y faltó una clave obligatoria.
    ClaveFaltante(&'static str),
}

impl fmt::Display for ErrorConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorConfig::FormatoInvalido { linea, contenido } => {
                write!(
                    f,
                    "línea {linea}: '{contenido}' no tiene formato clave = valor"
                )
            }
            ErrorConfig::ClaveDesconocida { linea, clave } => {
                write!(f, "línea {linea}: clave desconocida '{clave}'")
            }
            ErrorConfig::ValorInvalido {
                linea,
                clave,
                valor,
            } => {
                write!(f, "línea {linea}: '{valor}' no es válido para '{clave}'")
            }
            ErrorConfig::ClaveFaltante(clave) => {
                write!(f, "falta la clave obligatoria '{clave}'")
            }
        }
    }
}

// Con Debug + Display podemos integrarnos al ecosistema estándar de errores.
impl std::error::Error for ErrorConfig {}

/// Parsea el texto completo de una configuración.
pub fn parsear_config(texto: &str) -> Result<Config, ErrorConfig> {
    // Acumulamos en Option: None = "todavía no la vimos".
    // Al final exigimos que todas sean Some — así detectamos faltantes.
    let mut host: Option<String> = None;
    let mut puerto: Option<u16> = None;
    let mut reintentos: Option<u8> = None;

    // enumerate() nos da el índice para reportar líneas 1-based.
    for (i, linea) in texto.lines().enumerate() {
        let num_linea = i + 1;
        let linea = linea.trim();

        // Comentarios y líneas vacías: seguir de largo.
        if linea.is_empty() || linea.starts_with('#') {
            continue;
        }

        // split_once devuelve Option — lo convertimos a NUESTRO error con
        // ok_or_else + `?`. Este patrón (Option → Result con contexto)
        // aparece constantemente en código real.
        let (clave, valor) = linea
            .split_once('=')
            .ok_or_else(|| ErrorConfig::FormatoInvalido {
                linea: num_linea,
                contenido: linea.to_string(),
            })?;

        let clave = clave.trim();
        let valor = valor.trim();

        // Closure auxiliar: fabrica el error de valor inválido con todo el
        // contexto capturado. Evita repetir 4 líneas en cada rama.
        let err_valor = || ErrorConfig::ValorInvalido {
            linea: num_linea,
            clave: clave.to_string(),
            valor: valor.to_string(),
        };

        match clave {
            "host" => {
                if valor.is_empty() {
                    return Err(err_valor());
                }
                host = Some(valor.to_string());
            }
            "puerto" => {
                // parse::<u16>() ya rechaza negativos y > 65535 gratis:
                // elegir bien el TIPO elimina validaciones enteras.
                let p: u16 = valor.parse().map_err(|_| err_valor())?;
                if p == 0 {
                    return Err(err_valor()); // puerto 0 = "cualquiera", no lo aceptamos
                }
                puerto = Some(p);
            }
            "reintentos" => {
                reintentos = Some(valor.parse().map_err(|_| err_valor())?);
            }
            otra => {
                return Err(ErrorConfig::ClaveDesconocida {
                    linea: num_linea,
                    clave: otra.to_string(),
                });
            }
        }
    }

    // ok_or convierte Option<T> → Result<T, E>: la forma limpia de exigir
    // "esta clave tenía que aparecer".
    Ok(Config {
        host: host.ok_or(ErrorConfig::ClaveFaltante("host"))?,
        puerto: puerto.ok_or(ErrorConfig::ClaveFaltante("puerto"))?,
        reintentos: reintentos.ok_or(ErrorConfig::ClaveFaltante("reintentos"))?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_valida() {
        let texto = "\
# configuración del servidor
host = 127.0.0.1
puerto = 8080

reintentos = 3";
        let config = parsear_config(texto).unwrap();
        assert_eq!(
            config,
            Config {
                host: "127.0.0.1".into(),
                puerto: 8080,
                reintentos: 3
            }
        );
    }

    #[test]
    fn detecta_formato_invalido_con_linea() {
        let err = parsear_config("host = ok\nesto no es clave valor").unwrap_err();
        assert_eq!(
            err,
            ErrorConfig::FormatoInvalido {
                linea: 2,
                contenido: "esto no es clave valor".into()
            }
        );
    }

    #[test]
    fn detecta_clave_desconocida() {
        let err = parsear_config("timeout = 30").unwrap_err();
        assert!(matches!(
            err,
            ErrorConfig::ClaveDesconocida { linea: 1, .. }
        ));
    }

    #[test]
    fn puerto_no_numerico() {
        let err = parsear_config("puerto = ochenta").unwrap_err();
        assert!(matches!(err, ErrorConfig::ValorInvalido { .. }));
    }

    #[test]
    fn puerto_fuera_de_rango_u16() {
        // 70000 > 65535: el TIPO u16 lo rechaza, no hace falta un if.
        let err = parsear_config("puerto = 70000").unwrap_err();
        assert!(matches!(err, ErrorConfig::ValorInvalido { .. }));
    }

    #[test]
    fn clave_faltante() {
        let err = parsear_config("host = localhost\npuerto = 80").unwrap_err();
        assert_eq!(err, ErrorConfig::ClaveFaltante("reintentos"));
    }

    #[test]
    fn los_errores_se_muestran_bien() {
        let err = parsear_config("puerto = -5").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("línea 1"));
        assert!(msg.contains("-5"));
    }
}
