use pinocchio::AccountView;

#[repr(C)]
pub struct Escrow {
    pub seed: u64,        // Random seed for PDA derivation
    pub maker: AccountView,    // Creator of escrow
    pub mint_a: AccountView,   // Token being deposited
    pub mint_b: AccountView,   // Token being requested
    pub receive: u64,     // Amount of token B wanted
    pub bump: [u8;1]      // PDA bump seed
}