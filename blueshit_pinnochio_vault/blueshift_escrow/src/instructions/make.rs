use pinocchio::{
    account_info::AccountInfo,
    ProgramError,
    pubkey::Pubkey,
    ProgramResult,
    program::{invoke, invoke_signed},
    instruction::AccountMeta,
    sysvars::Sysvar,
};
use pinocchio_token::{
    instruction::transfer,
};
use pinocchio_associated_token_account::{
    create_associated_token_account,
};
use pinocchio_system::{
    create_program_address,
    find_program_address,
};
use pinocchio::program::invoke;
use crate::{
    state::Escrow,
};
use core::mem::size_of;

pub struct MakeAccounts<'a> {
    pub maker: &'a AccountInfo,
    pub escrow: &'a AccountInfo,
    pub mint_a: &'a AccountInfo,
    pub mint_b: &'a AccountInfo,
    pub maker_ata_a: &'a AccountInfo,
    pub vault: &'a AccountInfo,
    pub system_program: &'a AccountInfo,
    pub token_program: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for MakeAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [maker, escrow, mint_a, mint_b, maker_ata_a, vault, system_program, token_program, _] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Basic Accounts Checks
        if !maker.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if !escrow.is_writable || !maker_ata_a.is_writable || !vault.is_writable {
            return Err(ProgramError::InvalidAccountData);
        }

        if !system_program.executable || !token_program.executable {
            return Err(ProgramError::InvalidAccountData);
        }

        // Return the accounts
        Ok(Self {
            maker,
            escrow,
            mint_a,
            mint_b,
            maker_ata_a,
            vault,
            system_program,
            token_program,
        })
    }
}

pub struct MakeInstructionData {
    pub seed: u64,
    pub receive: u64,
    pub amount: u64,
}

impl<'a> TryFrom<&'a [u8]> for MakeInstructionData {
    type Error = ProgramError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        if data.len() != size_of::<u64>() * 3 {
            return Err(ProgramError::InvalidInstructionData);
        }

        let seed = u64::from_le_bytes(data[0..8].try_into().unwrap());
        let receive = u64::from_le_bytes(data[8..16].try_into().unwrap());
        let amount = u64::from_le_bytes(data[16..24].try_into().unwrap());

        // Instruction Checks
        if amount == 0 {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(Self {
            seed,
            receive,
            amount,
        })
    }
}

pub struct Make<'a> {
    pub accounts: MakeAccounts<'a>,
    pub instruction_data: MakeInstructionData,
    pub bump: u8,
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for Make<'a> {
    type Error = ProgramError;
    
    fn try_from((data, accounts): (&'a [u8], &'a [AccountInfo])) -> Result<Self, Self::Error> {
        let accounts = MakeAccounts::try_from(accounts)?;
        let instruction_data = MakeInstructionData::try_from(data)?;

        // Find PDA bump
        let (_, bump) = find_program_address(
            &[
                b"escrow", 
                accounts.maker.key().as_ref(), 
                &instruction_data.seed.to_le_bytes()
            ], 
            &crate::ID
        );

        // Initialize the Escrow account
        let seed_binding = instruction_data.seed.to_le_bytes();
        let bump_binding = [bump];
        let signer_seeds = &[
            b"escrow",
            accounts.maker.key().as_ref(),
            &seed_binding,
            &bump_binding,
        ];
        let signer = &[&signer_seeds[..]];
        let rent = pinocchio::sysvars::rent::Rent::get()?;
        let escrow_space = Escrow::LEN;
        let required_lamports = rent.try_minimum_balance(escrow_space)?;

        // Transfer lamports from maker to escrow account
        let create_account_instruction = pinocchio_system::instruction::create_account(
            accounts.maker.key(),
            accounts.escrow.key(),
            required_lamports,
            escrow_space as u64,
            &crate::ID,
        );
        invoke(
            &create_account_instruction,
            &[
                accounts.maker,
                accounts.escrow,
                accounts.system_program,
            ],
        )?;

        // Initialize the vault (ATA for escrow)
        let create_ata_instruction = create_associated_token_account(
            accounts.maker.key(),
            accounts.escrow.key(),
            accounts.mint_a.key(),
            accounts.token_program.key(),
        );
        invoke(
            &create_ata_instruction,
            &[
                accounts.maker,
                accounts.vault,
                accounts.escrow,
                accounts.mint_a,
                accounts.system_program,
                accounts.token_program,
            ],
        )?;

        Ok(Self {
            accounts,
            instruction_data,
            bump,
        })
    }
}

impl<'a> Make<'a> {
    pub const DISCRIMINATOR: u8 = 0;
    
    pub fn process(&mut self) -> ProgramResult {
        // Populate the escrow account
        let mut data = self.accounts.escrow.try_borrow_mut_data()?;
        let escrow = Escrow::load_mut(data.as_mut())?;
        
        escrow.set_inner(
            self.instruction_data.seed,
            *self.accounts.maker.key(),
            *self.accounts.mint_a.key(),
            *self.accounts.mint_b.key(),
            self.instruction_data.receive,
            [self.bump],
        );

        // Transfer tokens to vault
        let transfer_instruction = transfer(
            self.accounts.maker_ata_a.key(),
            self.accounts.vault.key(),
            self.accounts.maker.key(),
            &self.instruction_data.amount.to_le_bytes(),
        );
        invoke(
            &transfer_instruction,
            &[
                self.accounts.token_program,
                self.accounts.maker_ata_a,
                self.accounts.vault,
                self.accounts.maker,
            ],
        )?;

        Ok(())
    }
}