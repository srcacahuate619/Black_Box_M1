//Bienvenidos a la "interfaz" de blackbox
//mi wallet actual de playground actuará como la aseguradora (la que paga)
const aseguradora = pg.wallet.keypair; 
//generamos una wallet completamente nueva y aleatoria para simular a un paciente real
const patient = new web3.Keypair(); 
console.log("Iniciando simulación de la Caja Negra Médica...");
console.log("Wallet Aseguradora:", aseguradora.publicKey.toString());
console.log("Wallet Paciente:", patient.publicKey.toString());
//encontramos la blackbox(el calculo del pda)
const [medicalRecordPda, bump] = web3.PublicKey.findProgramAddressSync(
  [Buffer.from("blackbox"), patient.publicKey.toBuffer()],
  pg.program.programId
);
console.log("Dirección de la Caja Negra (PDA):", medicalRecordPda.toString());
//envolveremos todo en una función asíncrona principal
async function main() {
  try {
    //inicializar el expediente (la Aseguradora paga)
    console.log("\n1️Creando el expediente médico en la blockchain...");
    const txHashInit = await pg.program.methods
      .initializeRecord()
      .accounts({
        medicalRecord: medicalRecordPda,
        aseguradora: aseguradora.publicKey,
        patient: patient.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([aseguradora, patient]) //ambos tienen que firmar
      .rpc();
    console.log("Expediente creado. Hash de transacción:", txHashInit);
    //registrar signos(la Bomba de Insulina inyecta datos)
    const glucosa = 120;  //mg/dL
    const insulina = 5;   //unidades
    console.log(`\n2️Registrando signos: Glucosa ${glucosa}, Insulina ${insulina}...`);
    const txHashLog = await pg.program.methods
      .logVitals(glucosa, insulina)
      .accounts({
        medicalRecord: medicalRecordPda,
        patient: patient.publicKey,
      })
      .signers([patient]) //aquí solo firma el dispositivo del paciente
      .rpc();
    console.log("Datos registrados. Hash de transacción:", txHashLog);
    //auditria clinica (leemos la blockchain para demostrar la inmutabilidad)
    console.log("\nLeyendo la blockchain para confirmar los datos guardados...");
    const record = await pg.program.account.medicalRecord.fetch(medicalRecordPda);
    console.log("\nDATOS INMUTABLES GUARDADOS CON ÉXITO:");
    console.log("- Dueño del Expediente:", record.patient.toString());
    console.log("- Última Glucosa:", record.lastGlucose.toString(), "mg/dL");
    console.log("- Última Insulina:", record.lastInsulin.toString(), "U");
    console.log("- Total de registros:", record.totalLogs.toString());  
    //convertimos el Timestamp de Solana (segundos) a una fecha legible
    const fecha = new Date(record.lastTimestamp.toNumber() * 1000);
    console.log("- Hora exacta (Descentralizada):", fecha.toLocaleString());
  } catch (error) {
    console.error("Ocurrió un error en la simulación:", error);
  }
}
//ejecutar la simulación
main();
