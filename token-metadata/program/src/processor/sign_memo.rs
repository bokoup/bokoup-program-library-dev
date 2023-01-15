use crate::{
    utils::create_memo,
    SignMemo
};
use anchor_lang::prelude::*;

impl<'info> SignMemo<'info> {
    pub fn process(&mut self, memo: String) -> Result<()> {
        msg!("Sign memo");

        let account_infos = vec![self.payer.to_account_info()];
        create_memo(memo, account_infos)
    }
}
