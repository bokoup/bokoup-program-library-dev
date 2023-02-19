use crate::{state::Device, utils::create_memo, CreateDevice};

use anchor_lang::prelude::*;

impl<'info> CreateDevice<'info> {
    pub fn process(&mut self, data: Device, memo: Option<String>) -> Result<()> {
        msg!("Create device");

        *self.device = data;

        if let Some(memo) = memo {
            let account_infos = vec![self.payer.to_account_info()];
            create_memo(memo, account_infos)?;
        }
        Ok(())
    }
}
