# mini-blockchain — blockchain educativa

Implementación educativa de una blockchain minimalista: bloques encadenados
por hash SHA-256, proof-of-work con dificultad ajustable, y validación de
la cadena completa (incluida la detección de manipulación de bloques).

> **Alcance deliberado**: esto sirve para ENTENDER el mecanismo (hashing
> encadenado, PoW, por qué la cadena es inmutable en la práctica). El
> mercado freelance real de Web3 en Rust está en programas de **Solana con
> Anchor**, no en escribir blockchains desde cero — ver
> [`fase-4-solana-anchor/README.md`](../../fase-4-solana-anchor/README.md).

## Cómo correrlo

```bash
cargo run     # mina una cadena de 3 bloques (dificultad 4) y demuestra
              # en vivo cómo la manipulación de un bloque la invalida
cargo test    # 14 tests
```

Salida esperada de `cargo run` (los tiempos y nonces varían):

```
== Minando una cadena con dificultad 4 ==

bloque 1 minado en 270ms (nonce=52424): 00004a6db3e6...
...
cadena válida: true

== Manipulando el bloque 1 (sin re-minar) ==
cadena válida después de la manipulación: false
```

## Cómo funciona

1. **`Bloque`** ([`src/bloque.rs`](src/bloque.rs)): cada bloque guarda su
   `index`, `timestamp`, `data`, el `hash` del bloque anterior, su propio
   `hash`, y el `nonce` usado para minarlo.
2. **Hashing encadenado**: el hash de cada bloque se calcula sobre TODOS
   sus campos (`index + timestamp + data + hash_anterior + nonce`) con
   SHA-256. Cambiar cualquier campo cambia el hash por completo (efecto
   avalancha de las funciones criptográficas de hash).
3. **Proof-of-work**: `minar(dificultad)` incrementa el `nonce` en un loop
   de fuerza bruta hasta que el hash resultante empiece con N ceros
   hexadecimales. No hay atajo: la única forma de encontrar un nonce válido
   es probar — por diseño. Cada cero adicional multiplica por ~16 el
   trabajo esperado.
4. **`Cadena`** ([`src/cadena.rs`](src/cadena.rs)): una lista de bloques.
   `es_valida()` verifica DOS cosas por cada bloque: que su hash sea
   consistente con su contenido (PoW real, nadie lo alteró después de
   minarlo) y que enlace correctamente con el `hash` real del bloque
   anterior. Manipular un bloque intermedio sin re-minar TODA la cadena
   desde ahí en adelante invalida el resto — esa es la propiedad de
   inmutabilidad práctica que hace valiosa a una blockchain.

## Tests (14 en total)

- **Creación de bloques**: génesis correcto, hash determinístico, cambiar
  un campo cambia el hash.
- **Minado con distinta dificultad**: el hash resultante cumple la
  dificultad pedida; dificultades mayores exigen más ceros.
- **Detección de manipulación**: alterar `data` sin re-minar invalida el
  bloque; alterar y re-minar UN bloque igual invalida la cadena (rompe el
  enlace con el bloque siguiente).
- **Validación de cadena completa**: cadenas recién minadas son válidas;
  la cadena de un solo bloque (génesis) es válida; distintas dificultades
  producen cadenas válidas.

## Qué conceptos de Rust demuestra

- **`sha2` (RustCrypto)**: hashing criptográfico real, no una
  implementación de juguete.
- **Serde `derive`**: `Bloque` deriva `Serialize`/`Deserialize` — se
  puede persistir/transmitir como JSON sin escribir un parser a mano
  (ver el bloque génesis impreso como JSON al final de `cargo run`).
- **Invariantes documentadas**: `ultimo_bloque()` usa `.expect(...)` con
  un mensaje que explica POR QUÉ el `unwrap` es seguro (la cadena nunca
  está vacía) — la diferencia entre un `unwrap()` descuidado y una
  decisión consciente y documentada.
- **Mutación controlada para tests**: los tests de manipulación acceden
  directamente a `cadena.bloques[i].data` porque los campos son públicos
  — en un diseño de producción se restringiría el acceso mutable directo,
  pero acá sirve para simular un "ataque" de forma explícita y didáctica.

## Estructura

```
src/
  bloque.rs   # Bloque: hash, minar(), es_valido() + 6 tests
  cadena.rs   # Cadena: agregar_bloque(), es_valida() + 7 tests
  lib.rs      # re-exporta Bloque y Cadena
  main.rs     # demo ejecutable
```

Parte de la [fase 3 del roadmap](../README.md).
