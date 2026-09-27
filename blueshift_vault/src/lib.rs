#![no_std]

use core::convert::TryInto;
use core::mem::size_of;

use pinocchio::{
    account::AccountView as AccountInfo,
    address::Address as Pubkey,
    entrypoint,
    error::ProgramResult,
    nostd_panic_handler,
};
use pinocchio::instruction::cpi::{Seed, Signer};
use pinocchio_system::instructions::Transfer;
use pinocchio_system::ID as SYSTEM_ID;

entrypoint!(process_instruction);
nostd_panic_handler!();

// 22222222222222222222222222222222222222222222
pub const ID: Pubkey = Pubkey::new_from_array([
    0x0f, 0x1e, 0x6b, 0x14, 0x21, 0xc0, 0x4a, 0x07,
    0x04, 0x31, 0x26, 0x5c, 0x19, 0xc5, 0xbb, 0xee,
    0x19, 0x92, 0xba, 0xe8, 0xaf, 0xd1, 0xcd, 0x07,
    0x8e, 0xf8, 0xaf, 0x70, 0x47, 0xdc, 0x11, 0xf7,
]);

fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    match instruction_data.split_first() {
        Some((disc, data)) => match *disc {
            Deposit::DISCRIMINATOR => {
                Deposit::try_from((data, accounts))?.process()?;
                Ok(())
            }
            Withdraw::DISCRIMINATOR => {
                Withdraw::try_from(accounts)?.process()?;
                Ok(())
            }
            _ => Err(1u64.into()), // Error code for invalid instruction
        },
        None => Err(1u64.into()), // Error code for invalid instruction
    }
}

pub struct DepositAccounts<'a> {
    pub owner: &'a AccountInfo,
    pub vault: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for DepositAccounts<'a> {
    type Error = u64; // Error code

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        if accounts.len() < 2 {
            return Err(1);
        }
        let owner = &accounts[0];
        let vault = &accounts[1];


        // Accounts Checks
        if !owner.is_signer() {
            return Err(1);
        }

        if !vault.owned_by(&SYSTEM_ID) {
            return Err(1);
        }

        if vault.lamports().ne(&0) {
            return Err(1);
        }

        let (vault_key, _) = solana_program::pubkey::Pubkey::find_program_address(&[b"vault", owner.key.as_ref()], &crate::ID.to_bytes());
        if vault.key != &vault_key {
            return Err(1);
        }

        // Return the accounts
        Ok(Self { owner, vault })
    }
}

pub struct DepositInstructionData {
    pub amount: u64,
}

impl<'a> TryFrom<&'a [u8]> for DepositInstructionData {
    type Error = u64; // Error code

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        if data.len() != size_of::<u64>() {
            return Err(1);
        }

        let amount = u64::from_le_bytes(data.try_into().unwrap());

        // Instruction Checks
        if amount.eq(&0) {
            return Err(1);
        }

        Ok(Self { amount })
    }
}

pub struct Deposit<'a> {
    pub accounts: DepositAccounts<'a>,
    pub instruction_data: DepositInstructionData,
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for Deposit<'a> {
    type Error = u64; // Error code

    fn try_from((data, accounts): (&'a [u8], &'a [AccountInfo])) -> Result<Self, Self::Error> {
        let accounts = DepositAccounts::try_from(accounts)?;
        let instruction_data = DepositInstructionData::try_from(data)?;

        Ok(Self {
            accounts,
            instruction_data,
        })
    }
}

impl<'a> Deposit<'a> {
    pub const DISCRIMINATOR: u8 = 0;

    pub fn process(&mut self) -> ProgramResult {
        Transfer {
            from: self.accounts.owner,
            to: self.accounts.vault,
            lamports: self.instruction_data.amount,
        }
        .invoke()?;

        Ok(())
    }
}

pub struct WithdrawAccounts<'a> {
    pub owner: &'a AccountInfo,
    pub vault: &'a AccountInfo,
    pub bumps: [u8; 1],
}

impl<'a> TryFrom<&'a [AccountInfo]> for WithdrawAccounts<'a> {
    type Error = u64; // Error code

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        if accounts.len() < 2 {
            return Err(1);
        }
        let owner = &accounts[0];
        let vault = &accounts[1];

        // Basic Accounts Checks
        if !owner.is_signer() {
            return Err(1);
        }

        if !vault.owned_by(&SYSTEM_ID) {
            return Err(1);
        }

        if vault.lamports().eq(&0) {
            return Err(1);
        }

        let (vault_key, bump) = solana_program::pubkey::Pubkey::find_program_address(&[b"vault", owner.key.as_ref()], &crate::ID.to_bytes());
        if &vault_key != vault.key {
            return Err(1);
        }

        Ok(Self { owner, vault, bumps: [bump] })
    }
}

pub struct Withdraw<'a> {
    pub accounts: WithdrawAccounts<'a>,
}

impl<'a> TryFrom<&'a [AccountInfo]> for Withdraw<'a> {
    type Error = u64; // Error code

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let accounts = WithdrawAccounts::try_from(accounts)?;

        Ok(Self { accounts })
    }
}

impl<'a> Withdraw<'a> {
    pub const DISCRIMINATOR: u8 = 1;

    pub fn process(&mut self) -> ProgramResult {
        // Create PDA signer seeds
        let seeds = [
            Seed::from(b"vault"),
            Seed::from(self.accounts.owner.key.as_ref()),
            Seed::from(&self.accounts.bumps),
        ];
        let signers = [Signer::from(&seeds)];

        // Transfer all lamports from vault to owner
        Transfer {
            from: self.accounts.vault,
            to: self.accounts.owner,
            lamports: self.accounts.vault.lamports(),
        }
        .invoke_signed(&signers)?;

        Ok(())
    }
}