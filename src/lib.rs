use anchor_lang::prelude::*;
declare_id!("9hopPdzQVBaLqjD8mRYToYKHkdMrszGSCAnYTCowbGJ6");

// =============================================================================
// M1-BlackBox — Cortafuegos Clínico para IMD (Dispositivo Médico Implantable)
// Cumplimiento: FDA Sección 524B | 21 CFR Part 11 | IEC 62304 | ISO 14971
// Auditoría: v4 — Modelo de firmas IoT corregido (asa cerrada)
//
// MODELO DE FIRMAS:
//   initialize_record → firman: aseguradora (paga) + doctor + paciente (consentimiento único)
//   log_vitals        → firma:  imd_device  (el hardware IMD, operación autónoma 24/7)
//   emergency_pause   → firma:  doctor      (kill-switch clínico)
//
// El consentimiento del paciente se da UNA VEZ en initialize_record.
// El dispositivo IMD opera en asa cerrada — no puede despertar al paciente
// a las 3am para firmar cada inyección.
// =============================================================================

// Rate Limiting: mínimo 5 minutos entre registros de dosis
const MIN_DOSE_INTERVAL_SEC: i64 = 300;

// Rangos fisiológicos absolutos (cualquier paciente, cualquier sensor FDA-cleared)
// Estos son los límites del hardware — los rangos del paciente se validan aparte
const GLUCOSE_ABS_MIN: u16 = 20;   // mg/dL — hipoglucemia severa
const GLUCOSE_ABS_MAX: u16 = 600;  // mg/dL — techo de sensores aprobados por FDA
const INSULIN_DOSE_MAX: u16 = 100; // U — techo absoluto para cualquier paciente

// Umbral de hipoglucemia: NO administrar insulina si la glucosa está por debajo
const HYPOGLYCEMIA_THRESHOLD: u16 = 70; // mg/dL

#[program]
pub mod medical_black_box {
    use super::*;

    // =========================================================================
    // Función 1: Crear la Caja Negra para un nuevo paciente
    // Firmantes: aseguradora (paga rent), doctor (prescripción), paciente (consentimiento único)
    // Se registra la clave pública del hardware IMD — solo ese dispositivo podrá hacer log_vitals
    // =========================================================================
    pub fn initialize_record(
        ctx: Context<InitializeRecord>,
        max_insulin_dose: u16,   // Dosis máxima prescrita — específica por paciente
        glucose_min: u16,        // Límite inferior de glucosa del paciente
        glucose_max: u16,        // Límite superior de glucosa del paciente
        imd_device_pubkey: Pubkey, // Clave pública del hardware IMD que operará en asa cerrada
    ) -> Result<()> {
        // --- VALIDACIÓN DE PARÁMETROS CLÍNICOS ---
        // FIX VLN-01-A: max_insulin_dose ya no acepta cualquier valor — rango seguro validado
        require!(
            max_insulin_dose >= 1 && max_insulin_dose <= INSULIN_DOSE_MAX,
            MedicalError::InvalidDoseRange
        );
        // FIX VLN-04-B: rangos de glucosa del paciente validados y almacenados
        require!(
            glucose_min >= GLUCOSE_ABS_MIN
                && glucose_max <= GLUCOSE_ABS_MAX
                && glucose_min < glucose_max,
            MedicalError::InvalidGlucoseRange
        );

        let record = &mut ctx.accounts.medical_record;

        record.patient          = ctx.accounts.patient.key();
        record.doctor           = ctx.accounts.doctor.key();
        record.imd_device       = imd_device_pubkey; // Clave del hardware IMD autorizado
        record.max_insulin_dose = max_insulin_dose;
        record.glucose_min      = glucose_min;
        record.glucose_max      = glucose_max;
        record.is_active        = true;
        record.total_logs       = 0;
        // Timestamp inicial = 0: el primer log_vitals no queda bloqueado por Rate Limiting
        record.last_timestamp   = 0;
        record.last_glucose     = 0;
        record.last_insulin     = 0;
        record.schema_version   = 1;

        // FIX VLN-01-B: emit!() on-chain para trazabilidad permanente (21 CFR 11.10(a))
        emit!(RecordInitialized {
            patient:          record.patient,
            doctor:           record.doctor,
            imd_device:       record.imd_device,
            max_insulin_dose: record.max_insulin_dose,
            glucose_min:      record.glucose_min,
            glucose_max:      record.glucose_max,
            timestamp:        Clock::get()?.unix_timestamp,
        });

        msg!("Black Box inicializada. Paciente: {}", record.patient);
        Ok(())
    }

