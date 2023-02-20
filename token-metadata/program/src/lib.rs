pub mod error;
/// Processors for each program instruction.
pub mod processor;
pub mod state;
pub mod utils;

use anchor_lang::prelude::*;
use anchor_spl::{
    self,
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use borsh::BorshDeserialize;
use state::{AdminSettings, Campaign, DataV2, Device, Location, Merchant, Promo};
use utils::{
    ADMIN_PREFIX, AUTHORITY_PREFIX, CAMPAIGN_PREFIX, DEVICE_PREFIX, LOCATIONS_CAPACITY,
    LOCATION_PREFIX, MAX_NAME_LENGTH, MAX_URI_LENGTH, MERCHANT_PREFIX, PROMO_PREFIX,
};

declare_id!("HB53jiCac5VtNdokJeibrfd1QJsyWWFe56M1TQUSKQfY");

// also update:
// Anchor.toml
// solana_server_setup.sh -> config.json
// TokenMetadataProgram.ts

/// Main module containing program instructions.
#[program]
pub mod bpl_token_metadata {
    use super::*;

    /// Creates AdminSettings account.
    pub fn create_admin_settings(
        ctx: Context<CreateAdminSettings>,
        data: AdminSettings,
    ) -> Result<()> {
        ctx.accounts.process(data)
    }

    /// Creates Merchant account
    pub fn create_merchant(
        ctx: Context<CreateMerchant>,
        data: Merchant,
        memo: Option<String>,
    ) -> Result<()> {
        ctx.accounts.process(data, memo)
    }

    /// Creates Location account
    pub fn create_location(
        ctx: Context<CreateLocation>,
        data: Location,
        memo: Option<String>,
    ) -> Result<()> {
        ctx.accounts.process(data, memo)
    }

    /// Creates Device account
    pub fn create_device(
        ctx: Context<CreateDevice>,
        data: Device,
        memo: Option<String>,
    ) -> Result<()> {
        ctx.accounts.process(data, memo)
    }

    /// Creates Group account used to grant transaction execution permissions to
    /// group members.
    pub fn create_campaign(
        ctx: Context<CreateCampaign>,
        data: Campaign,
        lamports: u64,
        memo: Option<String>,
    ) -> Result<()> {
        ctx.accounts.process(data, lamports, memo)
    }

    /// Creates Promo account and related mint and metadata accounts.
    pub fn create_promo(
        ctx: Context<CreatePromo>,
        promo_data: Promo,
        metadata_data: DataV2,
        is_mutable: bool,
        memo: Option<String>,
    ) -> Result<()> {
        let authority_seeds = [AUTHORITY_PREFIX.as_bytes(), &[ctx.bumps[AUTHORITY_PREFIX]]];

        ctx.accounts
            .process(promo_data, metadata_data, is_mutable, authority_seeds, memo)
    }

    /// Example of executing lamprts transfer from program derived account.
    pub fn transfer_cpi(ctx: Context<TransferCpi>, lamports: u64) -> Result<()> {
        ctx.accounts.process(lamports, ctx.bumps["campaign"])
    }

    /// Mints a promo token.
    pub fn mint_promo_token<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, MintPromoToken<'info>>,
        memo: Option<String>,
    ) -> Result<()> {
        let authority_seeds = [AUTHORITY_PREFIX.as_bytes(), &[ctx.bumps[AUTHORITY_PREFIX]]];
        ctx.accounts.process(memo, authority_seeds)
    }

    /// Delegates a promo token.
    pub fn delegate_promo_token<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, DelegatePromoToken<'info>>,
        memo: Option<String>,
    ) -> Result<()> {
        ctx.accounts.process(memo)
    }

    /// Burns a delegated promo token.
    pub fn burn_delegated_promo_token<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, BurnDelegatedPromoToken<'info>>,
        memo: Option<String>,
    ) -> Result<()> {
        ctx.accounts.process(memo)
    }

    /// Signs a memo.
    ///
    /// This could have just been done outside of the program, but doing it inside the program
    /// makes it easier to get the resulting transaction filtered by our indexer based on
    /// program address.
    pub fn sign_memo<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, SignMemo<'info>>,
        memo: String,
    ) -> Result<()> {
        ctx.accounts.process(memo)
    }

    /// Creates a non-fungible token. Will be used in the future with additional promo token form
    /// factors and to facilitate grouping promo tokens in collections.
    pub fn create_non_fungible(
        ctx: Context<CreateNonFungible>,
        data: DataV2,
        is_mutable: bool,
        max_supply: Option<u64>,
    ) -> Result<()> {
        let mint_authority_seeds = [AUTHORITY_PREFIX.as_bytes(), &[ctx.bumps[AUTHORITY_PREFIX]]];
        ctx.accounts
            .process(data, is_mutable, max_supply, mint_authority_seeds)
    }
}

