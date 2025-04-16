import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { Pumpstake } from "../target/types/pumpstake";
import { randomBytes } from "crypto"
import { str, struct, u64, u8 } from "@coral-xyz/borsh"
import { ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID } from "@solana/spl-token";
describe("initialize program tests", () => {
    const provider = anchor.AnchorProvider.env()
    anchor.setProvider(provider);
    const owner = anchor.Wallet.local().payer;
    const program = anchor.workspace.Pumpstake as Program<Pumpstake>;
    const TOKEN_METADATA_PROGRAM_ID = new anchor.web3.PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s")

    async function getSolanaClockTime(connection: anchor.web3.Connection) {
        const clockAccountInfo = await connection.getAccountInfo(anchor.web3.SYSVAR_CLOCK_PUBKEY);
        if (!clockAccountInfo) {
            throw new Error("Failed to fetch Solana Clock sysvar");
        }

        const unixTimestamp = clockAccountInfo.data.readBigInt64LE(8); // Offset 8 for unix_timestamp
        console.log("Solana On-Chain Time (Seconds):", unixTimestamp.toString());
        return Number(unixTimestamp);
    }

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
    class BettingOption {
        option_id: number;
        name: string;
        image: string;
        description: string;
        liquidity: anchor.BN;

        constructor(fields: {
            option_id: number;
            name: string;
            image: string;
            description: string;
            liquidity: anchor.BN;
        }) {
            Object.assign(this, fields);
        }
    }
    const BettingOptionSchema = new Map([
        [
            BettingOption,
            struct([
                u8("option_id"),
                str("name"),
                str("image"),
                str("description"),
                u64("liquidity"),
            ]),
        ],
    ]);
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
        const balance = await provider.connection.getBalance(anotherUser.publicKey)
        console.log("another user balance: ", balance)
    })

    it("can create a new coin toss market", async () => {
        let [market, _] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("market"), owner.publicKey.toBuffer(), seed.toArrayLike(Buffer, "le", 8)],
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
        const ix1 = await program.methods.createPredictionMarket(seed, totalOptions, duration, marketParams, optionDetails)
            .accountsPartial({
                signer: owner.publicKey,
                market
            })
            .instruction()
        // const headVault = anchor.web3.PublicKey.findProgramAddressSync(
        //     [Buffer.from("head"), seed.toArrayLike(Buffer, "le", 8), owner.publicKey.toBuffer()],
        //     program.programId
        // )[0]
        // const tailVault = anchor.web3.PublicKey.findProgramAddressSync(
        //     [Buffer.from("tail"), seed.toArrayLike(Buffer, "le", 8), owner.publicKey.toBuffer()],
        //     program.programId
        // )[0]
        // const ix2 = await program.methods.coinTossInitAccounts(seed).accountsPartial({
        //     payer: owner.publicKey,
        //     vault1: headVault,
        //     vault2: tailVault,

        // }).instruction()
        const instructions: anchor.web3.TransactionInstruction[] = [
            ix1
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
        console.log("Tx: ", tx)
    })
    it("can stake on heads", async () => {
        let betId = new anchor.BN(69420)
        let [market, _] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("market"), owner.publicKey.toBuffer(), seed.toArrayLike(Buffer, "le", 8)],
            program.programId
        )
        const bet = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("bet"), market.toBuffer(), owner.publicKey.toBuffer(), betId.toArrayLike(Buffer, "le", 8)],
            program.programId
        )[0]
        console.log("THIS IS BET ACCOUNT: ", bet.toBase58())
        const amount = new anchor.BN(anchor.web3.LAMPORTS_PER_SOL * 10)
        const option_id = 1 //lets assume 1 to be heads in coin toss
        const tx = await program.methods.stake(betId, option_id, amount)
            .accountsPartial({
                signer: owner.publicKey,
                market: market,
                bet: bet,

            }).signers([owner]).rpc()
        console.log("Sucessfully staked on heads: ", tx)
    })
    it("can stake on tails", async () => {
        let betId = new anchor.BN(69)
        let [market, _] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("market"), owner.publicKey.toBuffer(), seed.toArrayLike(Buffer, "le", 8)],
            program.programId
        )
        const bet = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("bet"), market.toBuffer(), owner.publicKey.toBuffer(), betId.toArrayLike(Buffer, "le", 8)],
            program.programId
        )[0]
        const amount = new anchor.BN(anchor.web3.LAMPORTS_PER_SOL * 100)
        const option_id = 0 // lets assume 0 to be tails
        const tx = await program.methods.stake(betId, option_id, amount)
            .accountsPartial({
                signer: owner.publicKey,
                market: market,
                bet: bet
            }).signers([owner]).rpc()
        console.log("Sucessfully staked on tails: ", tx)
        const data = await program.account.predictionMarket.fetch(market)
        console.log(data.marketOptions[0].liquidity.toNumber())
    })
    // it("can create a new polymarket bet with 5 options", async () => {
    //     let [market, _] = anchor.web3.PublicKey.findProgramAddressSync(
    //         [Buffer.from("market"), owner.publicKey.toBuffer(), seed2.toArrayLike(Buffer, "le", 8)],
    //         program.programId
    //     )
    //     console.log("market: ", market.toBase58())
    //     console.log("owner: ", owner.publicKey.toBase58())
    //     let duration = new anchor.BN(1000)
    //     let totalOptions = 5 // number of options for this market is 5 options
    //     const ix1 = await program.methods.createPredictionMarket(seed2, totalOptions, duration, marketParams)
    //         .accountsPartial({
    //             signer: owner.publicKey,
    //             market
    //         })
    //         .instruction()
    //     const instructions: anchor.web3.TransactionInstruction[] = [
    //         ix1
    //     ]
    //     let blockhash = (await provider.connection.getLatestBlockhash()).blockhash
    //     const messageV0 = new anchor.web3.TransactionMessage({
    //         payerKey: owner.publicKey,
    //         recentBlockhash: blockhash,
    //         instructions: instructions
    //     }).compileToV0Message()
    //     const transaction = new anchor.web3.VersionedTransaction(messageV0)
    //     transaction.sign([owner])
    //     const tx = await provider.connection.sendTransaction(transaction)
    //     const confirmation = await confirmTransaction(provider.connection, tx)
    //     if (confirmation.err) { throw new Error("❌ - Transaction not confirmed.") }
    //     console.log("Tx: ", tx)
    // })
    // it("can stake on option 4", async () => {
    //     let betId = new anchor.BN(5678)
    //     let [market, _] = anchor.web3.PublicKey.findProgramAddressSync(
    //         [Buffer.from("market"), owner.publicKey.toBuffer(), seed2.toArrayLike(Buffer, "le", 8)],
    //         program.programId
    //     )
    //     const bet = anchor.web3.PublicKey.findProgramAddressSync(
    //         [Buffer.from("bet"), market.toBuffer(), owner.publicKey.toBuffer(), betId.toArrayLike(Buffer, "le", 8)],
    //         program.programId
    //     )[0]
    //     console.log("THIS IS BET ACCOUNT: ", bet.toBase58())
    //     const amount = new anchor.BN(anchor.web3.LAMPORTS_PER_SOL * 10)
    //     const option_id = 3 //lets assume 1 to be heads in coin toss
    //     const tx = await program.methods.stake(betId, option_id, amount)
    //         .accountsPartial({
    //             signer: owner.publicKey,
    //             market: market,
    //             bet: bet,

    //         }).signers([owner]).rpc()
    //     console.log("Sucessfully staked on option 4: ", tx)
    // })
    // it("can stake on option 2", async () => {
    //     let betId = new anchor.BN(1234)
    //     let [market, _] = anchor.web3.PublicKey.findProgramAddressSync(
    //         [Buffer.from("market"), owner.publicKey.toBuffer(), seed2.toArrayLike(Buffer, "le", 8)],
    //         program.programId
    //     )
    //     const bet = anchor.web3.PublicKey.findProgramAddressSync(
    //         [Buffer.from("bet"), market.toBuffer(), owner.publicKey.toBuffer(), betId.toArrayLike(Buffer, "le", 8)],
    //         program.programId
    //     )[0]
    //     const amount = new anchor.BN(anchor.web3.LAMPORTS_PER_SOL * 5)
    //     const option_id = 1
    //     const tx = await program.methods.stake(betId, option_id, amount)
    //         .accountsPartial({
    //             signer: owner.publicKey,
    //             market: market,
    //             bet: bet
    //         }).signers([owner]).rpc()
    //     console.log("Sucessfully staked on option 2: ", tx)

    // })
    // it("another stake another on option 2", async () => {
    //     let betId = new anchor.BN(1111)
    //     let [market, _] = anchor.web3.PublicKey.findProgramAddressSync(
    //         [Buffer.from("market"), owner.publicKey.toBuffer(), seed2.toArrayLike(Buffer, "le", 8)],
    //         program.programId
    //     )
    //     const bet = anchor.web3.PublicKey.findProgramAddressSync(
    //         [Buffer.from("bet"), market.toBuffer(), anotherUser.publicKey.toBuffer(), betId.toArrayLike(Buffer, "le", 8)],
    //         program.programId
    //     )[0]
    //     const amount = new anchor.BN(anchor.web3.LAMPORTS_PER_SOL * 5)
    //     const option_id = 1
    //     const tx = await program.methods.stake(betId, option_id, amount)
    //         .accountsPartial({
    //             signer: anotherUser.publicKey,
    //             market: market,
    //             bet: bet
    //         }).signers([anotherUser]).rpc()
    //     console.log("Sucessfully staked on option 2: ", tx)
    //     const data = await program.account.predictionMarket.fetch(market)
    //     console.log("Staked on option heres the data: ", data)
    // })
    it("can resolve the coin toss market and mark a winner", async () => {
        await new Promise(resolve => setTimeout(resolve, 1050));
        console.log("Timeout completed. Now resolving market...");
        let [market, _] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("market"), owner.publicKey.toBuffer(), seed.toArrayLike(Buffer, "le", 8)],
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
        if (data.graduate) {
            console.log("-------------CREATING A COIN----------------")
            const mint = anchor.web3.PublicKey.findProgramAddressSync(
                [Buffer.from("mint"), market.toBuffer()],
                program.programId
            )[0]
            const [metadataAddress, _] = anchor.web3.PublicKey.findProgramAddressSync(
                [
                    Buffer.from("metadata"),
                    TOKEN_METADATA_PROGRAM_ID.toBuffer(),
                    mint.toBuffer(),
                ],
                TOKEN_METADATA_PROGRAM_ID
            );
            console.log("Metadata account is: ", metadataAddress.toBase58())
            let tx1 = await program.methods.createCoin()
                .accountsPartial({
                    market,
                    mint,
                    metadata: metadataAddress,
                    signer: owner.publicKey,
                    tokenProgram: TOKEN_PROGRAM_ID,
                    tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
                    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID
                })
                .signers([owner])
                .rpc()
            console.log("created and minted coins at reserve", tx1);
        } else {
            console.log("Market cannot graduate")
        }
    })
    it("can close and distribute rewards and amounts", async () => {
        let betId = new anchor.BN(69420)
        let [market, _] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("market"), owner.publicKey.toBuffer(), seed.toArrayLike(Buffer, "le", 8)],
            program.programId
        )
        const data = await program.account.predictionMarket.fetch(market)
        const bet = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("bet"), market.toBuffer(), owner.publicKey.toBuffer(), betId.toArrayLike(Buffer, "le", 8)],
            program.programId
        )[0]
        if (data.graduate == false) {
            let tx = await program.methods.claim()
                .accountsPartial({
                    bet,
                    market,
                    reciever: owner.publicKey
                }).signers([owner])
                .rpc()
            console.log("successfully refunded: ", tx)

        } else {
            let tx2 = await program.methods.claim2()
                .accountsPartial({
                    bet,
                    market,
                    receiver: owner.publicKey,
                    tokenProgram: TOKEN_PROGRAM_ID
                }).signers([owner])
                .rpc()
            console.log("Allocated tokens", tx2)
            const marketData = await program.account.predictionMarket.fetch(market)
            console.log("total winner liq: ", JSON.stringify(marketData))
        }
    })
    // it("can allocate tokens to another user", async () => {
    //     let betId = new anchor.BN(1111)
    //     let [market, _] = anchor.web3.PublicKey.findProgramAddressSync(
    //         [Buffer.from("market"), owner.publicKey.toBuffer(), seed2.toArrayLike(Buffer, "le", 8)],
    //         program.programId
    //     )
    //     const bet = anchor.web3.PublicKey.findProgramAddressSync(
    //         [Buffer.from("bet"), market.toBuffer(), anotherUser.publicKey.toBuffer(), betId.toArrayLike(Buffer, "le", 8)],
    //         program.programId
    //     )[0]
    //     let tx = await program.methods.claim2()
    //         .accountsPartial({
    //             bet,
    //             market,
    //         }).signers([owner])
    //         .rpc()
    //     console.log("Allocated tokens to another user", tx)
    //     const marketData = await program.account.predictionMarket.fetch(market)
    //     console.log("total winner liq: ", JSON.stringify(marketData))


    //     let betId2 = new anchor.BN(5678)
    //     const bet2 = anchor.web3.PublicKey.findProgramAddressSync(
    //         [Buffer.from("bet"), market.toBuffer(), owner.publicKey.toBuffer(), betId2.toArrayLike(Buffer, "le", 8)],
    //         program.programId)[0]
    //     tx = await program.methods.claim2()
    //         .accountsPartial({
    //             bet: bet2,
    //             market,
    //         }).signers([owner])
    //         .rpc()
    //     console.log("Allocated tokens to another another please check user", tx)
    //     const marketData2 = await program.account.predictionMarket.fetch(market)
    //     console.log("total winner liq: ", JSON.stringify(marketData2))
    // })
})