    // =========================================================================
    // Función 2: Registrar signos vitales e inyección de insulina (inmutable)
    // Firmante requerido: imd_device — el hardware IMD opera en asa cerrada 24/7.
    // El consentimiento del paciente fue dado en initialize_record (una sola vez).
    // NO se exige firma del paciente en cada dosis: un páncreas artificial no puede
    // despertar al paciente a las 3am para aprobar cada inyección autónoma.
    // La autenticidad la garantiza la clave privada residente en el hardware IMD.
    // =========================================================================
    pub fn log_vitals(
        ctx: Context<LogVitals>,
        glucose_level: u16,
        insulin_dose: u16,
    ) -> Result<()> {
        let record = &mut ctx.accounts.medical_record;

        // CORTAFUEGOS 1: ¿Está activo el dispositivo?
        require!(record.is_active, MedicalError::DevicePaused);

        // CORTAFUEGOS 2: Rate Limiting — mínimo 5 minutos entre registros
        // Solo aplica a partir del segundo log (total_logs > 0)
        let current_time = Clock::get()?.unix_timestamp;
        if record.total_logs > 0 {
            require!(
                current_time - record.last_timestamp >= MIN_DOSE_INTERVAL_SEC,
                MedicalError::RateLimitExceeded
            );
        }

        // CORTAFUEGOS 3: Rango clínico de glucosa — validado contra los rangos del paciente
        require!(
            glucose_level >= record.glucose_min && glucose_level <= record.glucose_max,
            MedicalError::GlucoseOutOfRange
        );

        // CORTAFUEGOS 4: Dosis máxima parametrizada por paciente
        require!(
            insulin_dose >= 1 && insulin_dose <= record.max_insulin_dose,
            MedicalError::ExceedsPatientMaxDose
        );

        // CORTAFUEGOS 5: Coherencia clínica — NO insulina en hipoglucemia
        require!(
            !(glucose_level < HYPOGLYCEMIA_THRESHOLD && insulin_dose > 0),
            MedicalError::HypoglycemiaContraindication
        );

        // Persistir datos
        record.last_glucose   = glucose_level;
        record.last_insulin   = insulin_dose;
        record.last_timestamp = current_time;
        record.total_logs    += 1;

        emit!(VitalsLogged {
            patient:   record.patient,
            imd_device: ctx.accounts.imd_device.key(),
            glucose:   glucose_level,
            insulin:   insulin_dose,
            timestamp: current_time,
            log_index: record.total_logs,
        });

        msg!(
            "Registro inmutable #{}: Glucosa {} mg/dL, Insulina {} U | IMD: {}",
            record.total_logs,
            glucose_level,
            insulin_dose,
            ctx.accounts.imd_device.key()
        );
        Ok(())
    }

    // =========================================================================
    // Función 3: Kill-Switch de emergencia — solo el doctor registrado puede activarlo
    // FIX VLN-03-A: contexto ahora ancla medical_record a la PDA del paciente
    // =========================================================================
    pub fn emergency_pause(ctx: Context<EmergencyPause>) -> Result<()> {
        let record = &mut ctx.accounts.medical_record;

        // La verificación de autoridad ya la hace has_one = doctor en el contexto,
        // pero la mantenemos explícita como segunda capa de defensa
        require!(
            record.doctor == ctx.accounts.doctor.key(),
            MedicalError::UnauthorizedDoctor
        );

        record.is_active = false;

        emit!(DevicePausedEvent {
            patient:   record.patient,
            doctor:    ctx.accounts.doctor.key(),
            timestamp: Clock::get()?.unix_timestamp,
        });

        msg!("EMERGENCIA: Dispositivo IMD pausado por el doctor {}.", ctx.accounts.doctor.key());
        Ok(())
    }
}

// =============================================================================
// ESTRUCTURA DE DATOS — MedicalRecord
// FIX VLN-04-A / VLN-04-B / VLN-04-C: campos clínicos completos + schema_version
// =============================================================================
#[account]
#[derive(InitSpace)]
pub struct MedicalRecord {
    // — Identidad —
    pub patient:          Pubkey, // Identidad criptográfica del paciente
    pub doctor:           Pubkey, // Autoridad médica registrada
    pub imd_device:       Pubkey, // Clave pública del hardware IMD autorizado (asa cerrada)

    // — Parámetros clínicos del paciente (fijados por médico en initialize) —
    pub max_insulin_dose: u16,   // Dosis máxima prescrita — específica por paciente (C-01)
    pub glucose_min:      u16,   // Límite inferior de glucosa del paciente
    pub glucose_max:      u16,   // Límite superior de glucosa del paciente

    // — Datos operacionales del último registro —
    pub last_glucose:     u16,   // Última glucosa registrada (mg/dL)
    pub last_insulin:     u16,   // Última dosis inyectada (U)
    pub last_timestamp:   i64,   // Timestamp del último evento (Rate Limiting)

    // — Contadores y control de ciclo de vida —
    pub total_logs:       u64,   // Total de registros inmutables
    pub is_active:        bool,  // Kill-Switch de emergencia (M-03)
    pub schema_version:   u8,    // Versión del esquema — para migraciones seguras (m-02)
}

// =============================================================================
// CONTEXTOS — Reglas de acceso por función
// =============================================================================

