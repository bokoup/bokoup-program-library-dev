use crate::utils::create_memo;
use crate::{error::ProgramError, MintPromoToken};
use anchor_lang::prelude::*;

impl<'info> MintPromoToken<'info> {
    pub fn process(&mut self, memo: Option<String>, authority_seeds: [&[u8]; 2]) -> Result<()> {
        msg!("Mint promo token");

        // Check to see if mint_count is still below max_mint.
        if let Some(max_mint) = self.promo.max_mint {
            if self.promo.mint_count >= max_mint {
                return Err(ProgramError::MaxMintExceeded.into());
            }
        }

        // Set the close authority to the program so it can close token
        // accounts when it burns the last token in them.
        // let set_authority_ctx = anchor_spl::token::SetAuthority {
        //     current_authority: self.token_owner.to_account_info(),
        //     account_or_mint: self.token_account.to_account_info(),
        // };

        // anchor_spl::token::set_authority(
        //     CpiContext::new(self.token_program.to_account_info(), set_authority_ctx),
        //     anchor_spl::token::spl_token::instruction::AuthorityType::CloseAccount,
        //     Some(self.authority.key()),
        // )?;

        let mint_to_ctx = anchor_spl::token::MintTo {
            mint: self.mint.to_account_info(),
            to: self.token_account.to_account_info(),
            authority: self.authority.to_account_info(),
        };

        anchor_spl::token::mint_to(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                mint_to_ctx,
                &[&authority_seeds],
            ),
            1,
        )?;

        if let Some(memo) = memo {
            let account_infos = vec![
                self.token_owner.to_account_info(),
                self.device_owner.to_account_info(),
            ];
            create_memo(memo.to_string(), account_infos)?;
        }

        self.promo.mint_count += 1;

        Ok(())
    }
}
