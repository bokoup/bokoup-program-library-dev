import * as anchor from '@project-serum/anchor';
import {
  ListingReceipt,
  BidReceipt,
  PurchaseReceipt,
} from '@metaplex-foundation/mpl-auction-house';
import { getAccount } from '@solana/spl-token';
import {
  TokenMetadataProgram,
  AdminSettings,
  DataV2,
  PromoExtended,
  AuctionHouseProgram,
  Merchant,
  CampaignLocation,
} from '../src';
import { PublicKey, Keypair, Transaction, Connection } from '@solana/web3.js';
import chai = require('chai');
import chaiAsPromised = require('chai-as-promised');
chai.use(chaiAsPromised);
const expect = chai.expect;
import * as dotenv from 'dotenv';
import path from 'path';
dotenv.config({ path: path.resolve(__dirname, '../../../.env') });
const process = require('process');

describe('promo', () => {
  const tokenOwnerProvider = anchor.AnchorProvider.env();
  // Configure the client to use the local cluster.
  // anchor.setProvider(provider);
  const tokenOwner = tokenOwnerProvider.wallet.publicKey;
  const tokenMetadataProgram = new TokenMetadataProgram(tokenOwnerProvider);

  // new Uint8Array(JSON.parse(fs.readFileSync('/keys/promo_owner-keypair.json'))),
  const merchantOwner = Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(process.env.MERCHANT_OWNER_KEYPAIR)),
  );

  console.log(merchantOwner.publicKey);

  const url = process.env.ANCHOR_PROVIDER_URL;
  if (url === undefined) {
    throw new Error('ANCHOR_PROVIDER_URL is not defined');
  }
  const options = anchor.AnchorProvider.defaultOptions();
  const connection = new Connection(url, options.commitment);
  const merchantOwnerWallet = new anchor.Wallet(merchantOwner);

  const merchantOwnerProvider = new anchor.AnchorProvider(connection, merchantOwnerWallet, options);

  const tokenMetadataProgramMerchantOwner = new TokenMetadataProgram(merchantOwnerProvider);

  const platformSigner = Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(process.env.PLATFORM_SIGNER_KEYPAIR)),
  );

  const buyer = Keypair.generate();

  const auctionHouseProgram = new AuctionHouseProgram(connection);
  let auctionHouse: PublicKey;

  const platform = Keypair.fromSecretKey(new Uint8Array(JSON.parse(process.env.PLATFORM_KEYPAIR)));
  console.log('merchantOwner: ', merchantOwner.publicKey.toString());
  console.log('platform: ', platform.publicKey.toString());
  console.log('platformSigner: ', platformSigner.publicKey.toString());

  let adminSettings: PublicKey;
  let adminSettingsAccount: AdminSettings;
  let mint: PublicKey;
  let promoExtended: PromoExtended;

  let merchant: PublicKey;
  let location: PublicKey;
  let device: PublicKey;
  let campaign: PublicKey;
  let campaignLocation: PublicKey;

  const deviceOwner = Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(process.env.DEVICE_OWNER_KEYPAIR)),
  );

  const tokenMetadataProgramDeviceOwner = new TokenMetadataProgram(
    new anchor.AnchorProvider(connection, new anchor.Wallet(deviceOwner), options),
  );

  it('funds accounts', async () => {
    const amount = 1_000_000_000;
    const transaction = new Transaction();
    const addresses = [
      platform.publicKey,
      merchantOwner.publicKey,
      deviceOwner.publicKey,
      platformSigner.publicKey,
    ];
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
      if (account != null) {
        expect(account.lamports).to.equal(amount, 'Platform lamports incorrect.');
      }
    });
  });

  it('creates admin settings', async () => {
    adminSettings = tokenMetadataProgram.findAdminAddress();

    await tokenMetadataProgram.createAdminSettings(platform, 10_000_000, 1_000_000);

    adminSettingsAccount = (await tokenMetadataProgram.program.account.adminSettings.fetch(
      adminSettings,
    )) as AdminSettings;
    expect(adminSettingsAccount.platform.toString()).to.equal(
      platform.publicKey.toString(),
      'Admin platform incorrect.',
    );
  });

  it('creates merchant', async () => {
    const merchantData: Merchant = {
      owner: merchantOwner.publicKey,
      name: 'Test Merchant',
      uri: 'https://merchant.example.com',
      active: true,
    };

    const memo = 'Created a new merchant';

    let _ = '';
    [_, merchant] = await tokenMetadataProgramMerchantOwner.createMerchant(
      merchantData,
      platformSigner,
      memo,
    );

    const merchantAccount = await tokenMetadataProgram.program.account.merchant.fetch(merchant);

    console.log(merchantAccount);
  });

  it('creates location', async () => {
    const name = 'Test Location';
    const uri = 'https://location.example.com';
    const memo = 'Created a new location';

    let _ = '';
    [_, location] = await tokenMetadataProgramMerchantOwner.createLocation(
      platformSigner,
      name,
      uri,
      memo,
    );

    const account = await tokenMetadataProgram.program.account.location.fetch(location);

    expect(account.name).to.equal(name);
  });

  it('creates device', async () => {
    const name = 'Test Device';
    const uri = 'https://device.example.com';
    const memo = 'Created a new device';

    let _ = '';
    [_, device] = await tokenMetadataProgramMerchantOwner.createDevice(
      platformSigner,
      deviceOwner.publicKey,
      name,
      uri,
      location,
      memo,
    );

    console.log('device', device);

    const account = await tokenMetadataProgram.program.account.device.fetch(device);

    expect(account.name).to.equal(name);
  });

  it('creates campaign', async () => {
    const name = 'Test Campaign';
    const uri = 'https://campaign.example.com';
    const memo = 'Created a new campaign';

    const merchantOwnerBalanceStarting =
      await tokenMetadataProgram.program.provider.connection.getBalance(merchantOwner.publicKey);

    let _ = '';
    [_, campaign] = await tokenMetadataProgramMerchantOwner.createCampaign(
      platformSigner,
      name,
      uri,
      true,
      500_000_000,
      memo,
    );

    const account = await tokenMetadataProgram.program.account.campaign.fetch(campaign);

    expect(account.name).to.equal(name);

    const accountInfo = await tokenMetadataProgram.program.account.campaign.getAccountInfo(
      campaign,
    );

    expect(accountInfo?.lamports).to.equal(503_013_680);

    const merchantOwnerBalance = await tokenMetadataProgram.program.provider.connection.getBalance(
      merchantOwner.publicKey,
    );

    expect(merchantOwnerBalanceStarting - merchantOwnerBalance).to.equal(500_010_000);
  });

  it('creates campaign location', async () => {
    let _ = '';
    [_, campaignLocation] = await tokenMetadataProgramMerchantOwner.createCampaignLocation(
      platformSigner,
      campaign,
      location,
      null,
    );

    const account = (await tokenMetadataProgram.program.account.campaignLocation.fetch(
      campaignLocation,
    )) as CampaignLocation;

    expect(account.campaign.toString()).to.equal(campaign.toString());
    expect(account.location.toString()).to.equal(location.toString());
  });

  it('transfers cpi', async () => {
    tokenMetadataProgramMerchantOwner.program.methods
      .transferCpi(1_000_000)
      .accounts({
        merchant,
        device,
        campaign,
        platform: adminSettingsAccount.platform,
        adminSettings,
      })
      .rpc();
  });

  it('Creates two promos', async () => {
    const metadataData1: DataV2 = {
      name: 'Test Promo',
      symbol: 'BTP',
      uri: 'https://arweave.net/frDiuZYzSVwYTwSUMR1YbggVkZqZfA7S9xsI3drPWBo',
      sellerFeeBasisPoints: 0,
      creators: null,
      collection: null,
      uses: null,
    };

    const metadataData2: DataV2 = {
      name: 'Test Promo 2',
      symbol: 'BTP2',
      uri: 'https://arweave.net/ZsjbP1xEBPGlS9ZLhMbR857KW9DxDwJbigl_4NSgz2E',
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

      const maxMint = 10;
      const maxRedeem = 5;

      const memo = {
        reference: metadataData.symbol,
        memo: 'Created new promo',
      };

      mint = await tokenMetadataProgramMerchantOwner.createPromo(
        platformSigner,
        metadataData,
        campaign,
        true,
        maxMint,
        maxRedeem,
        adminSettingsAccount.platform,
        JSON.stringify(memo),
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

  // This has group member1 pay for the transaction, which they are able to do because
  // of their membership in the group that owns the promo.
  it('Mints a promo token', async () => {
    const [tokenAccountAccount, mintAccount] = await tokenMetadataProgram
      .mintPromoToken(
        platformSigner,
        mint,
        deviceOwner,
        device,
        campaign,
        'just a string for a memo',
      )
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
    // try different keys for memo
    const memo = {
      source: 'spirit',
      platform: 'shoes',
      location: 'here',
    };

    const tokenAccountAccount = await tokenMetadataProgram
      .delegatePromoToken(mint, deviceOwner.publicKey, device, campaign, JSON.stringify(memo))
      .then((tokenAccount) => tokenMetadataProgram.getTokenAccount(tokenAccount));
    expect(Number(tokenAccountAccount.delegatedAmount)).to.equal(1, 'Delegated amount incorrect.');
    console.log('tokenAccountAccount: ', tokenAccountAccount);
  });

  it('Burns a delegated promo token', async () => {
    const platformStartAccountInfo =
      await tokenMetadataProgram.program.provider.connection.getAccountInfo(
        adminSettingsAccount.platform,
      );

    const campaignStartAccountInfo =
      await tokenMetadataProgram.program.provider.connection.getAccountInfo(campaign);

    console.log('mint', mint);
    const memo = {
      reference: 'myReference',
      memo: 'burned delegated token',
    };

    await tokenMetadataProgramDeviceOwner.burnDelegatedPromoToken(
      platformSigner,
      deviceOwner,
      device,
      location,
      campaign,
      tokenOwner,
      mint,
      platform.publicKey,
      JSON.stringify(memo),
    );

    const mintAccount = await tokenMetadataProgram.getMintAccount(mint);

    // TODO: Fix indexer to delete account from db if it is closed.
    // await expect(tokenMetadataProgram.getTokenAccount(tokenAccount)).to.be.rejected

    promoExtended = await tokenMetadataProgram.getPromoExtended(mint);

    expect(Number(mintAccount.supply)).to.equal(0, 'Mint supply incorrect.');
    expect(promoExtended.burnCount).to.equal(1, 'Promo burns incorrect.');

    const platformAccountInfo =
      await tokenMetadataProgram.program.provider.connection.getAccountInfo(
        adminSettingsAccount.platform,
      );

    const campaignAccountInfo =
      await tokenMetadataProgram.program.provider.connection.getAccountInfo(campaign);

    if (
      platformStartAccountInfo !== null &&
      platformAccountInfo !== null &&
      campaignStartAccountInfo !== null &&
      campaignAccountInfo !== null
    ) {
      expect(platformAccountInfo.lamports).to.equal(
        platformStartAccountInfo.lamports + adminSettingsAccount.burnPromoTokenLamports.toNumber(),
        'Campaign lamports incorrect.',
      );
      expect(campaignAccountInfo.lamports).to.equal(
        campaignStartAccountInfo.lamports - adminSettingsAccount.burnPromoTokenLamports.toNumber(),
        'Campaign lamports incorrect.',
      );
    }
  });

  it('Signs a memo', async () => {
    const memo = 'hello';
    const tx = await tokenMetadataProgram.signMemo(memo, merchantOwner);
    console.log(tx);
  });

  it('Creates an auction house', async () => {
    ({ auctionHouse } = await auctionHouseProgram.createAuctionHouse(platformSigner));
    console.log('ah_createAuctionHouse: ', auctionHouse.toString());
  });

  it('creates a sell offer', async () => {
    // payer is the seller in this case
    const salePrice = 1_000_000;
    const tokenSize = 1;

    await tokenMetadataProgram
      .mintPromoToken(
        platformSigner,
        mint,
        deviceOwner,
        device,
        campaign,
        'just a string for a memo',
      )
      .then((tokenAccount) =>
        Promise.all([
          tokenMetadataProgram.getTokenAccount(tokenAccount),
          tokenMetadataProgram.getMintAccount(promoExtended.mintAccount.address),
        ]),
      );

    console.log(
      'creates a sell offer',
      tokenMetadataProgram.payer.payer.publicKey.toString(),
      auctionHouse.toString(),
    );

    const { listingReceipt, tokenAccount } = await auctionHouseProgram.createSellOffer(
      tokenMetadataProgram.payer.payer,
      auctionHouse,
      platformSigner.publicKey,
      promoExtended.mint,
      promoExtended.metadata,
      salePrice,
      tokenSize,
    );

    const listingReceiptAccount = await ListingReceipt.fromAccountAddress(
      connection,
      listingReceipt,
    );

    console.log('ah_listing_receipt: ', listingReceiptAccount);

    const tokenAccountData = await getAccount(connection, tokenAccount);
    console.log('ah_token_account: ', tokenAccountData);
  });

  it('creates a buy offer', async () => {
    // payer is the seller in this case
    const salePrice = 1_000_000;
    const tokenSize = 1;

    const latestBlockhash = await connection.getLatestBlockhash();
    const tx = await connection.requestAirdrop(buyer.publicKey, 1_000_000_000);
    await connection.confirmTransaction({
      blockhash: latestBlockhash.blockhash,
      lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
      signature: tx,
    });

    const { bidReceipt, escrowPaymentAccount } = await auctionHouseProgram.createBuyOffer(
      buyer,
      tokenOwner,
      auctionHouse,
      platformSigner.publicKey,
      promoExtended.mint,
      promoExtended.metadata,
      salePrice,
      tokenSize,
      true,
    );

    const bidReceiptAccount = await BidReceipt.fromAccountAddress(connection, bidReceipt);
    console.log('ah_bidReceipt: ', bidReceiptAccount);

    const escrowAccountInfo = await connection.getAccountInfo(escrowPaymentAccount);
    if (escrowAccountInfo != null) {
      expect(escrowAccountInfo.lamports).to.equal(
        salePrice + 890880,
        'escrowAccount lamports incorrect.',
      );
    }
  });

  it('executes a sale', async () => {
    const salePrice = 1_000_000;
    const tokenSize = 1;

    const [auctionHouseFeeAccount] = auctionHouseProgram.findAuctionHouseFeeAddress(auctionHouse);
    const [auctionHouseTreasury] =
      auctionHouseProgram.findAuctionHouseTreasuryAddress(auctionHouse);

    for (const account of [auctionHouseFeeAccount, auctionHouseTreasury]) {
      const latestBlockhash = await connection.getLatestBlockhash();
      const tx = await connection.requestAirdrop(account, 1_000_000);
      await connection.confirmTransaction({
        blockhash: latestBlockhash.blockhash,
        lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
        signature: tx,
      });
    }

    const { purchaseReceipt, escrowPaymentAccount, buyerTokenAccount, sellerTokenAccount } =
      await auctionHouseProgram.executeSale(
        buyer,
        tokenOwner,
        auctionHouse,
        platformSigner.publicKey,
        promoExtended.mint,
        promoExtended.metadata,
        salePrice,
        tokenSize,
        true,
      );

    const purchaseReceiptAccount = await PurchaseReceipt.fromAccountAddress(
      connection,
      purchaseReceipt,
    );
    console.log('ah_purchaseReceipt: ', purchaseReceiptAccount);

    const escrowAccountInfo = await connection.getAccountInfo(escrowPaymentAccount);
    if (escrowAccountInfo != null) {
      expect(escrowAccountInfo.lamports).to.equal(890880, 'escrowAccount lamports incorrect.');
    }

    const sellerTokenAccountData = await getAccount(connection, sellerTokenAccount);
    if (sellerTokenAccountData != null) {
      expect(Number(sellerTokenAccountData.amount)).to.equal(
        0,
        'sellerTokenAccount amount incorrect.',
      );
    }

    const buyerTokenAccountData = await getAccount(connection, buyerTokenAccount);
    if (buyerTokenAccountData != null) {
      expect(Number(buyerTokenAccountData.amount)).to.equal(
        tokenSize,
        'buyerTokenAccount amount incorrect.',
      );
    }
  });

  it('creates a non-fungible', async () => {
    // payer is the seller in this case
    const metadataData2: DataV2 = {
      name: 'Test Merchant',
      symbol: 'BTM',
      uri: 'https://arweave.net/frDiuZYzSVwYTwSUMR1YbggVkZqZfA7S9xsI3drPWBo',
      sellerFeeBasisPoints: 0,
      creators: null,
      collection: null,
      uses: null,
    };

    const [_tx, _mint, metadata, edition] =
      await tokenMetadataProgramMerchantOwner.createNonFungible(metadataData2, merchantOwner);
    const [metadataAccount, editionAccount] = await Promise.all([
      tokenMetadataProgramMerchantOwner.getMetadataAccount(metadata),
      tokenMetadataProgramMerchantOwner.getMasterEditionAccount(edition),
    ]);
    console.log(metadataAccount, editionAccount);
  });
});
