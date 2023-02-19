use std::str::FromStr;

use crate::{CreateMetaData, CreateNonFungible, TransferSol};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::get_associated_token_address;
use mpl_token_metadata::{pda::find_metadata_account, state::DataV2};

pub const ADMIN_PREFIX: &str = "admin";
pub const AUTHORITY_PREFIX: &str = "authority";
pub const MERCHANT_PREFIX: &str = "merchant";
pub const LOCATION_PREFIX: &str = "location";
pub const DEVICE_PREFIX: &str = "device";
pub const CAMPAIGN_PREFIX: &str = "campaign";
pub const PROMO_PREFIX: &str = "promo";
pub const LOCATIONS_CAPACITY: usize = 10;
pub const MAX_NAME_LENGTH: usize = 64;
pub const MAX_URI_LENGTH: usize = 200;

pub fn transfer_sol<'a, 'b, 'c, 'info>(
    ctx: CpiContext<'a, 'b, 'c, 'info, TransferSol<'info>>,
    lamports: u64,
) -> Result<()> {
    let ix = anchor_lang::solana_program::system_instruction::transfer(
        &ctx.accounts.payer.key(),
        &ctx.accounts.to.key(),
        lamports,
    );
    anchor_lang::solana_program::program::invoke_signed(
        &ix,
        &[
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.to.to_account_info(),
        ],
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

pub fn create_metadata_accounts_v2<'a, 'b, 'c, 'info>(
    ctx: CpiContext<'a, 'b, 'c, 'info, CreateMetaData<'info>>,
    update_authority_is_signer: bool,
    is_mutable: bool,
    data: DataV2,
) -> Result<()> {
    let ix = mpl_token_metadata::instruction::create_metadata_accounts_v2(
        mpl_token_metadata::ID.clone(),
        ctx.accounts.metadata_account.to_account_info().key(),
        ctx.accounts.mint.to_account_info().key(),
        ctx.accounts.mint_authority.key(),
        ctx.accounts.payer.key(),
        ctx.accounts.metadata_authority.key(),
        data.name,
        data.symbol,
        data.uri,
        data.creators,
        data.seller_fee_basis_points,
        update_authority_is_signer,
        is_mutable,
        data.collection,
        data.uses,
    );
    anchor_lang::solana_program::program::invoke_signed(
        &ix,
        &[
            ctx.accounts.metadata_account.to_account_info(),
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.mint_authority.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.metadata_authority.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.rent.to_account_info(),
        ],
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

pub fn create_master_edition_v3<'a, 'b, 'c, 'info>(
    ctx: CpiContext<'a, 'b, 'c, 'info, CreateNonFungible<'info>>,
    max_supply: Option<u64>,
) -> Result<()> {
    let ix = mpl_token_metadata::instruction::create_master_edition_v3(
        mpl_token_metadata::ID.clone(),
        ctx.accounts.edition_account.key.clone(),
        ctx.accounts.mint.to_account_info().key(),
        ctx.accounts.authority.key.clone(),
        ctx.accounts.authority.key.clone(),
        ctx.accounts.metadata_account.key(),
        ctx.accounts.payer.key().clone(),
        max_supply,
    );
    anchor_lang::solana_program::program::invoke_signed(
        &ix,
        &[
            ctx.accounts.edition_account.to_account_info(),
            ctx.accounts.metadata_account.to_account_info(),
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.authority.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.authority.to_account_info(),
            ctx.accounts.rent.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
        ],
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

pub fn create_memo(memo: String, account_infos: Vec<AccountInfo>) -> Result<()> {
    let signer_pubkeys: Vec<&Pubkey> = account_infos
        .iter()
        .filter_map(|a| if a.is_signer { Some(a.key) } else { None })
        .collect();
    let ix = spl_memo::build_memo(memo.as_bytes(), &signer_pubkeys);
    anchor_lang::solana_program::program::invoke(&ix, &account_infos).map_err(Into::into)
}

pub fn find_associated_token_address(wallet: &Pubkey, mint: &Pubkey) -> Pubkey {
    get_associated_token_address(wallet, mint)
}

pub fn find_admin_address() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[ADMIN_PREFIX.as_bytes()], &crate::id())
}

pub fn find_authority_address() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[AUTHORITY_PREFIX.as_bytes()], &crate::id())
}

pub fn find_merchant_address(owner: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[MERCHANT_PREFIX.as_bytes(), owner.as_ref()], &crate::id())
}

pub fn find_location_address(merchant_owner: &Pubkey, name: &str) -> (Pubkey, u8) {
    let merchant = find_merchant_address(&merchant_owner).0;
    Pubkey::find_program_address(
        &[
            LOCATION_PREFIX.as_bytes(),
            merchant.as_ref(),
            name.as_bytes(),
        ],
        &crate::id(),
    )
}

pub fn find_device_address(location: &Pubkey, name: &str) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[DEVICE_PREFIX.as_bytes(), location.as_ref(), name.as_bytes()],
        &crate::id(),
    )
}

pub fn find_campaign_address(merchant: &Pubkey, name: &str) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            CAMPAIGN_PREFIX.as_bytes(),
            merchant.as_ref(),
            name.as_bytes(),
        ],
        &crate::id(),
    )
}

pub fn find_promo_address(mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[PROMO_PREFIX.as_bytes(), mint.as_ref()], &crate::id())
}

pub fn find_metadata_address(mint: &Pubkey) -> (Pubkey, u8) {
    find_metadata_account(mint)
}

pub fn find_program_data_address() -> Pubkey {
    Pubkey::find_program_address(
        &[&crate::id().as_ref()],
        &Pubkey::from_str("BPFLoaderUpgradeab1e11111111111111111111111").unwrap(),
    )
    .0
}

/// Pads the string to the desired size with `0u8`s.
/// NOTE: it is assumed that the string's size is never larger than the given size.
pub fn puffed_out_string(s: &str, size: usize) -> String {
    let mut array_of_zeroes = vec![];
    let puff_amount = size - s.len();
    while array_of_zeroes.len() < puff_amount {
        array_of_zeroes.push(0u8);
    }
    s.to_owned() + std::str::from_utf8(&array_of_zeroes).unwrap()
}
