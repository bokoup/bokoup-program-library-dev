use crate::{state::Location, utils::create_memo, CreateLocation};

use anchor_lang::prelude::*;

impl<'info> CreateLocation<'info> {
    pub fn process(&mut self, data: Location, memo: Option<String>) -> Result<()> {
        msg!("Create location");

        *self.location = data;

        if let Some(memo) = memo {
            let account_infos = vec![self.owner.to_account_info(), self.payer.to_account_info()];
            create_memo(memo, account_infos)?;
        }
        Ok(())
    }
}
