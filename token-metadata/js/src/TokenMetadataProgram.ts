import fetch from 'cross-fetch';
import { PublicKey, Keypair } from '@solana/web3.js';
import { Program, Provider, Wallet, Idl, AnchorProvider, BN } from '@project-serum/anchor';
import {
  Metadata,
  MasterEditionV2,
  PROGRAM_ID as METADATA_PROGRAM_ID,
} from '@metaplex-foundation/mpl-token-metadata';
import {
  getAccount as getTokenAccount,
  getMint,
  Account as TokenAccount,
  Mint,
} from '@solana/spl-token';
import idl from '../../../target/idl/bpl_token_metadata.json';
import {
  Promo,
  PromoExtended,
  DataV2,
  MetadataJson,
  AdminSettings,
  PromoExtendeds,
  PromoGroup,
} from '.';
const camelcaseKeysDeep = require('camelcase-keys-deep');

export class TokenMetadataProgram {
  readonly PUBKEY: PublicKey;

  readonly SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID: PublicKey;
  readonly TOKEN_PROGRAM_ID: PublicKey;
  readonly TOKEN_METADATA_PROGRAM_ID: PublicKey;
  readonly MEMO_PROGRAM_ID: PublicKey;

  readonly ADMIN_PREFIX: string;
  readonly AUTHORITY_PREFIX: string;
  readonly METADATA_PREFIX: string;
  readonly EDITION_PREFIX: string;
  readonly PROMO_PREFIX: string;

  program: Program;
  payer: Wallet;

  constructor(provider: Provider) {
    this.PUBKEY = new PublicKey('CjSoZrc2DBZTv1UdoMx8fTcCpqEMXCyfm2EuTwy8yiGi');
    this.SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID = new PublicKey(
      'ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL',
    );
    this.TOKEN_PROGRAM_ID = new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA');
    this.TOKEN_METADATA_PROGRAM_ID = METADATA_PROGRAM_ID;
    this.MEMO_PROGRAM_ID = new PublicKey('MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr');

    this.ADMIN_PREFIX = 'admin';
    this.AUTHORITY_PREFIX = 'authority';
    this.METADATA_PREFIX = 'metadata';
    this.EDITION_PREFIX = 'edition';
    this.PROMO_PREFIX = 'promo';

    this.program = new Program(idl as Idl, this.PUBKEY, provider);
    const anchorProvider = this.program.provider as AnchorProvider;
    this.payer = anchorProvider.wallet as Wallet;
  }

  // To keep things straight with promo owner paying for transactions
  // initiated and signed for by users, always pass and explicit
  // reference to the payer into accounts.

  /**
   * Creates admin settings account
   *
   * @param platform  Payer of the transaction and initialization fees
   *
   * @return Address of the admin settings account
   */
  async createAdminSettings(
    platform: Keypair,
    createPromoLamports: number,
    burnPromoTokenLamports: number,
  ): Promise<PublicKey> {
    const adminSettings = this.findAdminAddress();
    const programData = this.findProgramDataAdress();

    await this.program.methods
      .createAdminSettings({
        platform: platform.publicKey,
        createPromoLamports: new BN(createPromoLamports),
        burnPromoTokenLamports: new BN(burnPromoTokenLamports),
      })
      .accounts({
        payer: platform.publicKey,
        // program: this.PUBKEY,
        // programData,
      })
      .signers([platform])
      .rpc();
    return adminSettings;
  }

  async createPromoGroup(
    seed: PublicKey,
    members: Array<PublicKey>,
    lamports: number,
    memo: string | null,
  ): Promise<PublicKey> {
    const group = this.findPromoGroupAddress(seed);

    const groupData: PromoGroup = {
      owner: this.payer.publicKey,
      seed,
      nonce: 0,
      members,
    };

    await this.program.methods
      .createPromoGroup(groupData, new BN(lamports), memo)
      .accounts({
        group,
        seed,
        memoProgram: this.MEMO_PROGRAM_ID,
      })
      .rpc();
    console.log('jingus', group);
    return group;
  }

