import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { Pumpstake } from "../target/types/pumpstake";
import { randomBytes } from "crypto"
import { str, struct, u64, u8 } from "@coral-xyz/borsh"
import { ASSOCIATED_TOKEN_PROGRAM_ID, getAccount, getAssociatedTokenAddress, getAssociatedTokenAddressSync, getOrCreateAssociatedTokenAccount, TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID, tokenGroupMemberInitialize } from "@solana/spl-token";
import { configAddress } from "./config";
import { initialize } from "./utils";
describe("initialize program tests", () => {
    const provider = anchor.AnchorProvider.env()
    anchor.setProvider(provider);
    const owner = anchor.Wallet.local().payer;
    const marketCreator = anchor.web3.Keypair.generate();
    const program = anchor.workspace.Pumpstake as Program<Pumpstake>;
    const TOKEN_METADATA_PROGRAM_ID = new anchor.web3.PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s")
    const stakeEvent = program.addEventListener("stakeEvent", (e, slot) => {
        console.log("Someone staked and here's the details", JSON.stringify(e))
    })

    let marketParams = {
        marketType: 0,
        ticker: "Test",
        name: "Hello",
        image: "test",
        description: "who will win the US election", // This is the question
        twitter: "x.com",
        website: "x.com",
        telegram: "telegram.org",
    }
    const mint = anchor.web3.Keypair.generate()


    let seed = new anchor.BN(randomBytes(8)) // this is for coin toss bet
    let seed2 = new anchor.BN(randomBytes(8)) // this is for 5 options bet(polymarket)
    async function confirmTransaction(
        connection: anchor.web3.Connection,
        signature: anchor.web3.TransactionSignature,
        desiredConfirmationStatus: anchor.web3.TransactionConfirmationStatus = 'confirmed',
        timeout: number = 30000,
        pollInterval: number = 1000,
        searchTransactionHistory: boolean = false
    ): Promise<anchor.web3.SignatureStatus> {
        const start = Date.now();

        while (Date.now() - start < timeout) {
            const { value: statuses } = await connection.getSignatureStatuses([signature], { searchTransactionHistory });

            if (!statuses || statuses.length === 0) {
                throw new Error('Failed to get signature status');
            }

            const status = statuses[0];

            if (status === null) {
                // If status is null, the transaction is not yet known
                await new Promise(resolve => setTimeout(resolve, pollInterval));
                continue;
            }

            if (status.err) {
                throw new Error(`Transaction failed: ${JSON.stringify(status.err)}`);
            }

            if (status.confirmationStatus && status.confirmationStatus === desiredConfirmationStatus) {
                return status;
            }

            if (status.confirmationStatus === 'finalized') {
                return status;
            }

            await new Promise(resolve => setTimeout(resolve, pollInterval));
        }

        throw new Error(`Transaction confirmation timeout after ${timeout}ms`);
    }
    let anotherUser = anchor.web3.Keypair.generate()
    it("initialize all the accounts", async () => {
        const tx = await provider.connection.requestAirdrop(anotherUser.publicKey, anchor.web3.LAMPORTS_PER_SOL * 100);
        await provider.connection.confirmTransaction(tx)
        const tx1 = await provider.connection.requestAirdrop(marketCreator.publicKey, anchor.web3.LAMPORTS_PER_SOL * 100);
        await provider.connection.confirmTransaction(tx1)

    })

    it("can create a new coin toss market", async () => {
        let [market, _] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("market"), marketCreator.publicKey.toBuffer(), seed.toArrayLike(Buffer, "le", 8)],
            program.programId
        )
        console.log("market: ", market.toBase58())
        console.log("owner: ", owner.publicKey.toBase58())
        let duration = new anchor.BN(1000)
        let totalOptions = 2 // number of options for coin toss is 2
        let optionDetails = [
            {
                optionId: 0,
                name: "Option 1",
                image: "xyz",
                description: "xyz",
                liquidity: new anchor.BN(0),
            },
            {
                optionId: 1,
                name: "Option 2",
                image: "xyz",
                description: "xyz",
                liquidity: new anchor.BN(0),
            }
        ]
        const tx = await program.methods.createPredictionMarket(seed, totalOptions, duration, marketParams, optionDetails)
            .accountsPartial({
                signer: marketCreator.publicKey,
                market
            })
            .signers([marketCreator])
            .rpc()
        console.log("created market: ", tx)
    })
    it("can stake on heads", async () => {
        let betId = new anchor.BN(69420)
        let [market, _] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("market"), marketCreator.publicKey.toBuffer(), seed.toArrayLike(Buffer, "le", 8)],
            program.programId
        )
        const bet = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("bet"), market.toBuffer(), owner.publicKey.toBuffer(), betId.toArrayLike(Buffer, "le", 8)],
            program.programId
        )[0]
        console.log("THIS IS BET ACCOUNT: ", bet.toBase58())
        const amount = new anchor.BN(anchor.web3.LAMPORTS_PER_SOL * 100)
        const option_id = 1 //lets assume 1 to be heads in coin toss
        const tx = await program.methods.stake(betId, option_id, amount)
            .accountsPartial({
                signer: owner.publicKey,
                market: market,
                bet: bet,
                revenue: new anchor.web3.PublicKey("GmkqS3uguupCzEbwcWYnRrhtSvNZj2ycUWWSCE4QHedr")

            }).signers([owner]).rpc()
        console.log("Sucessfully staked on heads: ", tx)
    })
    it("can stake on tails", async () => {
        let betId = new anchor.BN(69)
        let [market, _] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("market"), marketCreator.publicKey.toBuffer(), seed.toArrayLike(Buffer, "le", 8)],
            program.programId
        )
        const bet = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("bet"), market.toBuffer(), owner.publicKey.toBuffer(), betId.toArrayLike(Buffer, "le", 8)],
            program.programId
        )[0]
        const amount = new anchor.BN(anchor.web3.LAMPORTS_PER_SOL * 1)
        const option_id = 0 // lets assume 0 to be tails
        const tx = await program.methods.stake(betId, option_id, amount)
            .accountsPartial({
                signer: owner.publicKey,
                market: market,
                bet: bet,
                revenue: new anchor.web3.PublicKey("GmkqS3uguupCzEbwcWYnRrhtSvNZj2ycUWWSCE4QHedr")
            }).signers([owner]).rpc()
        console.log("Sucessfully staked on tails: ", tx)
        const data = await program.account.predictionMarket.fetch(market)
        console.log(data.marketOptions[0].liquidity.toNumber())
    })
    it("can stake on tails again(by anotherUser)", async () => {
        let betId = new anchor.BN(70)
        let [market, _] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("market"), marketCreator.publicKey.toBuffer(), seed.toArrayLike(Buffer, "le", 8)],
            program.programId
        )
        const bet = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("bet"), market.toBuffer(), anotherUser.publicKey.toBuffer(), betId.toArrayLike(Buffer, "le", 8)],
            program.programId
        )[0]
        const amount = new anchor.BN(anchor.web3.LAMPORTS_PER_SOL * 0.1)
        const option_id = 0 // lets assume 0 to be tails
        const tx = await program.methods.stake(betId, option_id, amount)
            .accountsPartial({
                signer: anotherUser.publicKey,
                market: market,
                bet: bet,
                revenue: new anchor.web3.PublicKey("GmkqS3uguupCzEbwcWYnRrhtSvNZj2ycUWWSCE4QHedr")
            }).signers([anotherUser]).rpc()
        console.log("Sucessfully staked on tails: ", tx)
        const data = await program.account.predictionMarket.fetch(market)
        console.log(data.marketOptions[0].liquidity.toNumber())
    })
    it("can resolve the coin toss market and mark a winner", async () => {
        await new Promise(resolve => setTimeout(resolve, 1050));
        console.log("Timeout completed. Now resolving market...");
        let [market, _] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("market"), marketCreator.publicKey.toBuffer(), seed.toArrayLike(Buffer, "le", 8)],
            program.programId
        )
        const tx = await program.methods.resolveMarket(0)
            .accountsPartial({
                market,
                signer: owner.publicKey
            }).signers([owner])
            .rpc()
        console.log("resolved winner: ", tx)
        const data = await program.account.predictionMarket.fetch(market)
        if (data.graduate && data.winnerPresent) {
            console.log("-------------CREATING A COIN----------------")
            // const mint = anchor.web3.PublicKey.findProgramAddressSync(
            //     [Buffer.from("mint"), market.toBuffer()],
            //     program.programId
            // )[0]
            const [metadataAddress, _] = anchor.web3.PublicKey.findProgramAddressSync(
                [
                    Buffer.from("metadata"),
                    TOKEN_METADATA_PROGRAM_ID.toBuffer(),
                    mint.publicKey.toBuffer(),
                ],
                TOKEN_METADATA_PROGRAM_ID
            );
            console.log("Metadata account is: ", metadataAddress.toBase58())
            console.log("Mint account is: ", mint.publicKey.toBase58())
            const tokenReserve = getAssociatedTokenAddressSync(mint.publicKey, market, true, TOKEN_2022_PROGRAM_ID)
            console.log("Token reserve is: ", tokenReserve.toBase58())

            let tx1 = await program.methods.createCoin()
                .accountsPartial({
                    market,
                    mint: mint.publicKey,
                    metadata: metadataAddress,
                    revenue: new anchor.web3.PublicKey("GmkqS3uguupCzEbwcWYnRrhtSvNZj2ycUWWSCE4QHedr"),
                    tokenReserve,
                    signer: owner.publicKey,
                    tokenProgram: TOKEN_2022_PROGRAM_ID,
                    tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
                    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
                    sysvarProgram: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY
                })
                .signers([owner, mint])
                .rpc()
            console.log("created and minted coins at reserve", tx1);
        } else {
            console.log("Market cannot graduate")
        }
    })
    it("can close and distribute rewards and amounts", async () => {
        let betId1 = new anchor.BN(69420) // loser
        let betId2 = new anchor.BN(69)
        let betId3 = new anchor.BN(70)
        let [market, _] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("market"), marketCreator.publicKey.toBuffer(), seed.toArrayLike(Buffer, "le", 8)],
            program.programId
        )
        const data = await program.account.predictionMarket.fetch(market)
        const bet1 = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("bet"), market.toBuffer(), owner.publicKey.toBuffer(), betId1.toArrayLike(Buffer, "le", 8)],
            program.programId
        )[0]
        const bet2 = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("bet"), market.toBuffer(), owner.publicKey.toBuffer(), betId2.toArrayLike(Buffer, "le", 8)],
            program.programId
        )[0]
        const bet3 = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("bet"), market.toBuffer(), anotherUser.publicKey.toBuffer(), betId3.toArrayLike(Buffer, "le", 8)],
            program.programId
        )[0]
        if (data.winnerPresent) {

            if (data.graduate == false) {
                let tx = await program.methods.claim()
                    .accountsPartial({
                        bet: bet1,
                        market,
                        reciever: owner.publicKey,

                    }).signers([owner])
                    .rpc()
                console.log("successfully refunded: ", tx)
                let tx2 = await program.methods.claim()
                    .accountsPartial({
                        bet: bet2,
                        market,
                        reciever: owner.publicKey,

                    }).signers([owner])
                    .rpc()
                console.log("successfully refunded2: ", tx2)
                let tx3 = await program.methods.claim()
                    .accountsPartial({
                        bet: bet3,
                        market,
                        reciever: anotherUser.publicKey,

                    }).signers([owner])
                    .rpc()
                console.log("successfully resolved bet: ", tx3)


            } else {
                let ix1 = await program.methods.claim2()
                    .accountsPartial({
                        bet: bet1,
                        marketCreator: marketCreator.publicKey,
                        market,
                        receiver: owner.publicKey,
                        tokenProgram: TOKEN_2022_PROGRAM_ID,
                        mint: mint.publicKey
                    }).signers([owner])
                    .rpc()
                console.log("Tx1: ", ix1)
                let ix2 = await program.methods.claim2()
                    .accountsPartial({
                        bet: bet2,
                        market,
                        marketCreator: marketCreator.publicKey,
                        receiver: owner.publicKey,
                        tokenProgram: TOKEN_2022_PROGRAM_ID,
                        mint: mint.publicKey
                    }).signers([owner])
                    .rpc()
                console.log("Tx2: ", ix2)
                let ix3 = await program.methods.claim2()
                    .accountsPartial({
                        bet: bet3,
                        market,
                        marketCreator: marketCreator.publicKey,
                        receiver: anotherUser.publicKey,
                        tokenProgram: TOKEN_2022_PROGRAM_ID,
                        mint: mint.publicKey
                    }).signers([owner])
                    .rpc()
                console.log("Tx3: ", ix3)

                // const instructions: anchor.web3.TransactionInstruction[] = [
                //     ix1, ix2,
                //     // ix3
                // ]
                // let blockhash = (await provider.connection.getLatestBlockhash()).blockhash
                // const messageV0 = new anchor.web3.TransactionMessage({
                //     payerKey: owner.publicKey,
                //     recentBlockhash: blockhash,
                //     instructions: instructions
                // }).compileToV0Message()
                // const transaction = new anchor.web3.VersionedTransaction(messageV0)
                // transaction.sign([owner])
                // const tx = await provider.connection.sendTransaction(transaction)
                // const confirmation = await confirmTransaction(provider.connection, tx)
                // if (confirmation.err) { throw new Error("❌ - Transaction not confirmed.") }
                // console.log("Tx: ", tx)
            }
        } else {
            console.log("NO WINNER FOUND. REFUNDING ALL BET AMOUNTS")
            let ix1 = await program.methods.refund()
                .accountsPartial({
                    bet: bet1,
                    market,
                    reciever: owner.publicKey
                }).signers([owner])
                .instruction()
            let ix2 = await program.methods.refund()
                .accountsPartial({
                    bet: bet2,
                    market,
                    reciever: owner.publicKey
                }).signers([owner])
                .instruction()
            let ix3 = await program.methods.refund()
                .accountsPartial({
                    bet: bet3,
                    market,
                    reciever: anotherUser.publicKey,
                }).signers([owner])
                .instruction()

            const instructions: anchor.web3.TransactionInstruction[] = [
                ix1, ix2,
                // ix3
            ]
            let blockhash = (await provider.connection.getLatestBlockhash()).blockhash
            const messageV0 = new anchor.web3.TransactionMessage({
                payerKey: owner.publicKey,
                recentBlockhash: blockhash,
                instructions: instructions
            }).compileToV0Message()
            const transaction = new anchor.web3.VersionedTransaction(messageV0)
            transaction.sign([owner])
            const tx = await provider.connection.sendTransaction(transaction)
            const confirmation = await confirmTransaction(provider.connection, tx)
            if (confirmation.err) { throw new Error("❌ - Transaction not confirmed.") }
            console.log("Refunded all accounts: ", tx)
        }
    })
    it("can transfer tokens to raydium and create raydium pool", async () => {
        let [market, _] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("market"), marketCreator.publicKey.toBuffer(), seed.toArrayLike(Buffer, "le", 8)],
            program.programId
        )
        // const mint = anchor.web3.PublicKey.findProgramAddressSync(
        //     [Buffer.from("mint"), market.toBuffer()],
        //     program.programId
        // )[0]
        const wsolMint = new anchor.web3.PublicKey("So11111111111111111111111111111111111111112")

        // const eventListener = await program.addEventListener("tokenDataForRaydium", async (e, slot) => {
        // console.log("Event data got: ", e.tokenAmount, e.wsolAmount, e)

        try {
            const tokenAccount = getAssociatedTokenAddressSync(mint.publicKey, market, true, TOKEN_2022_PROGRAM_ID)
            const creatorTokenAccount = getAssociatedTokenAddressSync(mint.publicKey, owner.publicKey, false, TOKEN_2022_PROGRAM_ID)
            const tokenBalance = (await getAccount(provider.connection, tokenAccount, "confirmed", TOKEN_2022_PROGRAM_ID)).amount.toString()
            // console.log("OTKEN ACCOIUNT HERE: ", tokenAccount.toBase58(), tokenBalance)
            const marketVault = anchor.web3.PublicKey.findProgramAddressSync(
                [Buffer.from("vault"), market.toBuffer()],
                program.programId
            )[0]
            const marketVaultBalance = await provider.connection.getBalance(marketVault)
            console.log("BALANCES ARE: ", tokenBalance, marketVaultBalance)
            const tx1 = await program.methods.transferTokensToCreator()
                .accountsPartial({
                    signer: owner.publicKey,
                    marketCreator: marketCreator.publicKey,
                    market,
                    wsolMint,
                    creatorTokenAccount,
                    mint: mint.publicKey,
                    token0Program: TOKEN_PROGRAM_ID,
                    token1Program: TOKEN_2022_PROGRAM_ID,
                    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID
                })
                .signers([owner])
                .rpc()

            console.log("Transferred tokens to creator: ", tx1)
            const creatorToken = getAssociatedTokenAddressSync(mint.publicKey, owner.publicKey, false, TOKEN_2022_PROGRAM_ID)
            console.log("TOKEN ACCOUNT IN TESTS: ", creatorToken.toBase58())
            const { poolAddress, cpSwapPoolState, tx } = await initialize(
                program,
                owner.publicKey,
                configAddress,
                wsolMint,
                TOKEN_PROGRAM_ID,
                mint.publicKey,
                TOKEN_2022_PROGRAM_ID,
                { initAmount0: new anchor.BN(marketVaultBalance), initAmount1: new anchor.BN(tokenBalance) }
            );
            console.log("created raydium pool, ", tx)

        } catch (err) {
            console.log("Cannot transfer to raydium, ", err);
        }
        // });

        // Set a timeout so it doesn’t hang forever

        // const eventListener = program.addEventListener("tokenDataForRaydium", async (e, slot) => {
        //     console.log("Event data got: ", e.tokenAmount, e.wsolAmount)
        //     const wsolAmount = e.wsolAmount;
        //     const tokenAmount = e.tokenAmount;
        //     const { poolAddress, cpSwapPoolState, tx } = await initialize(
        //         program,
        //         owner.publicKey,
        //         configAddress,
        //         wsolMint,
        //         TOKEN_PROGRAM_ID,
        //         mint,
        //         TOKEN_PROGRAM_ID,
        //         { initAmount0: wsolAmount, initAmount1: tokenAmount }
        //     );
        //     console.log("created raydium pool, ", tx)
        // });
    })

    it("can close event listeners", async () => {
        await program.removeEventListener(stakeEvent)
    })

})