// Inicialización: paga la aseguradora, firman doctor + paciente
#[derive(Accounts)]
pub struct InitializeRecord<'info> {
    #[account(
        init,
        payer = aseguradora,
        space = 8 + MedicalRecord::INIT_SPACE,
        seeds = [b"blackbox", patient.key().as_ref()],
        bump
    )]
    pub medical_record: Account<'info, MedicalRecord>,

    #[account(mut)]
    pub aseguradora: Signer<'info>,   // Paga la transacción (rent)

    pub patient: Signer<'info>,       // Consentimiento del paciente
    pub doctor:  Signer<'info>,       // Prescripción médica firmada

    pub system_program: Program<'info, System>,
}

// Log de vitales: firma el hardware IMD (asa cerrada 24/7)
// patient es AccountInfo — solo se usa como semilla de la PDA.
// La autenticación se delega al imd_device: su clave privada vive en el hardware implantado.
// has_one = imd_device garantiza que SOLO el IMD registrado en initialize puede enviar datos.
#[derive(Accounts)]
pub struct LogVitals<'info> {
    #[account(
        mut,
        seeds = [b"blackbox", patient.key().as_ref()],
        bump,
        has_one = patient,     // La PDA pertenece a este paciente
        has_one = imd_device   // Solo el IMD autorizado puede registrar vitales
    )]
    pub medical_record: Account<'info, MedicalRecord>,

    /// CHECK: Solo se usa como semilla para derivar la PDA. No firma — el IMD opera autónomamente.
    pub patient: AccountInfo<'info>,

    pub imd_device: Signer<'info>, // El hardware IMD firma cada transmisión de datos
}

// Pausa de emergencia: el doctor debe firmar y solo puede actuar sobre su propio paciente
// FIX VLN-03-A: medical_record ahora está anclado con seeds + has_one = doctor
#[derive(Accounts)]
pub struct EmergencyPause<'info> {
    #[account(
        mut,
        seeds = [b"blackbox", medical_record.patient.as_ref()],
        bump,
        has_one = doctor // Verifica que medical_record.doctor == doctor.key()
    )]
    pub medical_record: Account<'info, MedicalRecord>,

    pub doctor: Signer<'info>, // Solo el doctor registrado en ESTE expediente puede pausarlo
}

// =============================================================================
// EVENTOS ON-CHAIN — Trazabilidad forense permanente (21 CFR 11.10(a))
// FIX VLN-01-B / VLN-02-E: emit!() reemplaza/complementa msg!()
// =============================================================================
#[event]
pub struct RecordInitialized {
    pub patient:          Pubkey,
    pub doctor:           Pubkey,
    pub imd_device:       Pubkey,
    pub max_insulin_dose: u16,
    pub glucose_min:      u16,
    pub glucose_max:      u16,
    pub timestamp:        i64,
}

#[event]
pub struct VitalsLogged {
    pub patient:    Pubkey,
    pub imd_device: Pubkey, // Trazabilidad: qué hardware específico generó este registro
    pub glucose:    u16,
    pub insulin:    u16,
    pub timestamp:  i64,
    pub log_index:  u64,
}

#[event]
pub struct DevicePausedEvent {
    pub patient:   Pubkey,
    pub doctor:    Pubkey,
    pub timestamp: i64,
}

// =============================================================================
// CÓDIGOS DE ERROR — Cobertura clínica completa
// FIX: 2 errores originales expandidos a 9 con semántica clínica precisa
// =============================================================================
#[error_code]
pub enum MedicalError {
    // Errores clínicos
    #[msg("[CLN-001] Glucosa fuera del rango clínico del paciente (20-600 mg/dL).")]
    GlucoseOutOfRange,

    #[msg("[CLN-002] Dosis excede el máximo prescrito para este paciente.")]
    ExceedsPatientMaxDose,

    #[msg("[CLN-003] Contraindicación: no se puede administrar insulina con glucosa < 70 mg/dL.")]
    HypoglycemiaContraindication,

    // Errores de seguridad / Rate Limiting
    #[msg("[SEC-001] Rate Limit: deben pasar 5 minutos entre registros de dosis.")]
    RateLimitExceeded,

    #[msg("[SEC-002] Dispositivo pausado por emergencia. No se aceptan registros.")]
    DevicePaused,

    #[msg("[SEC-003] Firma médica no autorizada para este expediente.")]
    UnauthorizedDoctor,

    // Errores de inicialización / configuración
    #[msg("[CFG-001] Dosis máxima inválida: debe estar entre 1 y 100 unidades.")]
    InvalidDoseRange,

    #[msg("[CFG-002] Rango de glucosa inválido: min >= 20, max <= 600, min < max.")]
    InvalidGlucoseRange,

    // Mantenido por compatibilidad con código legacy
    #[msg("[LEG-001] Nivel de glucosa no puede ser cero.")]
    InvalidGlucose,
}