  /**
   * Fetch platform address
   *
   * @return Address of the platform account
   */
  async fetchPlatformAddress(): Promise<PublicKey> {
    const adminSettings = this.findAdminAddress();
    const adminSettingsAccount = (await this.program.account.adminSettings.fetch(
      adminSettings,
    )) as AdminSettings;
    return adminSettingsAccount.platform;
  }

  /**
   * Create promo and associated metadata accounts
   *
   * @param payer         Payer of the transaction, will be the owner of the promo
   * @param platform      Platform address
   * @param metadataData  Metadata data
   * @param isMutable     Whether metadata is mutable
   * @param maxMint       Optional Max number of tokens to mint
   * @param maxRedeemable Optional max number of tokens that can used
   *
   * @return Address of promo account
   */
  async createPromo(
    metadataData: DataV2,
    isMutable: boolean,
    groupSeed: PublicKey,
    maxMint: number | null,
    maxBurn: number | null,
    platform: PublicKey,
    memo: string | null,
  ): Promise<PublicKey> {
    const mint = Keypair.generate();

    const metadata = this.findMetadataAddress(mint.publicKey);
    const group = this.findPromoGroupAddress(groupSeed);

    const promoData: Promo = {
      owner: group,
      mint: mint.publicKey,
      metadata,
      mintCount: 0,
      burnCount: 0,
      maxMint,
      maxBurn,
    };

    await this.program.methods
      .createPromo(promoData, metadataData, isMutable, memo)
      .accounts({
        group,
        mint: mint.publicKey,
        metadata,
        platform,
        metadataProgram: this.TOKEN_METADATA_PROGRAM_ID,
        memoProgram: this.MEMO_PROGRAM_ID,
      })
      .signers([mint])
      .rpc();

    return mint.publicKey;
  }

  /**
   * Mint promo token
   *
   * @param mint       Promo mint
   * @param platform   Address of platform account
   * @param promoOwner Keypair of promo owner
   *
   * @return Address of promo account
   */
  // no promo owner as signer for demo
  async mintPromoToken(
    mint: PublicKey,
    groupMember: Keypair,
    groupSeed: PublicKey,
    memo: string | null,
  ): Promise<PublicKey> {
    const tokenAccount = this.findAssociatedTokenAccountAddress(mint, this.payer.publicKey);
    const group = this.findPromoGroupAddress(groupSeed);

    await this.program.methods
      .mintPromoToken(memo)
      .accounts({
        payer: groupMember.publicKey,
        group,
        tokenOwner: this.payer.publicKey,
        mint,
        tokenAccount,
        memoProgram: this.MEMO_PROGRAM_ID,
      })
      .signers([groupMember])
      .rpc();

    return tokenAccount;
  }

  /**
   * Delegate promo token
   *
   * @param mint  Mint address
   *
   * @return Token account address
   */
  async delegatePromoToken(
    mint: PublicKey,
    delegate: PublicKey,
    groupSeed: PublicKey,
    memo: string | null,
  ): Promise<PublicKey> {
    const tokenAccount = this.findAssociatedTokenAccountAddress(mint, this.payer.publicKey);
    const group = this.findPromoGroupAddress(groupSeed);

    await this.program.methods
      .delegatePromoToken(memo)
      .accounts({
        delegate,
        group,
        tokenOwner: this.payer.publicKey,
        mint,
        tokenAccount,
        memoProgram: this.MEMO_PROGRAM_ID,
      })
      .rpc();

    return tokenAccount;
  }

  /**
   * Burn promo token.
   *
   * @param platform  Platform address
   * @param mint  Mint address
   *
   * @return Token account address
   */
  // no promo owner as signer for demo
  async burnDelegatedPromoToken(
    mint: PublicKey,
    tokenOwner: PublicKey,
    platform: PublicKey,
    groupSeed: PublicKey,
    memo: string | null,
  ): Promise<PublicKey> {
    const tokenAccount = this.findAssociatedTokenAccountAddress(mint, tokenOwner);
    const group = this.findPromoGroupAddress(groupSeed);

    await this.program.methods
      .burnDelegatedPromoToken(memo)
      .accounts({
        group,
        mint,
        platform,
        tokenAccount,
        memoProgram: this.MEMO_PROGRAM_ID,
      })
      .rpc();

    return tokenAccount;
  }

