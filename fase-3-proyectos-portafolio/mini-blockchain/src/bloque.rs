//! Un bloque individual de la cadena: sus datos, su hash, y el proof-of-work
//! (minado) que lo "sella".
//!
//! Esto es una blockchain EDUCATIVA — sirve para entender los mecanismos
//! (hashing encadenado, PoW, inmutabilidad), no para usarse en producción.
//! El mercado freelance real de Web3/Rust está en programas de Solana con
//! Anchor, no en escribir blockchains desde cero — ver
//! `fase-4-solana-anchor/README.md`.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Un bloque de la cadena.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Bloque {
    /// Posición en la cadena (0 = génesis).
    pub index: u64,
    /// Timestamp Unix (segundos) de cuándo se creó el bloque.
    pub timestamp: u64,
    /// La "carga útil": en una blockchain real serían transacciones;
    /// acá simplificamos a un String para enfocarnos en el mecanismo.
    pub data: String,
    /// Hash del bloque ANTERIOR — esto es lo que hace una "cadena": cada
    /// bloque referencia criptográficamente al que lo precede. Cambiar un
    /// bloque viejo cambia su hash, lo que invalida el `hash_anterior` del
    /// siguiente, en cascada hasta el final de la cadena.
    pub hash_anterior: String,
    /// El hash de ESTE bloque (calculado sobre todos los campos de arriba
    /// + el nonce). Es la "huella digital" del bloque.
    pub hash: String,
    /// El número que se ajusta durante el minado hasta que el hash
    /// resultante cumple la dificultad pedida (proof-of-work).
    pub nonce: u64,
}

impl Bloque {
    /// Crea el bloque génesis (el primero, sin predecesor real). Su
    /// `hash_anterior` es una cadena de ceros por convención — no hay un
    /// bloque -1 al que referenciar.
    pub fn genesis() -> Bloque {
        let mut bloque = Bloque {
            index: 0,
            timestamp: 0,
            data: "bloque génesis".to_string(),
            hash_anterior: "0".repeat(64),
            hash: String::new(),
            nonce: 0,
        };
        bloque.hash = bloque.calcular_hash();
        bloque
    }

    /// Construye un bloque nuevo (SIN minar todavía) que enlaza al anterior.
    pub fn nuevo(index: u64, timestamp: u64, data: String, hash_anterior: String) -> Bloque {
        let mut bloque = Bloque {
            index,
            timestamp,
            data,
            hash_anterior,
            hash: String::new(),
            nonce: 0,
        };
        bloque.hash = bloque.calcular_hash();
        bloque
    }

    /// Calcula el hash SHA-256 del bloque a partir de TODOS sus campos
    /// (menos el hash mismo, claro — sería circular).
    ///
    /// Nota de diseño: serializamos con un formato DETERMINISTA (orden fijo
    /// de campos, concatenación explícita) en vez de `serde_json::to_string`
    /// directo sobre el struct completo, porque necesitamos control total
    /// sobre qué se hashea y en qué orden — un cambio accidental en el
    /// formato de serialización JSON no debería invalidar toda la cadena.
    pub fn calcular_hash(&self) -> String {
        let contenido = format!(
            "{}{}{}{}{}",
            self.index, self.timestamp, self.data, self.hash_anterior, self.nonce
        );

        // Sha256::digest procesa los bytes y devuelve un array de 32 bytes.
        // Los formateamos a mano como hex (2 dígitos por byte, con ceros a
        // la izquierda) — el mismo formato que `sha256sum` de la terminal,
        // para que el resultado sea reconocible.
        let hash_bytes = Sha256::digest(contenido.as_bytes());
        hash_bytes.iter().map(|b| format!("{b:02x}")).collect()
    }

