use crate::{utils::transfer_sol, utils::CAMPAIGN_PREFIX, TransferCpi, TransferSol};
use anchor_lang::prelude::*;

impl<'info> TransferCpi<'info> {
    pub fn process(&mut self, lamports: u64, nonce: u8) -> Result<()> {
        msg!("Transfer cpi");
        let merchant = self.merchant.key();
        let seeds = [
            CAMPAIGN_PREFIX.as_bytes(),
            merchant.as_ref(),
            self.campaign.name.as_bytes(),
            &[nonce],
        ];

        msg!("seeds: {:?}", seeds);

        transfer_sol(
            CpiContext::new_with_signer(
                self.system_program.to_account_info(),
                TransferSol {
                    payer: self.campaign.to_account_info(),
                    to: self.platform.to_account_info(),
                },
                &[&seeds[..]],
            ),
            lamports,
        )?;

        Ok(())
    }
}
