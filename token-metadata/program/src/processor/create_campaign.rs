use crate::{
    state::Campaign,
    utils::{create_memo, transfer_sol},
    CreateCampaign, TransferSol,
};
use anchor_lang::prelude::*;

impl<'info> CreateCampaign<'info> {
    pub fn process(&mut self, data: Campaign, lamports: u64, memo: Option<String>) -> Result<()> {
        msg!("Create campaign");

        *self.campaign = data;

        transfer_sol(
            CpiContext::new(
                self.system_program.to_account_info(),
                TransferSol {
                    payer: self.payer.to_account_info(),
                    to: self.campaign.to_account_info(),
                },
            ),
            lamports,
        )?;

        if let Some(memo) = memo {
            let account_infos = vec![self.payer.to_account_info()];
            create_memo(memo, account_infos)?;
        }

        Ok(())
    }
}
