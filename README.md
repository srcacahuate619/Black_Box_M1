# 🩺 M1-BlackBox: Cortafuegos Clínico Inmutable para IMDs

[![Solana](https://img.shields.io/badge/Solana-Blockchain-black?logo=solana)](https://solana.com/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Standard: FDA 524B](https://img.shields.io/badge/Standard-FDA_524B-blue)](https://www.fda.gov/medical-devices)

**M1-BlackBox** es un protocolo de ciberseguridad descentralizado diseñado para Dispositivos Médicos Implantables (IMDs), específicamente bombas de insulina. Utiliza el framework Anchor sobre la blockchain de Solana para crear un registro inmutable y un sistema de validación de dosis en tiempo real que previene ataques de sobredosis y manipulación de datos.

## 🛡️ Características de Seguridad (FDA 524B Compliance)
Tras una auditoría técnica de ciberseguridad, el contrato inteligente fue actualizado para incluir los siguientes salvaguardas clínicos:

* **Dose Hard-Cap (Prevención de Sobredosis):** Validación estricta de dosis máximas permitidas basadas en la prescripción del paciente.
* **Temporal Rate Limiting:** Mecanismo que impide la administración de dosis en intervalos de tiempo menores a 5 minutos, bloqueando ataques de "spam" de insulina.
* **Emergency Pause (Kill-Switch):** Función de emergencia que permite al médico detener el dispositivo ante la detección de un comportamiento anómalo o ciberataque.
* **Role-Based Access Control (RBAC):** Requiere la firma criptográfica conjunta del Médico Tratante y el Paciente para la inicialización segura del expediente.

## 📊 Arquitectura Técnica
* **Lenguaje:** Rust (Anchor Framework)
* **Blockchain:** Solana (Devnet/Localhost)
* **Validación:** Suite de pruebas clínicas en TypeScript (`client.ts`) que simula telemetría IoT y respuesta a ataques.

## 🧪 Pruebas de Concepto (PoC)
El sistema ha sido validado mediante una suite de simulación que verifica:
1. Inicialización segura con multirirma.
2. Registro inmutable de signos vitales.
3. Activación efectiva de protocolos de emergencia.

> **Nota para Revisores:** Este proyecto ha sido desarrollado bajo un enfoque híbrido de Ingeniería de Software y Medicina, integrando normativas de la FDA y estándares ISO 14971 para la gestión de riesgos en dispositivos médicos.

## 🔗 Referencias Académicas
* Investigación publicada en Zenodo (CERN). https://zenodo.org/records/18905940
* Investigación publicada en Zenodo previa (CERN). https://zenodo.org/records/18668319
* Reportes de Auditoría Técnica 524B incluidos en el repositorio. 