/// Accounts related to creating [AdminSettings].
///
/// Admin settings sets the platform account to which protocol fees are remitted and sets the
/// platform fee levels for creating and burning a promo token. There are no platform fees
/// for minting or delegating tokens. Can only be created by the program authority.
///
/// Program derived address allows only one account to exist per program.
// TODO: uncomment prorgram data check when deploying to devnet
#[derive(Accounts)]
pub struct CreateAdminSettings<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(init_if_needed, seeds = [ADMIN_PREFIX.as_bytes()], bump, payer = payer, space = AdminSettings::LEN)]
    pub admin_settings: Account<'info, AdminSettings>,
    // #[account(constraint = program.programdata_address()? == Some(program_data.key()))]
    // pub program: Program<'info, crate::program::BplTokenMetadata>,
    // #[account(constraint = program_data.upgrade_authority_address == Some(payer.key()))]
    // pub program_data: Account<'info, ProgramData>,
    pub system_program: Program<'info, System>,
}

/// Accounts related to creating [Merchant].
///
#[derive(Accounts, Clone)]
#[instruction(merchant_data: Merchant)]
pub struct CreateMerchant<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: pubkey checked via seeds
    #[account(
        init,
        constraint = merchant_data.owner == payer.key(),
        constraint = merchant_data.name.len() <= MAX_NAME_LENGTH,
        constraint = merchant_data.uri.len() <= MAX_URI_LENGTH,
        seeds = [MERCHANT_PREFIX.as_bytes(), payer.key().as_ref()], bump,
        payer = payer,
        space = Merchant::LEN
    )]
    pub merchant: Account<'info, Merchant>,
    pub memo_program: Program<'info, SplMemo>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

/// Accounts related to creating [Location].
///
#[derive(Accounts, Clone)]
#[instruction(data: Location)]
pub struct CreateLocation<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: pubkey checked via seeds
    #[account(
        constraint = merchant.owner == payer.key(),
        seeds = [MERCHANT_PREFIX.as_bytes(), payer.key().as_ref()], bump,
    )]
    pub merchant: Account<'info, Merchant>,
    #[account(
        init,
        constraint = data.merchant == merchant.key(),
        constraint = data.name.len() <= MAX_NAME_LENGTH,
        constraint = data.uri.len() <= MAX_URI_LENGTH,
        seeds = [LOCATION_PREFIX.as_bytes(), merchant.key().as_ref(), data.name.as_bytes()], bump,
        payer = payer,
        space = Location::LEN
    )]
    pub location: Account<'info, Location>,
    pub memo_program: Program<'info, SplMemo>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

/// Accounts related to creating [Merchant].
///
#[derive(Accounts, Clone)]
#[instruction(data: Device)]
pub struct CreateDevice<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        constraint = merchant.owner == payer.key(),
        seeds = [MERCHANT_PREFIX.as_bytes(), payer.key().as_ref()], bump,
    )]
    pub merchant: Account<'info, Merchant>,
    #[account(constraint = location.merchant == merchant.key())]
    pub location: Account<'info, Location>,
    #[account(
        init,
        constraint = data.location == location.key(),
        constraint = data.name.len() <= MAX_NAME_LENGTH,
        constraint = data.uri.len() <= MAX_URI_LENGTH,
        seeds = [DEVICE_PREFIX.as_bytes(), location.key().as_ref(), data.name.as_bytes()], bump,
        payer = payer,
        space = Device::LEN
    )]
    pub device: Account<'info, Device>,
    pub memo_program: Program<'info, SplMemo>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

/// Accounts related to creating a [Group].
///
/// [Group] account owns [Promo] account. [Group] has an owner and members. Members can sign on behalf of the
/// group to mint tokens. Can be set with a members_capacity value of up to 255. Checks to make sure that
/// members_capacity is at least as big as the initial number of members. Also requires that the owner of the group
/// be the payer of the transaction and that the owner be include in the members.
///
/// [Group] has a program derived address so that permissions to it can be managed by the program. The seed is based
/// on a public key passed in to the program.
#[derive(Accounts)]
#[instruction(data: Campaign)]
pub struct CreateCampaign<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(constraint = payer.key() == merchant.owner)]
    pub merchant: Account<'info, Merchant>,
    #[account(
        init,
        constraint = data.merchant == merchant.key(),
        constraint = data.locations.len() <= LOCATIONS_CAPACITY as usize,
        constraint = data.name.len() <= MAX_NAME_LENGTH,
        seeds = [CAMPAIGN_PREFIX.as_bytes(), merchant.key().as_ref(), data.name.as_bytes()], bump,
        payer = payer,
        space = Campaign::LEN
    )]
    pub campaign: Account<'info, Campaign>,
    pub memo_program: Program<'info, SplMemo>,
    pub system_program: Program<'info, System>,
}

