import * as anchor from '@project-serum/anchor';
import { TokenMetadataProgram, AdminSettings, DataV2, PromoExtended, Memo } from '../src';
import { PublicKey, Keypair, Transaction, Connection } from '@solana/web3.js';
import chai = require('chai');
import chaiAsPromised = require('chai-as-promised');
const fs = require('fs');
chai.use(chaiAsPromised);
const expect = chai.expect;
import * as dotenv from 'dotenv';
import path from 'path';
dotenv.config({ path: path.resolve(__dirname, '../../../demo-web/.env') });

describe('promo', () => {
  const tokenOwnerProvider = anchor.AnchorProvider.env();
  // Configure the client to use the local cluster.
  // anchor.setProvider(provider);
  const tokenOwner = tokenOwnerProvider.wallet.publicKey;
  const tokenMetadataProgram = new TokenMetadataProgram(tokenOwnerProvider);

  const promoOwner = Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(fs.readFileSync('/keys/promo_owner-keypair.json'))),
  );

  const process = require("process");
  const url = process.env.ANCHOR_PROVIDER_URL;
  if (url === undefined) {
    throw new Error("ANCHOR_PROVIDER_URL is not defined");
  }
  const options = anchor.AnchorProvider.defaultOptions();
  const connection = new Connection(url, options.commitment);
  const promoOwnerWallet = new anchor.Wallet(promoOwner)

  const promoOwnerProvider = new anchor.AnchorProvider(
    connection,
    promoOwnerWallet,
    options
  )

  const tokenMetadataProgramPromoOwner = new TokenMetadataProgram(promoOwnerProvider);

  const platform = Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(process.env.REACT_APP_PLATFORM_KEYPAIR!)),
  );
  console.log('promoOwner: ', promoOwner.publicKey.toString());
  console.log('platform: ', platform.publicKey.toString());

  let adminSettings: PublicKey;
  let adminSettingsAccount: AdminSettings;
  let mint: PublicKey;
  let promoExtended: PromoExtended;

  it('funds accounts', async () => {
    const amount = 1_000_000_000;
    const transaction = new Transaction();
    const addresses = [platform.publicKey, promoOwner.publicKey];
    addresses.forEach((address) => {
      transaction.add(
        anchor.web3.SystemProgram.transfer({
          fromPubkey: tokenMetadataProgram.payer.publicKey,
          lamports: amount,
          toPubkey: address,
        }),
      );
    });
    await tokenOwnerProvider.sendAndConfirm(transaction);
    const accountInfos = await Promise.all(
      addresses.map((address) => tokenOwnerProvider.connection.getAccountInfo(address)),
    );
    accountInfos.map((account) => {
      expect(account!.lamports).to.equal(amount, 'Platform lamports incorrect.');
    });
  });

  it('creates admin settings', async () => {
    [adminSettings] = await tokenMetadataProgram.findAdminAddress();

    await tokenMetadataProgram.createAdminSettings(platform, 10_000_000, 1_000_000);

    adminSettingsAccount = (await tokenMetadataProgram.program.account.adminSettings.fetch(
      adminSettings,
    )) as AdminSettings;
    expect(adminSettingsAccount.platform.toString()).to.equal(
      platform.publicKey.toString(),
      'Admin platform incorrect.',
    );
  });

  it('Creates two promos', async () => {
    const metadataData1: DataV2 = {
      name: 'Promo 1',
      symbol: 'P1',
      uri: 'https://arweave.net/u27CJpMzXZnmrTwqXzHjXQnECxP0_iMzSjE-WMAec24',
      sellerFeeBasisPoints: 0,
      creators: null,
      collection: null,
      uses: null,
    };

    const metadataData2: DataV2 = {
      name: 'Promo 2',
      symbol: 'P2',
      uri: 'https://arweave.net/RK8RPpNWs1nVy0qqvKVAFcVQOxOltGMyDrJtt6SktTc',
      sellerFeeBasisPoints: 0,
      creators: null,
      collection: null,
      uses: null,
    };

    for (const metadataData of [metadataData1, metadataData2] as DataV2[]) {
      const platformStartAccountInfo =
        await tokenMetadataProgram.program.provider.connection.getAccountInfo(
          adminSettingsAccount.platform,
        );

      const maxMint = 1_000;
      const maxRedeem = 500;

      const memo: Memo = {
        reference: metadataData.symbol,
        memo: "Created new promo"
      };

      mint = await tokenMetadataProgramPromoOwner.createPromo(
        metadataData,
        true,
        maxMint,
        maxRedeem,
        adminSettingsAccount.platform,
        memo
      );

      promoExtended = await tokenMetadataProgram.getPromoExtended(mint);
      console.log('promoExtended: ', promoExtended);
      console.log('mintAddress: ', promoExtended.mintAccount.address.toString());

      const platformAccountInfo =
        await tokenMetadataProgram.program.provider.connection.getAccountInfo(
          adminSettingsAccount.platform,
        );
      if (platformStartAccountInfo !== null && platformAccountInfo !== null) {
        expect(platformAccountInfo.lamports).to.equal(
          platformStartAccountInfo.lamports + adminSettingsAccount.createPromoLamports.toNumber(),
          'Platform lamports incorrect.',
        );
      }
    }
  });

  it('Mints a promo token', async () => {
    const [tokenAccountAccount, mintAccount] = await tokenMetadataProgram
      .mintPromoToken(mint, promoOwner, null)
      .then((tokenAccount) =>
        Promise.all([
          tokenMetadataProgram.getTokenAccount(tokenAccount),
          tokenMetadataProgram.getMintAccount(promoExtended.mintAccount.address),
        ]),
      );

    promoExtended = await tokenMetadataProgram.getPromoExtended(mint);

    expect(Number(tokenAccountAccount.amount)).to.equal(1, 'Token account amount incorrect.');
    expect(Number(mintAccount.supply)).to.equal(1, 'Mint supply incorrect.');
    expect(promoExtended.mintCount).to.equal(1, 'Promo mints incorrect.');

    console.log('tokenAccountAccount: ', tokenAccountAccount);
    console.log('mintAccount: ', mintAccount);
  });

  it('Delegates a promo token', async () => {
    const memo: Memo = {
      reference: "myReference",
      memo: "delegated token"
    };

    const tokenAccountAccount = await tokenMetadataProgram
      .delegatePromoToken(mint, promoOwner, memo)
      .then((tokenAccount) => tokenMetadataProgram.getTokenAccount(tokenAccount));
    expect(Number(tokenAccountAccount.delegatedAmount)).to.equal(1, 'Delegated amount incorrect.');
    console.log('tokenAccountAccount: ', tokenAccountAccount);
  });

  it('Burns a delegated promo token', async () => {
    const platformStartAccountInfo =
      await tokenMetadataProgram.program.provider.connection.getAccountInfo(
        adminSettingsAccount.platform,
      );

    console.log("mint", mint);
    const memo: Memo = {
      reference: "myReference",
      memo: "burned delegated token"
    };

    const [tokenAccountAccount, mintAccount] = await tokenMetadataProgramPromoOwner
      .burnDelegatedPromoToken(mint, tokenOwner, platform.publicKey, memo)
      .then((tokenAccount) =>
        Promise.all([
          tokenMetadataProgram.getTokenAccount(tokenAccount),
          tokenMetadataProgram.getMintAccount(mint),
        ]),
      );

    promoExtended = await tokenMetadataProgram.getPromoExtended(mint);
    expect(Number(tokenAccountAccount.amount)).to.equal(0, 'Token account amount incorrect.');
    expect(Number(mintAccount.supply)).to.equal(0, 'Mint supply incorrect.');
    expect(promoExtended.burnCount).to.equal(1, 'Promo burns incorrect.');

    const platformAccountInfo =
      await tokenMetadataProgram.program.provider.connection.getAccountInfo(
        adminSettingsAccount.platform,
      );
    if (platformStartAccountInfo !== null && platformAccountInfo !== null) {
      expect(platformAccountInfo.lamports).to.equal(
        platformStartAccountInfo.lamports + adminSettingsAccount.burnPromoTokenLamports.toNumber(),
        'Platform lamports incorrect.',
      );
    }
  });

});
