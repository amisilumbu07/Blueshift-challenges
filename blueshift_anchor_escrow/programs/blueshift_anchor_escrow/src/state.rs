use anchor_lang::prelude::*;

#[derive(InitSpace)]
#[account]
pub struct Escrow {
    pub seed: u64,
    pub maker: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub receive: u64,
    pub bump: u8,
}

impl Escrow {
    pub const DISCRIMINATOR: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
}