  /**
   * Sign memo
   *
   * @param memo  Memo
   *
   * @return      Signature
   */
  async signMemo(memo: string, signer: Keypair): Promise<string> {
    const tx = await this.program.methods
      .signMemo(memo)
      .accounts({
        signer: signer.publicKey,
        memoProgram: this.MEMO_PROGRAM_ID,
      })
      .signers([signer])
      .rpc();

    return tx;
  }

  /**
   * Create non-fungible
   *
   * @param memo  Memo
   *
   * @return      Signature
   */
  async createNonFungible(
    metadataData: DataV2,
    payer: Keypair,
  ): Promise<[string, PublicKey, PublicKey, PublicKey]> {
    const mint = Keypair.generate();
    const tokenAccount = this.findAssociatedTokenAccountAddress(mint.publicKey, payer.publicKey);
    const metadataAccount = this.findMetadataAddress(mint.publicKey);
    const editionAccount = this.findMasterEditionAccountAddress(mint.publicKey);

    const tx = await this.program.methods
      .createNonFungible(metadataData, true, null)
      .accounts({
        payer: payer.publicKey,
        mint: mint.publicKey,
        tokenAccount,
        metadataAccount,
        editionAccount,
        metadataProgram: this.TOKEN_METADATA_PROGRAM_ID,
        memoProgram: this.MEMO_PROGRAM_ID,
      })
      .signers([payer, mint])
      .rpc();

    return [tx, mint.publicKey, metadataAccount, editionAccount];
  }

  async getTokenAccount(address: PublicKey): Promise<TokenAccount> {
    return await getTokenAccount(this.program.provider.connection, address);
  }

  async getMintAccount(address: PublicKey): Promise<Mint> {
    return await getMint(this.program.provider.connection, address);
  }

  async getMetadataAccount(address: PublicKey): Promise<Metadata> {
    return await Metadata.fromAccountAddress(this.program.provider.connection, address);
  }

  async getMasterEditionAccount(address: PublicKey): Promise<MasterEditionV2> {
    return await MasterEditionV2.fromAccountAddress(this.program.provider.connection, address);
  }

  async getPromoExtended(mint: PublicKey): Promise<PromoExtended> {
    const [promo, metadata] = await Promise.all([
      this.createPromoAddress(mint),
      this.findMetadataAddress(mint),
    ]);

    const [promoAccount, mintAccount, metadataAccount] = (await Promise.all([
      this.program.account.promo.fetch(promo),
      this.getMintAccount(mint),
      this.getMetadataAccount(metadata),
    ])) as [Promo, Mint, Metadata];
    const metadataJson = camelcaseKeysDeep(
      await fetch(metadataAccount.data.uri).then((res) => {
        return res.json();
      }),
    ) as MetadataJson;
    return new PromoExtendedImpl(
      promo,
      promoAccount,
      mintAccount,
      metadata,
      metadataAccount,
      metadataJson,
    );
  }

  async updatePromoExtended(promoExtended: PromoExtended): Promise<PromoExtended> {
    const promoAccount = (await this.program.account.promo.fetch(promoExtended.publicKey)) as Promo;
    const mintAccount = await this.getMintAccount(promoExtended.mintAccount.address);
    return new PromoExtendedImpl(
      promoExtended.publicKey,
      promoAccount,
      mintAccount,
      promoExtended.metadata,
      promoExtended.metadataAccount,
      promoExtended.metadataJson,
    );
  }

  async updatePromoExtendeds(promoExtendeds: PromoExtendeds): Promise<PromoExtendeds> {
    const results = await Promise.all(
      Object.values(promoExtendeds).map((promoExtended) => this.updatePromoExtended(promoExtended)),
    );
    return results.reduce(
      (promoExtendedsNew, promoExtended) => (
        (promoExtendedsNew[promoExtended.mintAccount.address.toString()] = promoExtended),
        promoExtendedsNew
      ),
      {} as PromoExtendeds,
    );
  }

  async getPromoExtendeds(mints: PublicKey[]): Promise<PromoExtendeds> {
    const results = await Promise.all(mints.map((mint) => this.getPromoExtended(mint)));
    return results.reduce(
      (promoExtendeds, promoExtended) => (
        (promoExtendeds[promoExtended.mintAccount.address.toString()] = promoExtended),
        promoExtendeds
      ),
      {} as PromoExtendeds,
    );
  }

