use pinocchio::{
    account_info::AccountInfo,
    ProgramError,
    pubkey::Pubkey,
    ProgramResult,
    program::invoke_signed,
};
use pinocchio_token::{
    instruction::{transfer, close_account},
    TokenAccount,
};
use pinocchio_system::{
    create_program_address,
};
use crate::{
    state::Escrow,
};

pub struct RefundAccounts<'a> {
    pub maker: &'a AccountInfo,
    pub escrow: &'a AccountInfo,
    pub mint_a: &'a AccountInfo,
    pub vault: &'a AccountInfo,
    pub maker_ata_a: &'a AccountInfo,
    pub system_program: &'a AccountInfo,
    pub token_program: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for RefundAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [maker, escrow, mint_a, vault, maker_ata_a, system_program, token_program, _] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Basic Accounts Checks
        if !maker.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if !escrow.is_writable || !vault.is_writable || !maker_ata_a.is_writable {
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
            vault,
            maker_ata_a,
            system_program,
            token_program,
        })
    }
}

pub struct Refund<'a> {
    pub accounts: RefundAccounts<'a>,
}

impl<'a> TryFrom<&'a [AccountInfo]> for Refund<'a> {
    type Error = ProgramError;
    
    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let accounts = RefundAccounts::try_from(accounts)?;

        Ok(Self {
            accounts,
        })
    }
}

impl<'a> Refund<'a> {
    pub const DISCRIMINATOR: u8 = 2;
    
    pub fn process(&mut self) -> ProgramResult {
        let data = self.accounts.escrow.try_borrow_data()?;
        let escrow = Escrow::load(&data)?;

        // Verify that the maker is the same as the one who created the escrow
        if *self.accounts.maker.key() != escrow.maker {
            return Err(ProgramError::InvalidAccountOwner);
        }

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
        
        // Transfer all tokens from the Vault to the Maker
        let transfer_instruction = transfer(
            self.accounts.vault.key(),
            self.accounts.maker_ata_a.key(),
            self.accounts.escrow.key(),
            &amount.to_le_bytes(),
        );
        invoke_signed(
            &transfer_instruction,
            &[
                self.accounts.token_program,
                self.accounts.vault,
                self.accounts.maker_ata_a,
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

        // Close the Escrow
        drop(data);
        
        // Transfer lamports from escrow to maker (closing the account)
        let escrow_lamports = self.accounts.escrow.lamports();
        **self.accounts.escrow.try_borrow_mut_lamports()? -= escrow_lamports;
        **self.accounts.maker.try_borrow_mut_lamports()? += escrow_lamports;

        Ok(())
    }
}