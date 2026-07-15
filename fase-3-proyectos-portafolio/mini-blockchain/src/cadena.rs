//! La cadena de bloques completa: agregar bloques minados y validar la
//! cadena entera de punta a punta.

use crate::bloque::Bloque;

/// Una blockchain: una lista de bloques enlazados por hash, con una
/// dificultad de minado fija para todos los bloques (en una blockchain
/// real la dificultad se ajusta dinámicamente según el poder de cómputo
/// de la red — acá la simplificamos a una constante por cadena).
pub struct Cadena {
    pub bloques: Vec<Bloque>,
    pub dificultad: usize,
}

impl Cadena {
    /// Crea una cadena nueva con solo el bloque génesis.
    pub fn nueva(dificultad: usize) -> Cadena {
        Cadena {
            bloques: vec![Bloque::genesis()],
            dificultad,
        }
    }

    /// El último bloque de la cadena (siempre existe: nunca está vacía).
    pub fn ultimo_bloque(&self) -> &Bloque {
        // unwrap() es seguro acá: la invariante de este struct es "nunca
        // vacío" (nueva() siempre mete el génesis, y no exponemos ningún
        // método que pueda vaciarla). Documentar el POR QUÉ de un unwrap
        // es la diferencia entre descuido y una decisión consciente.
        self.bloques
            .last()
            .expect("la cadena siempre tiene al menos el bloque génesis")
    }

    /// Crea un bloque nuevo con `data`, lo MINA (proof-of-work) y lo agrega
    /// a la cadena, enlazado al último bloque existente.
    pub fn agregar_bloque(&mut self, data: String, timestamp: u64) {
        let anterior = self.ultimo_bloque();
        let mut nuevo = Bloque::nuevo(anterior.index + 1, timestamp, data, anterior.hash.clone());

        nuevo.minar(self.dificultad); // acá se "gasta" el trabajo computacional

        self.bloques.push(nuevo);
    }

    /// Valida la cadena COMPLETA: cada bloque debe ser internamente válido
    /// (hash consistente + cumple dificultad) Y debe enlazar correctamente
    /// con el bloque anterior (su `hash_anterior` debe coincidir con el
    /// `hash` real del bloque previo).
    ///
    /// Esta segunda condición es la que hace que manipular UN bloque
    /// intermedio invalide TODA la cadena desde ese punto en adelante: el
    /// bloque siguiente sigue apuntando al hash VIEJO (correcto), que ya
    /// no coincide con el hash NUEVO (recalculado) del bloque manipulado.
    pub fn es_valida(&self) -> bool {
        for i in 0..self.bloques.len() {
            let actual = &self.bloques[i];

            // 1) El bloque en sí debe ser válido (hash consistente + PoW).
            //    El génesis tiene dificultad 0 exigida (no se minó de verdad).
            let dificultad_exigida = if i == 0 { 0 } else { self.dificultad };
            if !actual.es_valido(dificultad_exigida) {
                return false;
            }

            // 2) (excepto el génesis) debe enlazar con el bloque anterior.
            if i > 0 {
                let previo = &self.bloques[i - 1];
                if actual.hash_anterior != previo.hash {
                    return false;
                }
            }
        }
        true
    }

    pub fn longitud(&self) -> usize {
        self.bloques.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cadena_nueva_tiene_solo_el_genesis() {
        let cadena = Cadena::nueva(2);
        assert_eq!(cadena.longitud(), 1);
        assert_eq!(cadena.bloques[0].index, 0);
    }

    #[test]
    fn agregar_bloques_encadena_los_hashes() {
        let mut cadena = Cadena::nueva(2);
        cadena.agregar_bloque("primera transacción".to_string(), 1000);
        cadena.agregar_bloque("segunda transacción".to_string(), 2000);

        assert_eq!(cadena.longitud(), 3);
        // El hash_anterior de cada bloque debe ser el hash real del previo:
        assert_eq!(cadena.bloques[1].hash_anterior, cadena.bloques[0].hash);
        assert_eq!(cadena.bloques[2].hash_anterior, cadena.bloques[1].hash);
    }

    #[test]
    fn cadena_recien_creada_es_valida() {
        let mut cadena = Cadena::nueva(2);
        cadena.agregar_bloque("a".to_string(), 1);
        cadena.agregar_bloque("b".to_string(), 2);
        cadena.agregar_bloque("c".to_string(), 3);
        assert!(cadena.es_valida());
    }

    #[test]
    fn manipular_un_bloque_intermedio_invalida_toda_la_cadena() {
        let mut cadena = Cadena::nueva(2);
        cadena.agregar_bloque("saldo: 100".to_string(), 1);
        cadena.agregar_bloque("saldo: 50".to_string(), 2);
        cadena.agregar_bloque("saldo: 200".to_string(), 3);
        assert!(cadena.es_valida());

        // Un atacante modifica el bloque 1 (índice 1) para "robar" saldo,
        // SIN re-minarlo (no tiene el poder de cómputo para re-minar TODA
        // la cadena en el tiempo que tardaría la red honesta en avanzar
        // — esa es, en esencia, la garantía de seguridad del PoW real).
        cadena.bloques[1].data = "saldo: 999999".to_string();

        assert!(!cadena.es_valida());
    }

    #[test]
    fn manipular_y_re_minar_sin_actualizar_el_siguiente_sigue_invalido() {
        // Variante más "sofisticada" de ataque: el atacante SÍ re-mina el
        // bloque modificado (para que sea internamente válido)...
        let mut cadena = Cadena::nueva(2);
        cadena.agregar_bloque("dato original".to_string(), 1);
        cadena.agregar_bloque("dato siguiente".to_string(), 2);

        cadena.bloques[1].data = "dato alterado".to_string();
        cadena.bloques[1].minar(cadena.dificultad); // re-mina el bloque 1

        // ...pero el bloque 2 sigue apuntando al hash VIEJO del bloque 1
        // (el que existía antes de la alteración): el enlace se rompe.
        assert!(!cadena.es_valida());
    }

    #[test]
    fn cadena_de_un_solo_bloque_genesis_es_valida() {
        let cadena = Cadena::nueva(3);
        assert!(cadena.es_valida());
    }

    #[test]
    fn distintas_dificultades_producen_cadenas_validas() {
        for dificultad in [1, 2, 3] {
            let mut cadena = Cadena::nueva(dificultad);
            cadena.agregar_bloque("x".to_string(), 1);
            assert!(cadena.es_valida(), "falló con dificultad {dificultad}");
            assert!(cadena.bloques[1].hash.starts_with(&"0".repeat(dificultad)));
        }
    }
}
