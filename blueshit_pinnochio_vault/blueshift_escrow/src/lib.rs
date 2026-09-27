use pinocchio::{entrypoint, AccountView, Address, ProgramResult};

entrypoint!(process_instruction);

pub mod instructions;
pub use instructions::*;

pub mod state;
pub use state::*;

pub fn process_instruction(
    program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    match instruction_data.split_first() {
        Some((discriminator, data)) if *discriminator == 0 => {
            // Make instruction
            let _make = instructions::make_simple::Make::try_from((data, accounts));
            Ok(())
        },
        Some((discriminator, _)) if *discriminator == 1 => {
            // Take instruction
            let _take = instructions::take_simple::Take::try_from(accounts);
            Ok(())
        },
        Some((discriminator, _)) if *discriminator == 2 => {
            // Refund instruction
            let _refund = instructions::refund_simple::Refund::try_from(accounts);
            Ok(())
        },
        _ => Ok(()),
    }
}
