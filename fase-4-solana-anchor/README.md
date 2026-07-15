# Fase 4 — Solana / Anchor (Web3)

> ⛔ **Prerequisito estricto: haber completado la fase 3.**
> Esta fase es SOLO guía y recursos — el código llegará cuando toque.
> Ver la "regla de oro" en el [README principal](../README.md).

---

## 🧭 Una aclaración importante antes de empezar

**"Crear una crypto/blockchain desde cero" es un ejercicio educativo** — por eso
existe la [`mini-blockchain/`](../fase-3-proyectos-portafolio/mini-blockchain/) de la
fase 3: sirve para entender hashing, proof-of-work y por qué una cadena es
inmutable en la práctica.

Pero **el mercado freelance real de Web3 en Rust no está en escribir blockchains
desde cero**. Está en escribir **programas de Solana con el framework Anchor**:
tokens, NFTs, staking, vaults, integraciones DeFi, bots. Nadie te va a pagar por
reimplementar Bitcoin; te van a pagar por escribir y auditar programas on-chain
que mueven fondos de verdad.

Y de ahí la advertencia que se repite en todo este repo: en Solana un bug no es un
`500 Internal Server Error` — es **plata real de terceros** que se pierde y no se
recupera. Por eso esta fase va al final.

---

## 🛣️ Ruta de aprendizaje

### 1. Solana Playground — empezar sin instalar nada

<https://beta.solpg.io>

IDE completo en el navegador: editor, compilador, wallet de devnet y deploy con un
click. Ideal para los primeros programas ("hello world" on-chain, un contador, un
programa que guarda estado en una cuenta) sin pelearse con la instalación local de
`solana-cli` + `anchor-cli` (que tiene sus mañas).

### 2. Anchor Book — el framework

<https://book.anchor-lang.com>

Anchor es a Solana lo que Axum es a HTTP: el framework estándar de facto. El Anchor
Book cubre:

- El modelo de **cuentas** de Solana (todo es una cuenta: programas, datos, wallets).
- Macros de Anchor: `#[program]`, `#[derive(Accounts)]`, `#[account]`.
- Validación de cuentas y **constraints** (la parte de seguridad más importante).
- CPI (Cross-Program Invocation): llamar a otros programas.
- Tests con TypeScript y con Rust.

Acá es donde la fase 1-2 paga: las macros de Anchor generan código lleno de
lifetimes, traits y genéricos. Si no los dominás, los errores del compilador van a
ser jeroglíficos.

### 3. Cursos oficiales de Solana

<https://solana.com/developers/courses>

Cursos gratuitos y estructurados de la Solana Foundation: desde fundamentos hasta
programas nativos, tokens (SPL), NFTs y seguridad. Buen complemento guiado del
Anchor Book.

### 4. Repo de referencia: program-examples

<https://github.com/solana-developers/program-examples>

Patrones reales de programas, cada uno implementado en Anchor (y muchos también en
Rust nativo, útil para entender qué abstrae Anchor): cuentas, PDAs, tokens,
transferencias de SOL, realloc, etc. Es el equivalente de la carpeta `examples/`
de axum, pero para Solana.

### 5. RareSkills — entender el fondo, no solo copiar

<https://www.rareskills.io>

El tutorial de Solana de RareSkills está pensado para gente que ya programa y
quiere entender **por qué** (modelo de cuentas vs. modelo de contratos de Ethereum,
rent, compute units, ataques comunes). Es la diferencia entre "me funciona el
tutorial" y "puedo cobrar por esto".

---

## ✅ Checklist de la fase (cuando llegue el momento)

- [ ] Desplegar un programa "hello world" y un contador en devnet desde Solana Playground.
- [ ] Leer el Anchor Book completo, con foco en constraints y validación de cuentas.
- [ ] Completar al menos un curso de solana.com/developers/courses.
- [ ] Reproducir 5+ ejemplos de `program-examples` **sin copiar y pegar**: leer, cerrar, reescribir.
- [ ] Hacer el tutorial de Solana de RareSkills.
- [ ] Instalar el toolchain local (`solana-cli` + `anchor-cli`) y migrar el workflow fuera del Playground.
- [ ] Proyecto propio: un programa pequeño pero completo (ej. vault con depósito/retiro) con tests.

## 🔗 Recursos

Todos los links de esta fase están también centralizados en [`recursos/`](../recursos/README.md).
