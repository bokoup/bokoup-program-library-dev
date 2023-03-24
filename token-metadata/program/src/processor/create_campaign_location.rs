use crate::{state::CampaignLocation, utils::create_memo, CreateCampaignLocation};
use anchor_lang::prelude::*;

impl<'info> CreateCampaignLocation<'info> {
    pub fn process(&mut self, memo: Option<String>) -> Result<()> {
        msg!("Create campaign location");

        *self.campaign_location = CampaignLocation {
            campaign: self.campaign.key(),
            location: self.location.key(),
        };

        if let Some(memo) = memo {
            let account_infos = vec![self.owner.to_account_info(), self.payer.to_account_info()];
            create_memo(memo, account_infos)?;
        }

        Ok(())
    }
}
