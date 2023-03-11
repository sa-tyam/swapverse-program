import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Swapverse } from "../target/types/swapverse";

import {
  createMint,
  createAccount,
  getAccount,
  getOrCreateAssociatedTokenAccount,
  transfer,
  mintTo,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { BN } from "bn.js";

describe("swapverse", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.Swapverse as Program<Swapverse>;

  let provider = anchor.AnchorProvider.env();
  let wallet = anchor.AnchorProvider.env().wallet;

  const LAMPORTS_PER_SOL = 1_000_000_000;

  let signing_authority: anchor.web3.PublicKey;
  let global_state: anchor.web3.PublicKey;

  let usdc_dev_mint: anchor.web3.PublicKey;
  let usdt_dev_mint: anchor.web3.PublicKey;
  let uxd_dev_mint: anchor.web3.PublicKey;
  let pai_dev_mint: anchor.web3.PublicKey;
  let usdh_dev_mint: anchor.web3.PublicKey;

  const investor1 = anchor.web3.Keypair.generate();
  const investor2 = anchor.web3.Keypair.generate();
  const user1 = anchor.web3.Keypair.generate();
  const user2 = anchor.web3.Keypair.generate();

  let investor1_usdc_ata;
  let investor1_usdt_ata;
  let investor2_usdc_ata;
  let investor2_usdt_ata;
  let user1_usdc_ata;
  let user1_usdt_ata;
  let user2_usdc_ata;
  let user2_usdt_ata;

  let swap_pool: anchor.web3.PublicKey;
  let pool_share_token_a_mint: anchor.web3.PublicKey;
  let pool_share_token_b_mint: anchor.web3.PublicKey;
  let swap_pool_usdc_ata;
  let swap_pool_usdt_ata;
  let swap_pool_treasury_token_a_ata;
  let swap_pool_treasury_token_b_ata;
  let investor1_pool_share_token_a_ata;
  let investor1_pool_share_token_b_ata;
  let investor2_pool_share_token_a_ata;
  let investor2_pool_share_token_b_ata;

  let investor1_pool_info: anchor.web3.PublicKey;
  let investor2_pool_info: anchor.web3.PublicKey;

  before(async () => {
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(
        wallet.publicKey,
        100 * LAMPORTS_PER_SOL
      )
    );
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(
        investor1.publicKey,
        100 * LAMPORTS_PER_SOL
      )
    );
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(
        investor2.publicKey,
        100 * LAMPORTS_PER_SOL
      )
    );
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(
        user1.publicKey,
        100 * LAMPORTS_PER_SOL
      )
    );
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(
        user2.publicKey,
        100 * LAMPORTS_PER_SOL
      )
    );
  });

  it("Initialize global state!", async () => {
    // Add your test here.
    // const tx = await program.methods.initialize().rpc();
    // console.log("Your transaction signature", tx);
    let [global_state_add, global_state_b] =
      await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from("global-state")],
        program.programId
      );
    global_state = global_state_add;

    let [signing_authority_add, signing_authority_b] =
      await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from("signing-authority")],
        program.programId
      );
    signing_authority = signing_authority_add;

    let [usdc_dev_mint_add, usdc_dev_mint_b] =
      await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from("usdc-dev")],
        program.programId
      );
    usdc_dev_mint = usdc_dev_mint_add;

    let [usdt_dev_mint_add, usdt_dev_mint_b] =
      await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from("usdt-dev")],
        program.programId
      );
    usdt_dev_mint = usdt_dev_mint_add;

    let [uxd_dev_mint_add, uxd_dev_mint_b] =
      await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from("uxd-dev")],
        program.programId
      );
    uxd_dev_mint = uxd_dev_mint_add;

    let [pai_dev_mint_add, pai_dev_mint_b] =
      await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from("pai-dev")],
        program.programId
      );
    pai_dev_mint = pai_dev_mint_add;

    let [usdh_dev_mint_add, usdh_dev_mint_b] =
      await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from("usdh-dev")],
        program.programId
      );
    usdh_dev_mint = usdh_dev_mint_add;

    const tx = await program.methods
      .initializeGlobalState()
      .accounts({
        owner: wallet.publicKey,
        globalState: global_state,
        signingAuthority: signing_authority,
        usdcTokenMint: usdc_dev_mint,
        usdtTokenMint: usdt_dev_mint,
        uxdTokenMint: uxd_dev_mint,
        paiTokenMint: pai_dev_mint,
        usdhTokenMint: usdh_dev_mint,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("Your transaction signature", tx);
  });

  it("Gets test tokens", async () => {
    // Initialise ATA
    investor1_usdc_ata = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      investor1,
      usdc_dev_mint,
      investor1.publicKey
    );
    investor1_usdt_ata = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      investor1,
      usdt_dev_mint,
      investor1.publicKey
    );
    investor2_usdc_ata = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      investor2,
      usdc_dev_mint,
      investor2.publicKey
    );
    investor2_usdt_ata = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      investor2,
      usdt_dev_mint,
      investor2.publicKey
    );
    user1_usdc_ata = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      user1,
      usdc_dev_mint,
      user1.publicKey
    );
    user1_usdt_ata = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      user1,
      usdt_dev_mint,
      user1.publicKey
    );
    user2_usdc_ata = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      user2,
      usdc_dev_mint,
      user2.publicKey
    );
    user2_usdt_ata = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      user2,
      usdt_dev_mint,
      user2.publicKey
    );

    const tx = await program.methods
      .getTestTokens(new anchor.BN(1_000_000))
      .accounts({
        investor: investor1.publicKey,
        globalState: global_state,
        signingAuthority: signing_authority,
        tokenMint: usdc_dev_mint,
        investorTokenAccount: investor1_usdc_ata.address,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([investor1])
      .rpc();
    console.log("Your transaction signature", tx);

    const tx2 = await program.methods
      .getTestTokens(new anchor.BN(1_000_000))
      .accounts({
        investor: investor1.publicKey,
        globalState: global_state,
        signingAuthority: signing_authority,
        tokenMint: usdt_dev_mint,
        investorTokenAccount: investor1_usdt_ata.address,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([investor1])
      .rpc();
    console.log("Your transaction signature", tx2);

    const tx3 = await program.methods
      .getTestTokens(new anchor.BN(1_000_000))
      .accounts({
        investor: investor2.publicKey,
        globalState: global_state,
        signingAuthority: signing_authority,
        tokenMint: usdc_dev_mint,
        investorTokenAccount: investor2_usdc_ata.address,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([investor2])
      .rpc();
    console.log("Your transaction signature", tx3);

    const tx4 = await program.methods
      .getTestTokens(new anchor.BN(1_000_000))
      .accounts({
        investor: investor2.publicKey,
        globalState: global_state,
        signingAuthority: signing_authority,
        tokenMint: usdt_dev_mint,
        investorTokenAccount: investor2_usdt_ata.address,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([investor2])
      .rpc();
    console.log("Your transaction signature", tx4);

    const tx5 = await program.methods
      .getTestTokens(new anchor.BN(1_000_000))
      .accounts({
        investor: user1.publicKey,
        globalState: global_state,
        signingAuthority: signing_authority,
        tokenMint: usdc_dev_mint,
        investorTokenAccount: user1_usdc_ata.address,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([user1])
      .rpc();
    console.log("Your transaction signature", tx5);

    const tx6 = await program.methods
      .getTestTokens(new anchor.BN(1_000_000))
      .accounts({
        investor: user1.publicKey,
        globalState: global_state,
        signingAuthority: signing_authority,
        tokenMint: usdt_dev_mint,
        investorTokenAccount: user1_usdt_ata.address,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([user1])
      .rpc();
    console.log("Your transaction signature", tx6);

    const tx7 = await program.methods
      .getTestTokens(new anchor.BN(1_000_000))
      .accounts({
        investor: user2.publicKey,
        globalState: global_state,
        signingAuthority: signing_authority,
        tokenMint: usdc_dev_mint,
        investorTokenAccount: user2_usdc_ata.address,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([user2])
      .rpc();
    console.log("Your transaction signature", tx5);

    const tx8 = await program.methods
      .getTestTokens(new anchor.BN(1_000_000))
      .accounts({
        investor: user2.publicKey,
        globalState: global_state,
        signingAuthority: signing_authority,
        tokenMint: usdt_dev_mint,
        investorTokenAccount: user2_usdt_ata.address,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([user2])
      .rpc();
    console.log("Your transaction signature", tx6);
  });

  it("Creates swap pool", async () => {
    const swap_pool_num = new BN(0).toArrayLike(Buffer, "le", 8);
    let [swap_pool_add, swap_pool_b] =
      await anchor.web3.PublicKey.findProgramAddress(
        [swap_pool_num, Buffer.from("swap-pool")],
        program.programId
      );
    swap_pool = swap_pool_add;

    console.log("swap_pool_address: ", swap_pool.toBase58());

    let [pool_share_token_a_mint_add, pool_share_token_a_mint_b] =
      await anchor.web3.PublicKey.findProgramAddress(
        [
          swap_pool.toBuffer(),
          usdc_dev_mint.toBuffer(),
          Buffer.from("pool-share-token"),
        ],
        program.programId
      );
    pool_share_token_a_mint = pool_share_token_a_mint_add;

    let [pool_share_token_b_mint_add, pool_share_token_b_mint_b] =
      await anchor.web3.PublicKey.findProgramAddress(
        [
          swap_pool.toBuffer(),
          usdt_dev_mint.toBuffer(),
          Buffer.from("pool-share-token"),
        ],
        program.programId
      );
    pool_share_token_b_mint = pool_share_token_b_mint_add;

    let initial_amount_a = new BN(100_000);
    let min_amount = new BN(10_000);
    let life = new BN(360);
    const tx = await program.methods
      .createSwapPool(
        initial_amount_a,
        initial_amount_a,
        10,
        10,
        min_amount,
        30,
        life
      )
      .accounts({
        owner: wallet.publicKey,
        globalState: global_state,
        signingAuthority: signing_authority,
        swapPool: swap_pool,
        tokenAMint: usdc_dev_mint,
        tokenBMint: usdt_dev_mint,
        poolShareTokenAMint: pool_share_token_a_mint,
        poolShareTokenBMint: pool_share_token_b_mint,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("Your transaction signature", tx);
  });

  it("Invest in swap pool", async () => {
    investor1_pool_share_token_a_ata = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      investor1,
      pool_share_token_a_mint,
      investor1.publicKey
    );
    investor1_pool_share_token_b_ata = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      investor1,
      pool_share_token_b_mint,
      investor1.publicKey
    );
    investor2_pool_share_token_a_ata = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      investor2,
      pool_share_token_a_mint,
      investor2.publicKey
    );
    investor2_pool_share_token_b_ata = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      investor2,
      pool_share_token_b_mint,
      investor2.publicKey
    );

    let [swap_pool_usdc_ata_add, swap_pool_usdc_ata_b] =
      await anchor.web3.PublicKey.findProgramAddress(
        [swap_pool.toBuffer(), usdc_dev_mint.toBuffer()],
        program.programId
      );
    swap_pool_usdc_ata = swap_pool_usdc_ata_add;

    let [swap_pool_usdt_ata_add, swap_pool_usdt_ata_b] =
      await anchor.web3.PublicKey.findProgramAddress(
        [swap_pool.toBuffer(), usdt_dev_mint.toBuffer()],
        program.programId
      );
    swap_pool_usdt_ata = swap_pool_usdt_ata_add;

    let [investor1_pool_info_add, investor1_pool_info_b] =
      await anchor.web3.PublicKey.findProgramAddress(
        [swap_pool.toBuffer(), investor1.publicKey.toBuffer()],
        program.programId
      );
    investor1_pool_info = investor1_pool_info_add;

    let [investor2_pool_info_add, investor2_pool_info_b] =
      await anchor.web3.PublicKey.findProgramAddress(
        [swap_pool.toBuffer(), investor2.publicKey.toBuffer()],
        program.programId
      );
    investor2_pool_info = investor2_pool_info_add;

    let amount = new BN(70_000);
    let tx = await program.methods
      .investSwapPool(amount)
      .accounts({
        investor: investor1.publicKey,
        globalState: global_state,
        signingAuthority: signing_authority,
        swapPool: swap_pool,
        tokenMint: usdc_dev_mint,
        swapPoolTokenAccount: swap_pool_usdc_ata,
        investorTokenAccount: investor1_usdc_ata.address,
        poolShareTokenMint: pool_share_token_a_mint,
        investorPoolShareTokenAccount: investor1_pool_share_token_a_ata.address,
        investorPoolInfo: investor1_pool_info,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([investor1])
      .rpc();
    console.log("Your transaction signature is ", tx);

    amount = new BN(30_000);
    let tx2 = await program.methods
      .investSwapPool(amount)
      .accounts({
        investor: investor2.publicKey,
        globalState: global_state,
        signingAuthority: signing_authority,
        swapPool: swap_pool,
        tokenMint: usdc_dev_mint,
        swapPoolTokenAccount: swap_pool_usdc_ata,
        investorTokenAccount: investor2_usdc_ata.address,
        poolShareTokenMint: pool_share_token_a_mint,
        investorPoolShareTokenAccount: investor2_pool_share_token_a_ata.address,
        investorPoolInfo: investor2_pool_info,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([investor2])
      .rpc();
    console.log("Your transaction signature is ", tx2);

    amount = new BN(40_000);
    let tx3 = await program.methods
      .investSwapPool(amount)
      .accounts({
        investor: investor1.publicKey,
        globalState: global_state,
        signingAuthority: signing_authority,
        swapPool: swap_pool,
        tokenMint: usdt_dev_mint,
        swapPoolTokenAccount: swap_pool_usdt_ata,
        investorTokenAccount: investor1_usdt_ata.address,
        poolShareTokenMint: pool_share_token_b_mint,
        investorPoolShareTokenAccount: investor1_pool_share_token_b_ata.address,
        investorPoolInfo: investor1_pool_info,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([investor1])
      .rpc();
    console.log("Your transaction signature is ", tx3);

    amount = new BN(60_000);
    let tx4 = await program.methods
      .investSwapPool(amount)
      .accounts({
        investor: investor2.publicKey,
        globalState: global_state,
        signingAuthority: signing_authority,
        swapPool: swap_pool,
        tokenMint: usdt_dev_mint,
        swapPoolTokenAccount: swap_pool_usdt_ata,
        investorTokenAccount: investor2_usdt_ata.address,
        poolShareTokenMint: pool_share_token_b_mint,
        investorPoolShareTokenAccount: investor2_pool_share_token_b_ata.address,
        investorPoolInfo: investor2_pool_info,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([investor2])
      .rpc();
    console.log("Your transaction signature is ", tx4);
  });

  it("swaps token", async () => {
    let [swap_pool_treasury_token_a_ata_add, swap_pool_treasury_token_a_b] =
      await anchor.web3.PublicKey.findProgramAddress(
        [
          swap_pool.toBuffer(),
          usdc_dev_mint.toBuffer(),
          Buffer.from("treasury-account"),
        ],
        program.programId
      );
    swap_pool_treasury_token_a_ata = swap_pool_treasury_token_a_ata_add;

    let [swap_pool_treasury_token_b_ata_add, swap_pool_treasury_token_b_b] =
      await anchor.web3.PublicKey.findProgramAddress(
        [
          swap_pool.toBuffer(),
          usdt_dev_mint.toBuffer(),
          Buffer.from("treasury-account"),
        ],
        program.programId
      );
    swap_pool_treasury_token_b_ata = swap_pool_treasury_token_b_ata_add;

    let amount = new BN(10_000);
    let min_amount_out = new BN(8_000);
    let tx = await program.methods
      .swapToken(amount, min_amount_out, true)
      .accounts({
        user: user1.publicKey,
        globalState: global_state,
        signingAuthority: signing_authority,
        swapPool: swap_pool,
        tokenAMint: usdc_dev_mint,
        tokenBMint: usdt_dev_mint,
        userTokenAAccount: user1_usdc_ata.address,
        userTokenBAccount: user1_usdt_ata.address,
        swapPoolTokenAAccount: swap_pool_usdc_ata,
        swapPoolTokenBAccount: swap_pool_usdt_ata,
        swapPoolTreasuryTokenAAccount: swap_pool_treasury_token_a_ata,
        swapPoolTreasuryTokenBAccount: swap_pool_treasury_token_b_ata,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([user1])
      .rpc();
    console.log("Your transaction signature is ", tx);

    amount = new BN(10_000);
    min_amount_out = new BN(6_000);
    let tx2 = await program.methods
      .swapToken(amount, min_amount_out, true)
      .accounts({
        user: user1.publicKey,
        globalState: global_state,
        signingAuthority: signing_authority,
        swapPool: swap_pool,
        tokenAMint: usdc_dev_mint,
        tokenBMint: usdt_dev_mint,
        userTokenAAccount: user1_usdc_ata.address,
        userTokenBAccount: user1_usdt_ata.address,
        swapPoolTokenAAccount: swap_pool_usdc_ata,
        swapPoolTokenBAccount: swap_pool_usdt_ata,
        swapPoolTreasuryTokenAAccount: swap_pool_treasury_token_a_ata,
        swapPoolTreasuryTokenBAccount: swap_pool_treasury_token_b_ata,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([user1])
      .rpc();
    console.log("Your transaction signature is ", tx2);

    amount = new BN(10_000);
    min_amount_out = new BN(6_000);
    let tx3 = await program.methods
      .swapToken(amount, min_amount_out, false)
      .accounts({
        user: user1.publicKey,
        globalState: global_state,
        signingAuthority: signing_authority,
        swapPool: swap_pool,
        tokenAMint: usdc_dev_mint,
        tokenBMint: usdt_dev_mint,
        userTokenAAccount: user1_usdc_ata.address,
        userTokenBAccount: user1_usdt_ata.address,
        swapPoolTokenAAccount: swap_pool_usdc_ata,
        swapPoolTokenBAccount: swap_pool_usdt_ata,
        swapPoolTreasuryTokenAAccount: swap_pool_treasury_token_a_ata,
        swapPoolTreasuryTokenBAccount: swap_pool_treasury_token_b_ata,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([user1])
      .rpc();
    console.log("Your transaction signature is ", tx3);

    amount = new BN(10_000);
    min_amount_out = new BN(6_000);
    let tx4 = await program.methods
      .swapToken(amount, min_amount_out, false)
      .accounts({
        user: user1.publicKey,
        globalState: global_state,
        signingAuthority: signing_authority,
        swapPool: swap_pool,
        tokenAMint: usdc_dev_mint,
        tokenBMint: usdt_dev_mint,
        userTokenAAccount: user1_usdc_ata.address,
        userTokenBAccount: user1_usdt_ata.address,
        swapPoolTokenAAccount: swap_pool_usdc_ata,
        swapPoolTokenBAccount: swap_pool_usdt_ata,
        swapPoolTreasuryTokenAAccount: swap_pool_treasury_token_a_ata,
        swapPoolTreasuryTokenBAccount: swap_pool_treasury_token_b_ata,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([user1])
      .rpc();
    console.log("Your transaction signature is ", tx4);
  });

  it("claims profit", async () => {
    let tx = await program.methods
      .claimProfit()
      .accounts({
        investor: investor1.publicKey,
        globalState: global_state,
        signingAuthority: signing_authority,
        swapPool: swap_pool,
        withdrawTokenMint: usdc_dev_mint,
        tokenAMint: usdc_dev_mint,
        tokenBMint: usdt_dev_mint,
        swapPoolTreasuryTokenAAccount: swap_pool_treasury_token_a_ata,
        swapPoolTreasuryTokenBAccount: swap_pool_treasury_token_b_ata,
        investorTokenAccount: investor1_usdc_ata.address,
        poolShareTokenMint: pool_share_token_a_mint,
        investorPoolShareTokenAccount: investor1_pool_share_token_a_ata.address,
        investorPoolInfo: investor1_pool_info,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([investor1])
      .rpc();
    console.log("Your transaction signature is ", tx);

    let tx3 = await program.methods
      .claimProfit()
      .accounts({
        investor: investor2.publicKey,
        globalState: global_state,
        signingAuthority: signing_authority,
        swapPool: swap_pool,
        withdrawTokenMint: usdc_dev_mint,
        tokenAMint: usdc_dev_mint,
        tokenBMint: usdt_dev_mint,
        swapPoolTreasuryTokenAAccount: swap_pool_treasury_token_a_ata,
        swapPoolTreasuryTokenBAccount: swap_pool_treasury_token_b_ata,
        investorTokenAccount: investor2_usdc_ata.address,
        poolShareTokenMint: pool_share_token_a_mint,
        investorPoolShareTokenAccount: investor2_pool_share_token_a_ata.address,
        investorPoolInfo: investor2_pool_info,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([investor2])
      .rpc();
    console.log("Your transaction signature is ", tx3);
  });

  // following lines should be added at the end of check_for_withdrawal_open function in withdraw_swap_pool.rs
  // NOTE: dangerous to add them for live product
  // self.swap_pool.open_for_investment = false;
  // self.swap_pool.open_for_withdrawal = true;
  // self.set_withdrawable_values();
  it.skip("withdraws from swap pool", async () => {
    let tx = await program.methods
      .withdrawSwapPool(true)
      .accounts({
        investor: investor1.publicKey,
        globalState: global_state,
        signingAuthority: signing_authority,
        swapPool: swap_pool,
        tokenAMint: usdc_dev_mint,
        tokenBMint: usdt_dev_mint,
        swapPoolTokenAAccount: swap_pool_usdc_ata,
        swapPoolTokenBAccount: swap_pool_usdt_ata,
        investorTokenAAccount: investor1_usdc_ata.address,
        investorTokenBAccount: investor1_usdt_ata.address,
        poolShareTokenAMint: pool_share_token_a_mint,
        poolShareTokenBMint: pool_share_token_b_mint,
        investorPoolShareTokenAAccount: investor1_pool_share_token_a_ata.address,
        investorPoolShareTokenBAccount: investor1_pool_share_token_b_ata.address,
        investorPoolInfo: investor1_pool_info,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([investor1])
      .rpc();
    console.log("Your transaction signature is ", tx);
  });
});