/// Accounts related to creating a [Promo].
///
/// Currently set up to have the signer pay network fees. Only the group owner is able to create
/// create promos. Both mint and metadata authorities are retained by the program in order to facilitate
/// customers transacting without fees to the greatest extent possible and to centralize permissions.
///
/// Program derived address allows only one promo account to exist per mint.
///
/// Checks to make sure that promo owner property is equal to the group account address. Also checks
/// to make sure the platform address is the one contained in the admin setings account.
///
/// It may be desirable to have members of merchants' groups create promos in the future.
/// To avoid having multiple members each having wallets with crypto balances in them determine
/// whether a pda can pay network fees so the group account can pay them when authorized by
/// members' signatures.
///
/// The fee specified in the `create_promo_lamports` property of the [AdminSettings] account
/// is remitted from the [Group] specified in the `owner` property of the [Promo] is transferred
/// from the [Group] lamports to the account specified in the `platform` property of the [AdminSettings]
/// account.
#[derive(Accounts, Clone)]
#[instruction(promo_data: Promo, metadata_data: DataV2)]
pub struct CreatePromo<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, constraint = payer.key() == merchant.owner)]
    pub merchant: Account<'info, Merchant>,
    #[account(mut,
        constraint = merchant.key() == campaign.merchant,
        constraint = campaign.key() == promo_data.campaign,
    )]
    pub campaign: Account<'info, Campaign>,
    #[account(init, payer = payer, mint::decimals = 0, mint::authority = authority, mint::freeze_authority = authority)]
    pub mint: Account<'info, Mint>,
    /// CHECK: Created via cpi
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    /// Metadata authority as pda to enable program to authorize edits
    /// CHECK: pubkey checked via seeds
    #[account(seeds = [AUTHORITY_PREFIX.as_bytes()], bump)]
    pub authority: UncheckedAccount<'info>,
    #[account(init, payer = payer,
        seeds = [PROMO_PREFIX.as_bytes(), mint.key().as_ref()], bump,
        space = Promo::LEN)]
    pub promo: Account<'info, Promo>,
    /// CHECK: pubkey checked via constraint
    #[account(mut,
        constraint = platform.key() == admin_settings.platform
    )]
    pub platform: UncheckedAccount<'info>,
    #[account(seeds = [ADMIN_PREFIX.as_bytes()], bump)]
    pub admin_settings: Box<Account<'info, AdminSettings>>,
    pub metadata_program: Program<'info, TokenMetadata>,
    pub token_program: Program<'info, Token>,
    pub memo_program: Program<'info, SplMemo>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

/// Example of executing lamprts transfer from program derived account.
#[derive(Accounts, Clone)]
#[instruction(lamports: u64)]
pub struct TransferCpi<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(constraint = payer.key() == merchant.owner)]
    pub merchant: Account<'info, Merchant>,
    #[account(constraint = campaign.locations.contains(&location.key()))]
    pub location: Account<'info, Location>,
    #[account(
        constraint = device.owner == payer.key(),
        constraint = device.location == location.key(),
    )]
    pub device: Account<'info, Device>,
    #[account(mut)]
    pub campaign: Account<'info, Campaign>,
    /// CHECK: checked in contraints
    #[account(mut, constraint = platform.key() == admin_settings.platform)]
    pub platform: UncheckedAccount<'info>,
    #[account(seeds = [ADMIN_PREFIX.as_bytes()], bump)]
    pub admin_settings: Box<Account<'info, AdminSettings>>,
    pub system_program: Program<'info, System>,
}

