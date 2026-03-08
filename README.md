# Medical BlackBox: Immutable Web3 Ledger for IoT Healthcare 🛡️💉

[![Zenodo](https://img.shields.io/badge/DOI-10.5281%2Fzenodo.18905940-blue)](https://zenodo.org/records/18905940)
![Solana](https://img.shields.io/badge/Solana-Blockchain-black?logo=solana)
![Rust](https://img.shields.io/badge/Rust-Anchor-orange?logo=rust)

Este repositorio contiene la infraestructura de Smart Contracts (Backend Web3) para un ecosistema de auditoría médica inmutable diseñado para Dispositivos Médicos Implantables (IMDs). 

Este código es la implementación técnica de la investigación publicada en el Whitepaper:  
**"Arquitectura Descentralizada para la Telemetría Médica: M1-BlackBox como Registro Inmutable en Sistemas de Asa Cerrada y Bioseguridad."** 📖 **Leer la publicación oficial (DOI):** [https://zenodo.org/records/18905940](https://zenodo.org/records/18905940)
---

## 🔬 Evolución de la Investigación (Background)

Este proyecto es la culminación técnica de una línea de investigación continua sobre bioseguridad en dispositivos médicos:

* **Fase 1 (Conceptual):** [Arquitectura Híbrida Pasiva para el Manejo de la Diabetes](https://zenodo.org/records/18668319). En esta etapa se validó la factibilidad de la escisión de proinsulina mediante telemetría pasiva NFC (DOI: 10.5281/zenodo.18668319).
* **Fase 2 (Actual):** **M1-BlackBox**. Evolución hacia una infraestructura descentralizada en Solana para garantizar la inmutabilidad y cumplimiento regulatorio FDA del sistema completo.
---

## 🧠 El Problema: El "Dilema del Oráculo Médico"

Cuando un dispositivo médico crítico (como una bomba de insulina o un marcapasos) falla, demostrar la responsabilidad legal y técnica es un desafío. Los historiales médicos tradicionales centralizados son susceptibles a manipulación post-incidente o carecen de la granularidad necesaria para una auditoría forense en tiempo real. 

## 💡 La Solución Web3

**Medical BlackBox** utiliza la red de **Solana** para actuar como un notario público descentralizado con latencia subclínica (<400ms). A través de este Smart Contract escrito en **Rust (Anchor Framework)**, los signos vitales críticos y comandos telemétricos se registran de forma inmutable, creando una línea de tiempo auditable y sellada criptográficamente.

## 🏗️ Arquitectura y Modelo de Negocio (B2B2C)

Este contrato no es un simple CRUD; integra lógica de bioseguridad avanzada:

1.  **Transacciones Patrocinadas:** Diseñado para que la red hospitalaria o aseguradora absorba el costo de inicialización y operación, eliminando la fricción de entrada para el paciente.
2.  **Propiedad Criptográfica (PDAs):** El expediente médico se deriva matemáticamente (Program Derived Address) usando la Llave Pública del paciente. Ningún actor externo puede alterar el registro sin la firma del dispositivo autorizado.
3.  **Escudos Clínicos (Custom Errors):** El contrato previene inyecciones de datos corruptos mediante validaciones de seguridad (`require!`), bloqueando dosis letales irreales o lecturas de glucosa fuera de rango antes de que lleguen a la blockchain.

---

## ⚙️ Estructura del Smart Contract

* **`initialize_record`**: Crea la cuenta PDA única del paciente y establece la autoridad del dispositivo.
* **`log_vitals`**: Registra de forma inmutable los niveles de Glucosa (`u16`) e Insulina (`u16`), capturando la marca de tiempo exacta de los validadores de Solana (`Clock::get()?.unix_timestamp`).
* **`MedicalRecord`**: Estructura de almacenamiento optimizada usando la macro `#[derive(InitSpace)]` para el cálculo automático de RAM y eficiencia en costos de almacenamiento.

---

## 🚀 Cómo simular este proyecto (Solana Playground)

Puedes interactuar con este Smart Contract directamente desde tu navegador sin instalar dependencias locales:

1.  Abre [Solana Playground (beta.solpg.io)](https://beta.solpg.io).
2.  Copia el contenido de `src/lib.rs` en un nuevo proyecto de Anchor.
3.  Haz clic en el ícono de herramientas (**Build & Deploy**) y presiona "Deploy".
4.  Copia el script de simulación `client.ts` de este repositorio en la carpeta `client` de SolPG.
5.  Ejecuta el comando `run` en la terminal para ver la simulación de la "Bomba de Insulina" generando llaves, registrando signos y auditando la blockchain en tiempo real.

---
*Diseñado y programado por Johan Amezcua - Fusionando el pensamiento clínico con la ingeniería de software.*
