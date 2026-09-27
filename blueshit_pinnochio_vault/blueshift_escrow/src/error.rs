#[derive(Debug)]
pub enum EscrowError {
    InvalidInstructionData,
    InvalidAccountOwner,
    InvalidEscrow,
    InvalidAmount,
}

pub type ProgramError = EscrowError;