  findAssociatedTokenAccountAddress(mint: PublicKey, wallet: PublicKey): PublicKey {
    return PublicKey.findProgramAddressSync(
      [wallet.toBuffer(), this.TOKEN_PROGRAM_ID.toBuffer(), mint.toBuffer()],
      this.SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID,
    )[0];
  }

  findAdminAddress(): PublicKey {
    return PublicKey.findProgramAddressSync([Buffer.from(this.ADMIN_PREFIX)], this.PUBKEY)[0];
  }

  findAuthorityAddress(): PublicKey {
    return PublicKey.findProgramAddressSync([Buffer.from(this.AUTHORITY_PREFIX)], this.PUBKEY)[0];
  }

  findMetadataAddress(mint: PublicKey): PublicKey {
    return PublicKey.findProgramAddressSync(
      [
        Buffer.from(this.METADATA_PREFIX),
        this.TOKEN_METADATA_PROGRAM_ID.toBuffer(),
        mint.toBuffer(),
      ],
      this.TOKEN_METADATA_PROGRAM_ID,
    )[0];
  }

  findMasterEditionAccountAddress(mint: PublicKey): PublicKey {
    return PublicKey.findProgramAddressSync(
      [
        Buffer.from(this.METADATA_PREFIX),
        this.TOKEN_METADATA_PROGRAM_ID.toBuffer(),
        mint.toBuffer(),
        Buffer.from(this.EDITION_PREFIX),
      ],
      this.TOKEN_METADATA_PROGRAM_ID,
    )[0];
  }

  //   pub fn find_edition_account(mint: &Pubkey, edition_number: String) -> (Pubkey, u8) {
  //     Pubkey::find_program_address(
  //         &[
  //             PREFIX.as_bytes(),
  //             crate::id().as_ref(),
  //             mint.as_ref(),
  //             EDITION.as_bytes(),
  //             edition_number.as_bytes(),
  //         ],
  //         &crate::id(),
  //     )
  // }

  // pub fn find_master_edition_account(mint: &Pubkey) -> (Pubkey, u8) {
  //     Pubkey::find_program_address(
  //         &[
  //             PREFIX.as_bytes(),
  //             crate::id().as_ref(),
  //             mint.as_ref(),
  //             EDITION.as_bytes(),
  //         ],
  //         &crate::id(),
  //     )
  // }

  createPromoAddress(mint: PublicKey): PublicKey {
    return PublicKey.findProgramAddressSync(
      [Buffer.from(this.PROMO_PREFIX), mint.toBuffer()],
      this.PUBKEY,
    )[0];
  }

  findPromoGroupAddress(groupSeed: PublicKey): PublicKey {
    return PublicKey.findProgramAddressSync([groupSeed.toBuffer()], this.PUBKEY)[0];
  }

  findProgramDataAdress(): PublicKey {
    return PublicKey.findProgramAddressSync(
      [this.PUBKEY.toBytes()],
      new PublicKey('BPFLoaderUpgradeab1e11111111111111111111111'),
    )[0];
  }
}

export class PromoExtendedImpl implements PromoExtended {
  owner: PublicKey;
  mint: PublicKey;
  metadata: PublicKey;
  mintCount: number;
  burnCount: number;
  maxMint: number | null;
  maxBurn: number | null;
  publicKey: PublicKey;
  mintAccount: Mint;
  metadataAccount: Metadata;
  metadataJson: MetadataJson;

  constructor(
    promo: PublicKey,
    promoAccount: Promo,
    mintAccount: Mint,
    metadata: PublicKey,
    metadataAccount: Metadata,
    metadataJson: MetadataJson,
  ) {
    this.owner = promoAccount.owner;
    this.publicKey = promo;
    this.mint = mintAccount.address;
    this.metadata = metadata;
    this.mintAccount = mintAccount;
    this.metadataAccount = metadataAccount;
    this.metadataJson = metadataJson;
    this.mintCount = promoAccount.mintCount;
    this.burnCount = promoAccount.burnCount;
    this.maxMint = promoAccount.maxMint;
    this.maxBurn = promoAccount.maxBurn;
  }
}
