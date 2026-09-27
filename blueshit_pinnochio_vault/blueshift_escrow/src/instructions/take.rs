use pinocchio::{
    account_info::AccountInfo,
    ProgramError,
    pubkey::Pubkey,
    ProgramResult,
    program::{invoke_signed, invoke},
    system_instruction,
    instruction::AccountMeta,
};
use pinocchio_token::{
    instruction::{transfer, close_account},
    TokenAccount,
};
use pinocchio_associated_token_account::{
    create_associated_token_account,
    get_associated_token_address,
};
use pinocchio_system::{
    create_program_address,
};
use crate::{
    state::Escrow,
};

pub struct TakeAccounts<'a> {
    pub taker: &'a AccountInfo,
    pub maker: &'a AccountInfo,
    pub escrow: &'a AccountInfo,
    pub mint_a: &'a AccountInfo,
    pub mint_b: &'a AccountInfo,
    pub vault: &'a AccountInfo,
    pub taker_ata_a: &'a AccountInfo,
    pub taker_ata_b: &'a AccountInfo,
    pub maker_ata_b: &'a AccountInfo,
    pub system_program: &'a AccountInfo,
    pub token_program: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for TakeAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [taker, maker, escrow, mint_a, mint_b, vault, taker_ata_a, taker_ata_b, maker_ata_b, system_program, token_program, _] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Basic Accounts Checks
        if !taker.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if !escrow.is_writable || !maker.is_writable || !vault.is_writable 
            || !taker_ata_a.is_writable || !taker_ata_b.is_writable || !maker_ata_b.is_writable {
            return Err(ProgramError::InvalidAccountData);
        }

        if !system_program.executable || !token_program.executable {
            return Err(ProgramError::InvalidAccountData);
        }

        // Return the accounts
        Ok(Self {
            taker,
            maker,
            escrow,
            mint_a,
            mint_b,
            taker_ata_a,
            taker_ata_b,
            maker_ata_b,
            vault,
            system_program,
            token_program,
        })
    }
}

pub struct Take<'a> {
    pub accounts: TakeAccounts<'a>,
}

impl<'a> TryFrom<&'a [AccountInfo]> for Take<'a> {
    type Error = ProgramError;
    
    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let accounts = TakeAccounts::try_from(accounts)?;

        // Initialize necessary accounts if needed
        let taker_ata_a_address = get_associated_token_address(accounts.taker.key(), accounts.mint_a.key());
        if accounts.taker_ata_a.key() != &taker_ata_a_address {
            return Err(ProgramError::InvalidAccountData);
        }

        let maker_ata_b_address = get_associated_token_address(accounts.maker.key(), accounts.mint_b.key());
        if accounts.maker_ata_b.key() != &maker_ata_b_address {
            return Err(ProgramError::InvalidAccountData);
        }

        // Check if accounts need initialization (simplified check)
        if accounts.taker_ata_a.lamports() == 0 {
            let instruction = create_associated_token_account(
                accounts.taker.key(),
                accounts.taker.key(),
                accounts.mint_a.key(),
                accounts.token_program.key(),
            );
            invoke(&instruction, &[
                accounts.taker,
                accounts.taker_ata_a,
                accounts.taker,
                accounts.mint_a,
                accounts.system_program,
                accounts.token_program,
            ])?;
        }

        if accounts.maker_ata_b.lamports() == 0 {
            let instruction = create_associated_token_account(
                accounts.taker.key(),
                accounts.maker.key(),
                accounts.mint_b.key(),
                accounts.token_program.key(),
            );
            invoke(&instruction, &[
                accounts.taker,
                accounts.maker_ata_b,
                accounts.maker,
                accounts.mint_b,
                accounts.system_program,
                accounts.token_program,
            ])?;
        }

        Ok(Self {
            accounts,
        })
    }
}

impl<'a> Take<'a> {
    pub const DISCRIMINATOR: u8 = 1;
    
    pub fn process(&mut self) -> ProgramResult {
        let data = self.accounts.escrow.try_borrow_data()?;
        let escrow = Escrow::load(&data)?;

        // Check if the escrow is valid
        let escrow_key = create_program_address(
            &[
                b"escrow",
                self.accounts.maker.key().as_ref(),
                &escrow.seed.to_le_bytes(),
                &escrow.bump,
            ], 
            &crate::ID
        )?;
        if escrow_key != *self.accounts.escrow.key() {
            return Err(ProgramError::InvalidAccountOwner);
        }
        
        let seed_binding = escrow.seed.to_le_bytes();
        let bump_binding = escrow.bump;
        let signer_seeds = &[
            b"escrow",
            self.accounts.maker.key().as_ref(),
            &seed_binding,
            &bump_binding,
        ];
        let signer = &[&signer_seeds[..]];
        
        let vault_account = TokenAccount::from_account_info(self.accounts.vault)?;
        let amount = vault_account.amount();
        
        // Transfer from the Vault to the Taker
        let transfer_instruction = transfer(
            self.accounts.vault.key(),
            self.accounts.taker_ata_a.key(),
            self.accounts.escrow.key(),
            &amount.to_le_bytes(),
        );
        invoke_signed(
            &transfer_instruction,
            &[
                self.accounts.token_program,
                self.accounts.vault,
                self.accounts.taker_ata_a,
                self.accounts.escrow,
            ],
            &[&signer],
        )?;

        // Close the Vault
        let close_instruction = close_account(
            self.accounts.vault.key(),
            self.accounts.maker.key(),
            self.accounts.escrow.key(),
        );
        invoke_signed(
            &close_instruction,
            &[
                self.accounts.token_program,
                self.accounts.vault,
                self.accounts.maker,
                self.accounts.escrow,
            ],
            &[&signer],
        )?;

        // Transfer from the Taker to the Maker
        let transfer_b_instruction = transfer(
            self.accounts.taker_ata_b.key(),
            self.accounts.maker_ata_b.key(),
            self.accounts.taker.key(),
            &escrow.receive.to_le_bytes(),
        );
        invoke(
            &transfer_b_instruction,
            &[
                self.accounts.token_program,
                self.accounts.taker_ata_b,
                self.accounts.maker_ata_b,
                self.accounts.taker,
            ],
        )?;

        // Close the Escrow
        drop(data);
        
        // Transfer lamports from escrow to taker (closing the account)
        let escrow_lamports = self.accounts.escrow.lamports();
        **self.accounts.escrow.try_borrow_mut_lamports()? -= escrow_lamports;
        **self.accounts.taker.try_borrow_mut_lamports()? += escrow_lamports;

        Ok(())
    }
}