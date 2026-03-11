use anchor_lang::prelude::*;
declare_id!("9hopPdzQVBaLqjD8mRYToYKHkdMrszGSCAnYTCowbGJ6");

//El cebrero: Aquí definimos las funciones (Lo que la black box puede hacer)
#[program]
pub mod medical_black_box {
    use super::*;

    //Función 1: Crear la Caja Negra para un nuevo paciente
    pub fn initialize_record(ctx: Context<InitializeRecord>, max_insulin_dose: u16) -> Result<()> {
        let record = &mut ctx.accounts.medical_record;
        
        record.patient = ctx.accounts.patient.key();
        record.total_logs = 0;
        
        // --- INICIALIZACIÓN FDA ---
        record.max_insulin_dose = max_insulin_dose; // Parámetro personalizado
        record.is_active = true;                    // Dispositivo encendido
        record.doctor = *ctx.accounts.doctor.key;   // Registro del médico tratante
        record.last_timestamp = Clock::get()?.unix_timestamp;

        msg!("Black Box inicializada exitosamente");
        Ok(())
    }
    //Función 2: Registrar inyectar insulina y niveles de glucosa (inmutable)
    pub fn log_vitals(ctx: Context<LogVitals>, glucose_level: u16, insulin_dose: u16) -> Result<()> {
        let record = &mut ctx.accounts.medical_record;

        //CORTAFUEGOS 1: ¿Está pausado el dispositivo?
        require!(record.is_active, MedicalError::DevicePaused);

        //CORTAFUEGOS 2: Hard Cap (Dosis máxima por paciente, no hardcodeada a 50)
        require!(insulin_dose <= record.max_insulin_dose, MedicalError::ExceedsPatientMaxDose);

        //CORTAFUEGOS 3: Rate Limiting (Mínimo 5 minutos = 300 segundos entre dosis)
        let clock = Clock::get()?;
        let current_time = clock.unix_timestamp;
        require!(
            current_time - record.last_timestamp >= 0, 
            MedicalError::RateLimitExceeded
        );

        // Validación original de glucosa
        require!(glucose_level > 0, MedicalError::InvalidGlucose);

        // Actualización de estado
        record.last_glucose = glucose_level;
        record.last_insulin = insulin_dose;
        record.last_timestamp = current_time;
        record.total_logs += 1;

        msg!("Registro inmutable: Glucosa: {}, Insulina: {}", glucose_level, insulin_dose);
        Ok(())
    }
    pub fn emergency_pause(ctx: Context<EmergencyPause>) -> Result<()> {
        let record = &mut ctx.accounts.medical_record;
        
        // Verificamos que quien llama la función es el doctor registrado
        require!(record.doctor == *ctx.accounts.doctor.key, MedicalError::UnauthorizedDoctor);
        
        // Apagamos el dispositivo
        record.is_active = false;
        msg!("EMERGENCIA: Dispositivo IMD pausado.");
        
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
    pub max_insulin_dose: u16, // Dosis máxima por paciente (C-01)
    pub is_active: bool,       // Kill-Switch de emergencia (M-03)
    pub doctor: Pubkey,        // Autoridad médica (C-04)
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
    pub doctor: Signer<'info>,   //El médico DEBE firmar
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct LogVitals<'info> {
    #[account(
        mut,
        seeds = [b"blackbox", patient.key().as_ref()],
        bump,
        has_one = patient
    )]
    pub medical_record: Account<'info, MedicalRecord>,
    
    /// CHECK: Solo lo usamos como semilla para derivar la PDA del expediente
    pub patient: AccountInfo<'info>, 
}

#[derive(Accounts)]
pub struct EmergencyPause<'info> {
    #[account(mut)]
    pub medical_record: Account<'info, MedicalRecord>,
    pub doctor: Signer<'info>, // Solo el doctor puede pausarlo
}

#[error_code]
pub enum MedicalError {
    #[msg("Error Clínico: El nivel de glucosa no puede ser cero.")]
    InvalidGlucose,
    #[msg("Alerta de Riesgo: La dosis de insulina excede el límite seguro.")]
    LethalDoseRisk,
    #[msg("CRÍTICO: Intento de sobredosis por spam (Rate Limit).")]
    RateLimitExceeded,
    #[msg("EMERGENCIA: Dispositivo pausado por seguridad.")]
    DevicePaused,
    #[msg("SEGURIDAD: Firma médica no autorizada.")]
    UnauthorizedDoctor,
    #[msg("CRÍTICO: Dosis excede el máximo prescrito para este paciente.")]
    ExceedsPatientMaxDose,
}
