import { PublicKey } from '@solana/web3.js';
import { Program, Wallet, AnchorProvider } from '@project-serum/anchor';
import * as anchor from '@project-serum/anchor';
import {
  
  NATIVE_MINT
} from '@solana/spl-token';

export class AuctionHouseProgram {  
  readonly AUCTION_HOUSE_PROGRAM_ID: PublicKey;

  readonly AUCTION_HOUSE_PREFIX: string;
  readonly AUCTION_HOUSE_FEE_PAYER_PREFIX: string;
  readonly AUCTION_HOUSE_TREASURY_PREFIX: string;

  program: Program;
  payer: Wallet;

  private constructor(program: Program, payer: Wallet) {
    this.AUCTION_HOUSE_PROGRAM_ID = new PublicKey(
      'hausS13jsjafwWwGqZTUQRmWyvyxn9EQpqMwV1PBBmk',
    );

    this.AUCTION_HOUSE_PREFIX = 'auction_house';
    this.AUCTION_HOUSE_FEE_PAYER_PREFIX = 'fee_payer';
    this.AUCTION_HOUSE_TREASURY_PREFIX = 'treasury';

    this.program = program;
    this.payer = payer;
  }

  static async fetchProgram(provider:AnchorProvider) {
    const auction_house_program_id = new PublicKey(
      'hausS13jsjafwWwGqZTUQRmWyvyxn9EQpqMwV1PBBmk',
    );
    // const idl = await anchor.Program.fetchIdl(auction_house_program_id, provider);
    const idl = await fetch("https://raw.githubusercontent.com/metaplex-foundation/metaplex-program-library/f5a94fae0f7fa397cde6fc08435bb6a79de8c350/auction-house/js/idl/auction_house.json").then(res => res.json());
    console.log("ah_idl", idl);
    const program = new Program(idl!, auction_house_program_id, provider);
    const payer = provider.wallet as Wallet;
    return new AuctionHouseProgram(program, payer)
  }
  
  /**
   * Create auction house
   *
   * @param connection           Connection to use
   * @param authority            Authority, fee, treasury withdrawal
   * @param sellerFeeBasisPoints Auction house fee, separate from metadata sellerFeeBasisPoints
   * @param requiresSignOff      Require authority sign off to execute sales
   * @param canChangeSalePrice   Authority can change sale price
   *
   * @return Tx has, address of auction house account
   */
  async createAuctionHouse(
    sellerFeeBasisPoints = 0,
    requiresSignOff = false,
    canChangeSalePrice = false,
  ): Promise<{ tx: string; auctionHouse: PublicKey }> {
    const [auctionHouse, bump] = this.findAuctionHouseAddress(
      this.payer.publicKey,
      NATIVE_MINT,
    );
    const [auctionHouseFeeAccount, feePayerBump] =
      this.findAuctionHouseFeeAddress(auctionHouse);
    const [auctionHouseTreasury, treasuryBump] =
      await this.findAuctionHouseTreasuryAddress(auctionHouse);

    // TODO: separate withdrawal destination
    const accounts = {
      treasuryMint: NATIVE_MINT,
      authority: this.payer.publicKey,
      // if program is paying for fees if requiring sign off
      feeWithdrawalDestination: this.payer.publicKey,
      // token account - associated token account of mint
      treasuryWithdrawalDestination: this.payer.publicKey,
      // token account - public key used to create ata of treasuryWithdrawalDestination
      treasuryWithdrawalDestinationOwner: this.payer.publicKey,
      auctionHouse,
      auctionHouseFeeAccount,
      auctionHouseTreasury,
    };

    const args = {
      bump,
      feePayerBump,
      treasuryBump,
      sellerFeeBasisPoints,
      requiresSignOff,
      canChangeSalePrice,
    };

    const tx = await this.program.methods.createAuctionHouse(args).accounts(accounts).rpc();

    return { tx, auctionHouse };
  }

  findAuctionHouseAddress(
    creator: PublicKey,
    treasuryMint: PublicKey,
  ): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [Buffer.from(this.AUCTION_HOUSE_PREFIX), creator.toBuffer(), treasuryMint.toBuffer()],
      this.AUCTION_HOUSE_PROGRAM_ID,
    );
  };

  findAuctionHouseFeeAddress(
    auctionHouse: PublicKey,
  ): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [
        Buffer.from(this.AUCTION_HOUSE_PREFIX),
        auctionHouse.toBuffer(),
        Buffer.from(this.AUCTION_HOUSE_FEE_PAYER_PREFIX),
      ],
      this.AUCTION_HOUSE_PROGRAM_ID,
    );
  };

  findAuctionHouseTreasuryAddress(
    auctionHouse: PublicKey,
  ): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [
        Buffer.from(this.AUCTION_HOUSE_PREFIX),
        auctionHouse.toBuffer(),
        Buffer.from(this.AUCTION_HOUSE_TREASURY_PREFIX),
      ],
      this.AUCTION_HOUSE_PROGRAM_ID,
    );
  };
}


