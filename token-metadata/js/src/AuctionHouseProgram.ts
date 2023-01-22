import { BN } from 'bn.js';
import {
  PublicKey,
  Keypair,
  Connection,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  Transaction,
  TransactionBlockhashCtor,
  sendAndConfirmTransaction,
  SYSVAR_INSTRUCTIONS_PUBKEY,
} from '@solana/web3.js';
import {
  CreateAuctionHouseInstructionArgs,
  CreateAuctionHouseInstructionAccounts,
  createCreateAuctionHouseInstruction,
  SellInstructionArgs,
  SellInstructionAccounts,
  createSellInstruction,
  PrintListingReceiptInstructionArgs,
  PrintListingReceiptInstructionAccounts,
  createPrintListingReceiptInstruction,
} from '@metaplex-foundation/mpl-auction-house';

import { NATIVE_MINT } from '@solana/spl-token';

export class AuctionHouseProgram {
  readonly AUCTION_HOUSE_PROGRAM_ID: PublicKey;
  readonly SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID: PublicKey;
  readonly TOKEN_PROGRAM_ID: PublicKey;

  readonly PREFIX: string;
  readonly FEE_PAYER_PREFIX: string;
  readonly TREASURY_PREFIX: string;
  readonly LISTING_RECEIPT_PREFIX: string;
  readonly BID_RECEIPT_PREFIX: string;
  readonly PURCHASE_RECEIPT_PREFIX: string;
  readonly SIGNER_PREFIX: string;

  connection: Connection;

  constructor(connection: Connection) {
    this.AUCTION_HOUSE_PROGRAM_ID = new PublicKey('hausS13jsjafwWwGqZTUQRmWyvyxn9EQpqMwV1PBBmk');
    this.SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID = new PublicKey(
      'ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL',
    );
    this.TOKEN_PROGRAM_ID = new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA');

    this.PREFIX = 'auction_house';
    this.FEE_PAYER_PREFIX = 'fee_payer';
    this.TREASURY_PREFIX = 'treasury';
    this.LISTING_RECEIPT_PREFIX = 'listing_receipt';
    this.BID_RECEIPT_PREFIX = 'bid_receipt';
    this.PURCHASE_RECEIPT_PREFIX = 'purchase_receipt';
    this.SIGNER_PREFIX = 'signer';

    this.connection = connection;
  }

  /**
   * Create auction house
   *
   * @param sellerFeeBasisPoints Auction house fee, separate from metadata sellerFeeBasisPoints
   * @param requiresSignOff      Require authority sign off to execute sales
   * @param canChangeSalePrice   Authority can change sale price
   *
   * @return Tx has, address of auction house account
   */
  async createAuctionHouse(
    authority: Keypair,
    sellerFeeBasisPoints = 0,
    requiresSignOff = false,
    canChangeSalePrice = false,
  ): Promise<{ tx: string; auctionHouse: PublicKey }> {
    const [auctionHouse, bump] = this.findAuctionHouseAddress(authority.publicKey, NATIVE_MINT);
    console.log('TREASURY_MINT', NATIVE_MINT);
    const [auctionHouseFeeAccount, feePayerBump] = this.findAuctionHouseFeeAddress(auctionHouse);
    const [auctionHouseTreasury, treasuryBump] = await this.findAuctionHouseTreasuryAddress(
      auctionHouse,
    );

    // TODO: separate withdrawal destination or send to platform
    const accounts: CreateAuctionHouseInstructionAccounts = {
      treasuryMint: NATIVE_MINT,
      payer: authority.publicKey,
      authority: authority.publicKey,
      // if program is paying for fees if requiring sign off
      feeWithdrawalDestination: authority.publicKey,
      // token account - associated token account of mint
      treasuryWithdrawalDestination: authority.publicKey,
      // token account - public key used to create ata of treasuryWithdrawalDestination
      treasuryWithdrawalDestinationOwner: authority.publicKey,
      auctionHouse,
      auctionHouseFeeAccount,
      auctionHouseTreasury,
      tokenProgram: this.TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
      ataProgram: this.SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID,
      rent: SYSVAR_RENT_PUBKEY,
    };

    const args: CreateAuctionHouseInstructionArgs = {
      bump,
      feePayerBump,
      treasuryBump,
      sellerFeeBasisPoints,
      requiresSignOff,
      canChangeSalePrice,
    };

    const transaction = new Transaction({
      feePayer: authority.publicKey,
    } as TransactionBlockhashCtor);
    transaction.add(createCreateAuctionHouseInstruction(accounts, args));
    const tx = await sendAndConfirmTransaction(this.connection, transaction, [authority]);

    return { tx, auctionHouse };
  }