/// Accounts related to minting a promo token.
///
/// Requires a signature from the owner of a device with a location included in the campaign
/// as well as from the recipient (as a matter of responsible token issuance,
/// bokoup always gets a recipient's consent before minting them any tokens).
///
/// Creates a token account for the recipient if one does not already exist. Authority over the
/// token account is retained with the token owner. (All tokens are currently freely transferrable
/// by token owners. Future versions may retain authority with the program to facilitate execution
/// to enforce transfer restrictions).
///
/// No platform fees result from minting a token.
#[derive(Accounts, Clone)]
pub struct MintPromoToken<'info> {
    #[account(mut, constraint = device.owner == payer.key())]
    pub payer: Signer<'info>,
    #[account(mut, constraint = campaign.locations.contains(&location.key()))]
    pub location: Account<'info, Location>,
    #[account(
        constraint = device.location == location.key(),
        constraint = device.owner == payer.key()
    )]
    pub device: Account<'info, Device>,
    #[account(mut,
        constraint = campaign.locations.contains(&location.key()),
        constraint = campaign.key() == promo.campaign,
    )]
    pub campaign: Box<Account<'info, Campaign>>,
    #[account(mut)]
    pub token_owner: Signer<'info>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    /// CHECK: pubkey checked via seeds
    #[account(seeds = [AUTHORITY_PREFIX.as_bytes()], bump)]
    pub authority: UncheckedAccount<'info>,
    #[account(mut, seeds = [PROMO_PREFIX.as_bytes(), mint.key().as_ref()], bump)]
    pub promo: Account<'info, Promo>,
    #[account(init_if_needed, payer = payer, associated_token::mint = mint, associated_token::authority = token_owner)]
    pub token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub memo_program: Program<'info, SplMemo>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

/// Accounts related to the delegation of a promo token.
///
/// Delegates a token to a device owner.
///
/// Requires a signature from the owner of a device with a location included in the campaign
/// as well as from the recipient (as a matter of responsible token issuance,
/// bokoup always gets a recipient's consent before minting them any tokens).
///
/// Requires signature from token owner as the authority of the token account.
///
/// No platform fees result from delegating a token.
#[derive(Accounts, Clone)]
pub struct DelegatePromoToken<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: checked via device constraints
    pub device_owner: UncheckedAccount<'info>,
    #[account(mut, constraint = campaign.locations.contains(&location.key()))]
    pub location: Account<'info, Location>,
    #[account(
        constraint = device.location == location.key(),
        constraint = device.owner == device_owner.key()
    )]
    pub device: Account<'info, Device>,
    #[account(mut,
        constraint = campaign.locations.contains(&location.key()),
        constraint = campaign.key() == promo.campaign,
    )]
    pub campaign: Box<Account<'info, Campaign>>,
    #[account(mut)]
    pub token_owner: Signer<'info>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(mut, seeds = [PROMO_PREFIX.as_bytes(), mint.key().as_ref()], bump)]
    pub promo: Account<'info, Promo>,
    #[account(mut,
        constraint = token_owner.key() == token_account.owner,
        constraint = mint.key() == token_account.mint
    )]
    pub token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub memo_program: Program<'info, SplMemo>,
    pub system_program: Program<'info, System>,
}

/// Accounts related to the burning of a delegated promo token.
///
/// Checks to make sure signer is a member of group specified in owner property of
/// promo in order to execute transaction to transfer lamports from group to platform
/// to pay `burn_promo_token_lamports`.
///
/// The fee specified in the `burn_promo_token_lamports` property of the [AdminSettings] account
/// is transferred from the [Group] specified in the `owner` property of the [Promo] from the
/// lamports of the [Group] account to the account specified in the `platform` property of the [AdminSettings]
/// account.
#[derive(Accounts, Clone)]
pub struct BurnDelegatedPromoToken<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, constraint = campaign.locations.contains(&location.key()))]
    pub location: Account<'info, Location>,
    #[account(
        constraint = device.location == location.key(),
        constraint = device.owner == payer.key()
    )]
    pub device: Box<Account<'info, Device>>,
    #[account(mut,
        constraint = campaign.locations.contains(&location.key()),
        constraint = campaign.key() == promo.campaign,
    )]
    pub campaign: Box<Account<'info, Campaign>>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    /// CHECK: pubkey checked via spl token program instruction
    #[account(seeds = [AUTHORITY_PREFIX.as_bytes()], bump)]
    pub authority: UncheckedAccount<'info>,
    #[account(mut, seeds = [PROMO_PREFIX.as_bytes(), mint.key().as_ref()], bump)]
    pub promo: Account<'info, Promo>,
    /// CHECK: pubkey checked via constraint
    #[account(mut, constraint = platform.key() == admin_settings.platform)]
    pub platform: UncheckedAccount<'info>,
    #[account(seeds = [ADMIN_PREFIX.as_bytes()], bump)]
    pub admin_settings: Account<'info, AdminSettings>,
    #[account(mut,
        constraint = token_account.mint == mint.key(),
        constraint = token_account.delegate.unwrap() == payer.key(),
        constraint = token_account.delegated_amount > 0,
    )]
    pub token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub memo_program: Program<'info, SplMemo>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

