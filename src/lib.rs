use anchor_lang::prelude::*;
declare_id!("9hopPdzQVBaLqjD8mRYToYKHkdMrszGSCAnYTCowbGJ6");

//El cebrero: Aquí definimos las funciones (Lo que la black box puede hacer)
#[program]
pub mod medical_black_box {
    use super::*;

    //Función 1: Crear la Caja Negra para un nuevo paciente
    pub fn initialize_record(ctx: Context<InitializeRecord>) -> Result<()> {
        let record = &mut ctx.accounts.medical_record;
        record.patient = ctx.accounts.patient.key();
        record.total_logs = 0;
        
        msg!("Black Box inicializada exitosamente");
        Ok(())
    }
    //Función 2: Registrar inyectar insulina y niveles de glucosa (inmutable)
    pub fn log_vitals(
        ctx: Context<LogVitals>, 
        glucose_level: u16, 
        insulin_dose: u16
        ) -> Result<()> {
    //manejo de errores
    require!(glucose_level > 0, MedicalError::InvalidGlucose);
    require!(insulin_dose < 50, MedicalError::LethalDoseRisk);
        let record = &mut ctx.accounts.medical_record; 
        //Guardamos los datos médicos
        record.last_glucose = glucose_level;
        record.last_insulin = insulin_dose;
        //Obtenemos la hora exacta de la red de Solana (inmutable)
        record.last_timestamp = Clock::get()?.unix_timestamp;
        record.total_logs += 1;
        //Imprimimos el registro en la blockchain
        msg!(
            "Registro inmutable: Glucosa {} mg/dL, Insulina {} U, Hora: {}", 
            glucose_level, insulin_dose, record.last_timestamp
        );
        Ok(())
    }
}
//La base de datos: Cómo se estructura la información del paciente
#[account]
#[derive(InitSpace)] //sumador de bytes automatico (muy útil, gracias por el tip)
pub struct MedicalRecord {
    pub patient: Pubkey, // La identidad criptográfica del paciente
    pub last_glucose: u16,      // Nivel de glucosa 
    pub last_insulin: u16,      // Dosis inyectada 
    pub last_timestamp: i64,    // Hora exacta del evento
    pub total_logs: u64,        // Contador de cuántas veces ha operado la máquina
}
// Reglas de seguridad: Quién puede ejecutar las funciones
#[derive(Accounts)]
pub struct InitializeRecord<'info> {
    #[account(
        init, //aqui se crea la cuenta en la blockchain tantantaaannn
        payer = aseguradora, //la aseguradora paga la inicialización
        space = 8 + MedicalRecord::INIT_SPACE,
        seeds = [b"blackbox", patient.key().as_ref()], //pero el ID sigue siendo del paciente
        bump
    )]
    pub medical_record: Account<'info, MedicalRecord>,
    //la wallet corporativa (Tiene que firmar la transacción para autorizar el cobro)
    #[account(mut)]
    pub aseguradora: Signer<'info>, 
    //la wallet del paciente (Solo firma para dar consentimiento médico, pero NO paga nada)
    pub patient: Signer<'info>, 
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct LogVitals<'info> {
    //para actualizar la glucosa, Solana necesita buscar la blackbox usando los mismos seeds
    #[account(
        mut, 
        seeds = [b"blackbox", patient.key().as_ref()], // <-- LO BUSCAMOS CON EL MISMO SEED
        bump,
        has_one = patient
    )]
    pub medical_record: Account<'info, MedicalRecord>,
    
    pub patient: Signer<'info>,
}
#[error_code]
pub enum MedicalError {
    #[msg("Error Clínico: El nivel de glucosa no puede ser cero.")]
    InvalidGlucose,
    #[msg("Alerta de Riesgo: La dosis de insulina excede el límite seguro.")]
    LethalDoseRisk,
}