  /**
   * Creates sell offer and listing receipt
   *
   * @param connection   Connection to use
   * @param seller        Payer
   * @param auctionHouse Auction house
   * @param salePrice    Sale price for tokenSize tokens
   * @param tokenSize    Number of tokens to offer for sale
   *
   * @return Address of listing receipt, address of seller token account
   */
  async createSellOffer(
    seller: Keypair,
    auctionHouse: PublicKey,
    authority: PublicKey,
    mint: PublicKey,
    metadata: PublicKey,
    salePrice: number,
    tokenSize: number,
  ): Promise<{ tx: string; listingReceipt: PublicKey; tokenAccount: PublicKey }> {
    const tokenAccount = this.findAssociatedTokenAccountAddress(mint, seller.publicKey);
    const [auctionHouseFeeAccount] = this.findAuctionHouseFeeAddress(auctionHouse);
    const [programAsSigner, programAsSignerBump] = this.findAuctionHouseProgramAsSignerAddress();

    const [sellerTradeState, tradeStateBump] = this.findTradeStateAddress(
      seller.publicKey,
      auctionHouse,
      tokenAccount,
      NATIVE_MINT,
      mint,
      salePrice,
      tokenSize,
    );

    const [freeSellerTradeState, freeTradeStateBump] = this.findTradeStateAddress(
      seller.publicKey,
      auctionHouse,
      tokenAccount,
      NATIVE_MINT,
      mint,
      0,
      tokenSize,
    );

    const [listingReceipt, receiptBump] = this.findListingReceiptAddress(sellerTradeState);

    const accounts: SellInstructionAccounts = {
      wallet: seller.publicKey,
      tokenAccount,
      metadata,
      authority,
      auctionHouse,
      auctionHouseFeeAccount,
      sellerTradeState,
      freeSellerTradeState,
      programAsSigner,
    };

    const args: SellInstructionArgs = {
      tradeStateBump,
      freeTradeStateBump,
      programAsSignerBump,
      buyerPrice: new BN(salePrice),
      tokenSize: new BN(tokenSize),
    };

    const transaction = new Transaction({ feePayer: seller.publicKey } as TransactionBlockhashCtor);
    transaction.add(createSellInstruction(accounts, args, this.AUCTION_HOUSE_PROGRAM_ID));

    const listingReceiptAccounts: PrintListingReceiptInstructionAccounts = {
      receipt: listingReceipt,
      bookkeeper: seller.publicKey,
      instruction: SYSVAR_INSTRUCTIONS_PUBKEY,
    };

    const listingReceiptArgs: PrintListingReceiptInstructionArgs = {
      receiptBump,
    };

    transaction.add(
      createPrintListingReceiptInstruction(listingReceiptAccounts, listingReceiptArgs),
    );

    const latestBlockhash = await this.connection.getLatestBlockhash();
    transaction.recentBlockhash = latestBlockhash.blockhash;
    transaction.sign(seller);
    const tx = await this.connection.sendRawTransaction(transaction.serialize());
    await this.connection.confirmTransaction({
      blockhash: latestBlockhash.blockhash,
      lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
      signature: tx,
    });

    return { tx, listingReceipt, tokenAccount };
  }

  findAuctionHouseAddress(creator: PublicKey, treasuryMint: PublicKey): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [Buffer.from(this.PREFIX), creator.toBuffer(), treasuryMint.toBuffer()],
      this.AUCTION_HOUSE_PROGRAM_ID,
    );
  }

  findAuctionHouseFeeAddress(auctionHouse: PublicKey): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [Buffer.from(this.PREFIX), auctionHouse.toBuffer(), Buffer.from(this.FEE_PAYER_PREFIX)],
      this.AUCTION_HOUSE_PROGRAM_ID,
    );
  }

  findAuctionHouseTreasuryAddress(auctionHouse: PublicKey): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [Buffer.from(this.PREFIX), auctionHouse.toBuffer(), Buffer.from(this.TREASURY_PREFIX)],
      this.AUCTION_HOUSE_PROGRAM_ID,
    );
  }

  findAssociatedTokenAccountAddress(mint: PublicKey, wallet: PublicKey): PublicKey {
    return PublicKey.findProgramAddressSync(
      [wallet.toBuffer(), this.TOKEN_PROGRAM_ID.toBuffer(), mint.toBuffer()],
      this.SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID,
    )[0];
  }

  findAuctionHouseProgramAsSignerAddress(): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [Buffer.from(this.PREFIX), Buffer.from(this.SIGNER_PREFIX)],
      this.AUCTION_HOUSE_PROGRAM_ID,
    );
  }

  findTradeStateAddress(
    wallet: PublicKey,
    auctionHouse: PublicKey,
    tokenAccount: PublicKey,
    treasuryMint: PublicKey,
    tokenMint: PublicKey,
    buyPrice: number,
    tokenSize: number,
  ): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [
        Buffer.from(this.PREFIX),
        wallet.toBuffer(),
        auctionHouse.toBuffer(),
        tokenAccount.toBuffer(),
        treasuryMint.toBuffer(),
        tokenMint.toBuffer(),
        new BN(buyPrice).toArrayLike(Buffer, 'le', 8),
        new BN(tokenSize).toArrayLike(Buffer, 'le', 8),
      ],
      this.AUCTION_HOUSE_PROGRAM_ID,
    );
  }

  findListingReceiptAddress(sellerTradeState: PublicKey): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [Buffer.from(this.LISTING_RECEIPT_PREFIX), sellerTradeState.toBuffer()],
      this.AUCTION_HOUSE_PROGRAM_ID,
    );
  }

  findBidReceiptAddress(buyerTradeState: PublicKey): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [Buffer.from(this.BID_RECEIPT_PREFIX), buyerTradeState.toBuffer()],
      this.AUCTION_HOUSE_PROGRAM_ID,
    );
  }

  findPurchaseReceiptAddress(
    sellerTradeState: PublicKey,
    buyerTradeState: PublicKey,
  ): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [
        Buffer.from(this.PURCHASE_RECEIPT_PREFIX),
        sellerTradeState.toBuffer(),
        buyerTradeState.toBuffer(),
      ],
      this.AUCTION_HOUSE_PROGRAM_ID,
    );
  }
}
