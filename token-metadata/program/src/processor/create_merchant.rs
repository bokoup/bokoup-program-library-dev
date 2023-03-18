use crate::{state::Merchant, utils::create_memo, CreateMerchant};

use anchor_lang::prelude::*;

impl<'info> CreateMerchant<'info> {
    pub fn process(&mut self, data: Merchant, memo: Option<String>) -> Result<()> {
        msg!("Create merchant");

        *self.merchant = data;

        if let Some(memo) = memo {
            let account_infos = vec![self.owner.to_account_info(), self.payer.to_account_info()];
            create_memo(memo, account_infos)?;
        }
        Ok(())
    }
}