/// Account related to creation of non-fungibles - not yet implemented.
#[derive(Accounts, Clone)]
pub struct CreateNonFungible<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: pubkey checked via seeds
    #[account(seeds = [AUTHORITY_PREFIX.as_bytes()], bump)]
    pub authority: UncheckedAccount<'info>,
    #[account(init, payer = payer, mint::decimals = 0, mint::authority = authority, mint::freeze_authority = authority)]
    pub mint: Account<'info, Mint>,
    #[account(init, payer = payer, associated_token::mint = mint, associated_token::authority = payer)]
    pub token_account: Account<'info, TokenAccount>,
    /// CHECK: checked via cpi
    #[account(mut)]
    pub metadata_account: UncheckedAccount<'info>,
    /// CHECK: checked via cpi
    #[account(mut)]
    pub edition_account: UncheckedAccount<'info>,
    pub metadata_program: Program<'info, TokenMetadata>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

/// Accounts related to creation of token [Metadata].
#[derive(Accounts, Clone)]
pub struct CreateMetaData<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: checked via cpi
    pub metadata_account: UncheckedAccount<'info>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    /// CHECK: checked via cpi
    #[account(mut)]
    pub mint_authority: UncheckedAccount<'info>,
    /// CHECK: checked via cpi
    pub metadata_authority: UncheckedAccount<'info>,
    pub metadata_program: Program<'info, TokenMetadata>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

/// Accounts related to creation of token [Metadata].
#[derive(Accounts, Clone)]
pub struct SignMemo<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub signer: Signer<'info>,
    pub memo_program: Program<'info, SplMemo>,
    pub system_program: Program<'info, System>,
}

impl<'info> From<CreatePromo<'info>> for CreateMetaData<'info> {
    fn from(item: CreatePromo<'info>) -> Self {
        CreateMetaData {
            payer: item.payer,
            metadata_account: item.metadata,
            mint: item.mint,
            mint_authority: item.authority.clone(),
            metadata_authority: item.authority,
            metadata_program: item.metadata_program,
            rent: item.rent,
            system_program: item.system_program,
        }
    }
}

impl<'info> From<CreateNonFungible<'info>> for CreateMetaData<'info> {
    fn from(item: CreateNonFungible<'info>) -> Self {
        CreateMetaData {
            payer: item.payer,
            metadata_account: item.metadata_account,
            mint: item.mint,
            mint_authority: item.authority.clone(),
            metadata_authority: item.authority,
            metadata_program: item.metadata_program,
            rent: item.rent,
            system_program: item.system_program,
        }
    }
}

#[derive(Accounts, Clone)]
pub struct TransferSol<'info> {
    /// CHECK: unchecked
    #[account(mut)]
    pub payer: AccountInfo<'info>,
    /// CHECK: unchecked
    #[account(mut)]
    pub to: AccountInfo<'info>,
}

#[derive(Clone)]
pub struct TokenMetadata;

impl anchor_lang::Id for TokenMetadata {
    fn id() -> Pubkey {
        mpl_token_metadata::ID
    }
}

#[derive(Clone)]
pub struct SplMemo;

impl anchor_lang::Id for SplMemo {
    fn id() -> Pubkey {
        spl_memo::ID
    }
}

#[derive(AnchorDeserialize, Clone, Debug)]
pub struct Metadata(mpl_token_metadata::state::Metadata);

impl Metadata {
    pub const LEN: usize = mpl_token_metadata::state::MAX_METADATA_LEN;
}

impl anchor_lang::AccountDeserialize for Metadata {
    fn try_deserialize_unchecked(buf: &mut &[u8]) -> Result<Self> {
        mpl_token_metadata::utils::try_from_slice_checked(
            buf,
            mpl_token_metadata::state::Key::MetadataV1,
            mpl_token_metadata::state::MAX_METADATA_LEN,
        )
        .map_err(Into::into)
    }
}

impl anchor_lang::AccountSerialize for Metadata {}

impl anchor_lang::Owner for Metadata {
    fn owner() -> Pubkey {
        mpl_token_metadata::ID
    }
}

impl core::ops::Deref for Metadata {
    type Target = mpl_token_metadata::state::Metadata;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
