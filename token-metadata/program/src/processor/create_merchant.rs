use crate::{
    state::{Campaign, Device, Location, Merchant},
    utils::{create_memo, transfer_sol},
    CreateMerchant, TransferSol,
};

use anchor_lang::prelude::*;

impl<'info> CreateMerchant<'info> {
    pub fn process(
        &mut self,
        merchant_data: Merchant,
        location_data: Location,
        device_data: Device,
        campaign_data: Campaign,
        lamports: u64,
        memo: Option<String>,
    ) -> Result<()> {
        msg!("Create merchant");

        *self.merchant = merchant_data;
        *self.location = location_data;
        *self.device = device_data;
        *self.campaign = campaign_data;

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
