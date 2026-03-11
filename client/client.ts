// =============================================================================
// SIMULACIÓN CLÍNICA: M1-BlackBox (Cumplimiento FDA 524B)
// Auditoría: v4 — Modelo de firmas IoT corregido (asa cerrada)
//
// MODELO DE FIRMAS:
//   initialize_record → doctor + paciente firman (consentimiento único)
//   log_vitals        → imd_device firma (operación autónoma 24/7)
//   emergency_pause   → doctor firma
// =============================================================================

console.log("Iniciando pruebas clínicas de M1-BlackBox v4...");

// =============================================================================
// 1️⃣ CREACIÓN DE IDENTIDADES
// NOTA (dev/simulación): En producción estas claves deben provenir de:
//   - patient/doctor: HSM certificado FIPS 140-2 Level 3
//   - imd_device:     Secure Element del hardware IMD (ej: ARM TrustZone, TPM)
//     La clave privada del IMD NUNCA abandona el hardware físico implantado.
// Para el entorno de Playground/devnet esta generación es aceptable.
// =============================================================================
const patient    = web3.Keypair.generate(); // Identidad del paciente
const doctor     = web3.Keypair.generate(); // Médico tratante autorizado por el hospital
const imdDevice  = web3.Keypair.generate(); // Simula el Secure Element del hardware IMD

// 2️⃣ DERIVACIÓN DE LA PDA (El Expediente Inmutable del paciente)
const [medicalRecordPda] = web3.PublicKey.findProgramAddressSync(
  [Buffer.from("blackbox"), patient.publicKey.toBuffer()],
  pg.program.programId
);

console.log("📄 PDA del Expediente derivado:", medicalRecordPda.toBase58());

// =============================================================================
// PRUEBA 1: INICIALIZACIÓN SEGURA (Control de Acceso)
// Se registra la clave pública del hardware IMD en el expediente.
// A partir de este momento, SOLO ese dispositivo puede llamar log_vitals.
// =============================================================================
const maxInsulinDose = 30;  // U   — prescripción del médico para este paciente
const glucoseMin     = 70;  // mg/dL — límite inferior clínico del paciente
const glucoseMax     = 180; // mg/dL — límite superior clínico del paciente

console.log("\n1️⃣ Aseguradora inicializando expediente y registrando hardware IMD...");
const txInit = await pg.program.methods
  .initializeRecord(
    maxInsulinDose,
    glucoseMin,
    glucoseMax,
    imdDevice.publicKey  // Registramos la clave pública del hardware IMD
  )
  .accounts({
    medicalRecord: medicalRecordPda,
    aseguradora:   pg.wallet.publicKey,
    patient:       patient.publicKey,
    doctor:        doctor.publicKey,
    systemProgram: web3.SystemProgram.programId,
  })
  .signers([doctor, patient]) // El médico prescribe, el paciente da consentimiento (una sola vez)
  .rpc();

console.log("✅ Inicialización exitosa. Hash:", txInit);

// =============================================================================
// PRUEBA 2: REGISTRO DE SIGNOS VITALES (Telemetría autónoma del IMD)
// El hardware IMD detecta la glucosa y decide inyectar — opera en asa cerrada.
// Firma: imd_device  (la clave privada vive en el Secure Element del hardware)
// NO firma el paciente — no puede aprobar manualmente cada inyección a las 3am.
// El contrato verifica que quien firma sea el IMD registrado en initialize.
// =============================================================================
console.log("\n2️⃣ Hardware IMD registrando glucosa e inyectando insulina (autónomo)...");

const glucose = 120; // mg/dL — dentro del rango [70, 180] del paciente
const insulin = 15;  // U     — dentro del límite prescrito de 30 U

