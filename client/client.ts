// =================================================================
// SIMULACIÓN CLÍNICA: M1-BlackBox (Cumplimiento FDA 524B)
// =================================================================

console.log("Iniciando pruebas clínicas de M1-BlackBox...");

// 1️⃣ CREACIÓN DE IDENTIDADES (Wallets de simulación)
// Generamos pacientes y médicos ficticios para la prueba
const patient = web3.Keypair.generate();
const doctor = web3.Keypair.generate(); // El médico autorizado por el hospital

// 2️⃣ DERIVACIÓN DE LA PDA (El Expediente Inmutable)
const [medicalRecordPda] = web3.PublicKey.findProgramAddressSync(
  [Buffer.from("blackbox"), patient.publicKey.toBuffer()],
  pg.program.programId
);

console.log("📄 PDA del Expediente derivado:", medicalRecordPda.toBase58());

// =================================================================
// PRUEBA 1: INICIALIZACIÓN SEGURA (Control de Acceso)
// =================================================================
const maxInsulinDose = 50; // Hard Cap: Ningún algoritmo puede inyectar más de 50 unidades

console.log("\n1️⃣ Aseguradora inicializando expediente y validando firma médica...");
const txInit = await pg.program.methods
  .initializeRecord(maxInsulinDose)
  .accounts({
    medicalRecord: medicalRecordPda,
    aseguradora: pg.wallet.publicKey, // Quien paga la transacción
    patient: patient.publicKey,
    doctor: doctor.publicKey,         // La llave pública del médico
    systemProgram: web3.SystemProgram.programId,
  })
  .signers([doctor, patient]) // El médico autoriza y el paciente da consentimiento
  .rpc();

console.log("✅ Inicialización exitosa. Hash:", txInit);

// =================================================================
// PRUEBA 2: REGISTRO DE SIGNOS VITALES (Telemetría IoT)
// =================================================================
console.log("\n2️⃣ Dispositivo IoT registrando glucosa e inyectando insulina...");
const glucose = 120;
const insulin = 15; // Dosis segura (menor a 50)

const txVitals = await pg.program.methods
  .logVitals(glucose, insulin)
  .accounts({
    medicalRecord: medicalRecordPda,
    patient: patient.publicKey, // Le pasamos el paciente para que valide la semilla
  })
  .rpc(); // Quitamos .signers() porque el dispositivo IoT "dispara" esto automáticamente

console.log("✅ Signos vitales registrados en la blockchain. Hash:", txVitals);

// =================================================================
// PRUEBA 3: EL BOTÓN DE PÁNICO (Kill-Switch FDA)
// =================================================================
console.log("\n3️⃣ Simulando ataque cibernético... Médico activando Botón de Pánico 🛑");
const txPause = await pg.program.methods
  .emergencyPause()
  .accounts({
    medicalRecord: medicalRecordPda,
    doctor: doctor.publicKey,
  })
  .signers([doctor]) // Solo el doctor dueño de la firma puede apagarlo
  .rpc();

console.log("✅ Dispositivo pausado de emergencia. Hash:", txPause);

// =================================================================
// VERIFICACIÓN: LECTURA DEL ESTADO ON-CHAIN
// =================================================================
console.log("\n--- LECTURA DEL ESTADO FINAL ON-CHAIN ---");
const recordData = await pg.program.account.medicalRecord.fetch(medicalRecordPda);

console.log("👤 Paciente:", recordData.patient.toBase58());
console.log("👨‍⚕️ Médico Tratante:", recordData.doctor.toBase58());
console.log("🛡️ Dosis Máxima Permitida (Hard Cap):", recordData.maxInsulinDose.toString(), "unidades");
console.log("🩸 Última Glucosa:", recordData.lastGlucose.toString(), "mg/dL");
console.log("💉 Última Insulina:", recordData.lastInsulin.toString(), "unidades");
console.log("⚡ ¿Dispositivo Activo?:", recordData.isActive ? "SÍ 🟢" : "NO (Pausado por seguridad) 🔴");