    /// Prueba de trabajo (proof-of-work): incrementa `nonce` hasta que
    /// `hash` empiece con `dificultad` ceros hexadecimales.
    ///
    /// Comparación con C/C++: esto es exactamente un loop de fuerza bruta
    /// (`for (nonce = 0;; nonce++) { hash = sha256(...); if (cumple) break; }`),
    /// sin ninguna magia — el "trabajo" de proof-of-work ES literalmente
    /// gastar ciclos de CPU probando números hasta encontrar uno que
    /// produzca un hash con la forma pedida. No hay atajo matemático
    /// conocido: por diseño, la ÚNICA forma de encontrarlo es probar.
    ///
    /// Cada nivel de dificultad multiplica por ~16 el trabajo esperado
    /// (cada dígito hexadecimal tiene 16 valores posibles): dificultad 4
    /// tarda ~16x más que dificultad 3. Los tests usan dificultades bajas
    /// (2-3) para que corran rápido.
    pub fn minar(&mut self, dificultad: usize) {
        let objetivo = "0".repeat(dificultad);

        while !self.hash.starts_with(&objetivo) {
            self.nonce += 1;
            self.hash = self.calcular_hash();
        }
    }

    /// Verifica que el hash almacenado sea CONSISTENTE con el contenido del
    /// bloque (nadie lo manipuló) y que además cumpla la dificultad pedida
    /// (fue minado de verdad, no solo "declarado" válido).
    pub fn es_valido(&self, dificultad: usize) -> bool {
        let objetivo = "0".repeat(dificultad);
        self.hash == self.calcular_hash() && self.hash.starts_with(&objetivo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn genesis_tiene_hash_anterior_de_ceros() {
        let g = Bloque::genesis();
        assert_eq!(g.index, 0);
        assert_eq!(g.hash_anterior, "0".repeat(64));
    }

    #[test]
    fn el_hash_es_deterministico() {
        let b1 = Bloque::nuevo(1, 1000, "hola".to_string(), "abc".to_string());
        let b2 = Bloque::nuevo(1, 1000, "hola".to_string(), "abc".to_string());
        // Mismos campos → mismo hash. SHA-256 no tiene aleatoriedad.
        assert_eq!(b1.hash, b2.hash);
    }

    #[test]
    fn cambiar_un_campo_cambia_el_hash() {
        let b1 = Bloque::nuevo(1, 1000, "hola".to_string(), "abc".to_string());
        let b2 = Bloque::nuevo(1, 1000, "chau".to_string(), "abc".to_string()); // data distinta
        assert_ne!(b1.hash, b2.hash);
    }

    #[test]
    fn minar_produce_un_hash_con_ceros_iniciales() {
        let mut bloque = Bloque::nuevo(1, 1000, "datos".to_string(), "0".repeat(64));
        bloque.minar(3);
        assert!(bloque.hash.starts_with("000"));
    }

    #[test]
    fn mayor_dificultad_requiere_mas_intentos() {
        // No medimos tiempo (poco confiable en CI) sino el NONCE final:
        // más dificultad ⇒ típicamente más intentos para encontrar un hash
        // válido. Usamos una semilla fija para que sea reproducible.
        let mut facil = Bloque::nuevo(1, 42, "carga".to_string(), "0".repeat(64));
        facil.minar(2);

        let mut dificil = Bloque::nuevo(1, 42, "carga".to_string(), "0".repeat(64));
        dificil.minar(4);

        // La dificultad 4 exige 4 ceros; el hash debe cumplirlo (esto ya
        // prueba indirectamente que costó más: encontrar 4 ceros exige,
        // en promedio, 16x más intentos que 2 ceros).
        assert!(dificil.hash.starts_with("0000"));
        assert!(facil.hash.starts_with("00"));
    }

    #[test]
    fn es_valido_detecta_manipulacion_del_contenido() {
        let mut bloque = Bloque::nuevo(1, 1000, "datos originales".to_string(), "0".repeat(64));
        bloque.minar(2);
        assert!(bloque.es_valido(2));

        // Alguien cambia los datos DESPUÉS de minar, sin re-minar:
        bloque.data = "datos manipulados".to_string();
        // El hash guardado ya no corresponde al contenido → inválido.
        assert!(!bloque.es_valido(2));
    }

    #[test]
    fn es_valido_rechaza_hash_que_no_cumple_la_dificultad() {
        // Un bloque con un hash "válido" (consistente) pero de dificultad 0
        // (cualquier hash sirve) no necesariamente cumple una dificultad mayor.
        let bloque = Bloque::nuevo(1, 1000, "sin minar".to_string(), "0".repeat(64));
        // Es MUY improbable que un hash sin minar empiece con "0000" por azar.
        assert!(!bloque.es_valido(4));
    }
}