// Validación previa en cliente (defensa en profundidad — el contrato también valida)
if (glucose < glucoseMin || glucose > glucoseMax) {
  throw new Error(`[CLN-001] Glucosa ${glucose} fuera del rango del paciente [${glucoseMin}-${glucoseMax}] mg/dL`);
}
if (insulin < 1 || insulin > maxInsulinDose) {
  throw new Error(`[CLN-002] Insulina ${insulin} U excede el máximo prescrito de ${maxInsulinDose} U`);
}
if (glucose < 70 && insulin > 0) {
  throw new Error(`[CLN-003] Contraindicación: no administrar insulina con glucosa < 70 mg/dL`);
}

const txVitals = await pg.program.methods
  .logVitals(glucose, insulin)
  .accounts({
    medicalRecord: medicalRecordPda,
    patient:       patient.publicKey, // Solo para derivar la PDA — no firma
    imdDevice:     imdDevice.publicKey,
  })
  .signers([imdDevice]) // El hardware IMD firma — no el paciente dormido
  .rpc();

console.log("✅ Signos vitales registrados en la blockchain. Hash:", txVitals);

// =============================================================================
// PRUEBA 3: BOTÓN DE PÁNICO — Kill-Switch FDA
// FIX VLN-03-A (en lib.rs): el contexto EmergencyPause ahora ancla
//   medical_record con seeds + has_one = doctor, impidiendo que un doctor
//   pause el expediente de un paciente ajeno.
// =============================================================================
console.log("\n3️⃣ Simulando ataque cibernético... Médico activando Botón de Pánico 🛑");
const txPause = await pg.program.methods
  .emergencyPause()
  .accounts({
    medicalRecord: medicalRecordPda,
    doctor:        doctor.publicKey,
  })
  .signers([doctor]) // Solo el doctor registrado en ESTE expediente puede pausarlo
  .rpc();

console.log("✅ Dispositivo pausado de emergencia. Hash:", txPause);

// =============================================================================
// PRUEBA 4: VERIFICAR QUE EL DISPOSITIVO RECHAZA OPERACIONES POST-PAUSA
// =============================================================================
console.log("\n4️⃣ Verificando que el Kill-Switch bloquea nuevos registros del IMD...");
try {
  await pg.program.methods
    .logVitals(110, 10)
    .accounts({
      medicalRecord: medicalRecordPda,
      patient:       patient.publicKey,
      imdDevice:     imdDevice.publicKey,
    })
    .signers([imdDevice])
    .rpc();

  console.error("❌ ERROR CRÍTICO: El dispositivo aceptó un registro post-pausa.");
} catch (err) {
  console.log("✅ Kill-Switch verificado: el contrato rechazó el registro. (Esperado)");
  console.log("   Error recibido:", err.message?.split("\n")[0] ?? String(err));
}

// =============================================================================
// VERIFICACIÓN FINAL: LECTURA DEL ESTADO ON-CHAIN
// =============================================================================
console.log("\n--- LECTURA DEL ESTADO FINAL ON-CHAIN ---");
const recordData = await pg.program.account.medicalRecord.fetch(medicalRecordPda);

console.log("👤 Paciente:                ", recordData.patient.toBase58());
console.log("👨‍⚕️ Médico Tratante:          ", recordData.doctor.toBase58());
console.log("🔧 Hardware IMD Autorizado: ", recordData.imdDevice.toBase58());
console.log("🛡️ Dosis Máxima (Hard Cap):  ", recordData.maxInsulinDose.toString(), "U");
console.log("📊 Rango Glucosa Paciente:  ", recordData.glucoseMin.toString(), "–", recordData.glucoseMax.toString(), "mg/dL");
console.log("🩸 Última Glucosa:           ", recordData.lastGlucose.toString(), "mg/dL");
console.log("💉 Última Insulina:          ", recordData.lastInsulin.toString(), "U");
console.log("📋 Total de Registros:       ", recordData.totalLogs.toString());
console.log("🔖 Versión de Esquema:       ", recordData.schemaVersion.toString());
console.log("⚡ ¿Dispositivo Activo?:     ", recordData.isActive ? "SÍ 🟢" : "NO (Pausado por seguridad) 🔴");
