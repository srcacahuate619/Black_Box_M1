# Medical BlackBox: Immutable Web3 Ledger for IoT Healthcare

![Solana](https://img.shields.io/badge/Solana-362D59?style=for-the-badge&logo=solana&logoColor=white)
![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
![Anchor](https://img.shields.io/badge/Anchor-000000?style=for-the-badge&logo=anchor&logoColor=white)
![TypeScript](https://img.shields.io/badge/TypeScript-007ACC?style=for-the-badge&logo=typescript&logoColor=white)

Este repositorio contiene la infraestructura de Smart Contracts (Backend Web3) para un sistema de auditoría médica inmutable. 

Este código es la implementación práctica y arquitectónica de la investigación publicada en el *Whitepaper*: **"Arquitectura Híbrida Pasiva para el Manejo de la Diabetes: Escisión Enzimática de Proinsulina in situ mediada por Telemetría NFC"**.
 **Leer la publicación oficial (DOI):** [https://zenodo.org/records/18668319]

---

## 🧠 El Problema: El "Dilema del Oráculo Médico"
Cuando un dispositivo médico IoT (como una bomba de insulina o un páncreas artificial) falla, demostrar la responsabilidad legal es un desafío. Los historiales médicos tradicionales centralizados (Bases de Datos SQL, Excels) son susceptibles a manipulación post-incidente por parte de hospitales o fabricantes para evadir auditorías.

## 💡 La Solución Web3
**Medical BlackBox** utiliza la red de Solana para actuar como un notario público descentralizado. A través de este Smart Contract escrito en Rust (Anchor Framework), los signos vitales críticos (Glucosa e Insulina) se registran de forma inmutable, creando una línea de tiempo auditable y sellada criptográficamente.

### 🏗️ Arquitectura y Modelo de Negocio (B2B2C)
Este contrato no es un simple CRUD, está diseñado para resolver la fricción comercial de Web3 en el sector salud:
1. **Transacciones Patrocinadas (`payer = aseguradora`):** Los pacientes no necesitan comprar criptomonedas (SOL) ni pagar "Gas/Rent". La red hospitalaria o aseguradora absorbe el costo de inicialización del expediente.
2. **Propiedad Criptográfica del Paciente (`seeds`):** Aunque el hospital pague, la identidad de la "Caja Negra" se deriva matemáticamente (PDA - *Program Derived Address*) usando la Llave Pública del paciente. Ningún actor externo puede alterar el expediente sin la firma del dispositivo del paciente.
3. **Escudos Clínicos (Custom Errors):** El contrato previene inyecciones de datos corruptos mediante validaciones de seguridad (`require!`), bloqueando dosis letales irreales o datos de glucosa en cero antes de que toquen la blockchain.

---

## ⚙️ Estructura del Smart Contract

* **`InitializeRecord`**: Crea la cuenta PDA única del paciente.
* **`LogVitals`**: Registra de forma inmutable los niveles de Glucosa (`u16`), la dosis de Insulina (`u16`) y captura la hora exacta descentralizada de los validadores de Solana (`Clock::get()?.unix_timestamp`).
* **`MedicalRecord`**: Estructura de almacenamiento optimizada usando la macro `#[derive(InitSpace)]` para el cálculo automático de RAM.

---

## 🚀 Cómo simular este proyecto (Solana Playground)

Puedes interactuar con este Smart Contract directamente desde tu navegador sin instalar dependencias locales.

1. Abre [Solana Playground (beta.solpg.io)](https://beta.solpg.io/).
2. Copia el contenido de `src/lib.rs` en un nuevo proyecto de Anchor.
3. Haz clic en el ícono de herramientas (Build & Deploy) en la barra lateral izquierda.
4. Presiona **Build** y luego **Deploy** (SolPG te asignará Devnet SOL automáticamente).
5. Copia el script de simulación `client.ts` de este repositorio en la carpeta `client` de SolPG.
6. Ejecuta el comando `run` en la terminal para ver la "Bomba de Insulina" generar llaves al vuelo, registrar signos vitales y auditar la blockchain.

---

*Diseñado y programado por Johan Amezcua- Fusionando el pensamiento clínico con la ingeniería de software.*
