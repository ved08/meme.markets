import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { Pumpstake } from "../target/types/pumpstake";
import { randomBytes } from "crypto"
import { am } from "@raydium-io/raydium-sdk-v2/lib/api-373aef5f";
describe("initialize program tests", () => {
    const provider = anchor.AnchorProvider.env()
    anchor.setProvider(provider);
    const owner = anchor.Wallet.local().payer;

    const program = anchor.workspace.Pumpstake as Program<Pumpstake>;

    let marketParams = {
        ticker: "Test",
        name: "Hello",
        image: "test",
        description: "yoooo",
        twitter: "x.com",
        website: "x.com",
        telegram: "telegram.org",
    }
    let seed = new anchor.BN(randomBytes(8))
    let seed2 = new anchor.BN(randomBytes(8))
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

    it("can create a new coin toss market", async () => {
        let [market, _] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("market"), owner.publicKey.toBuffer(), seed.toArrayLike(Buffer, "le", 8)],
            program.programId
        )
        console.log("market: ", market.toBase58())
        console.log("owner: ", owner.publicKey.toBase58())
        let startTime = new anchor.BN(Date.now())
        let endTime = new anchor.BN(Date.now() + 1000)
        let totalOptions = 2 // number of options for coin toss is 2
        const ix1 = await program.methods.createPredictionMarket(seed, totalOptions, startTime, endTime, marketParams)
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
        let betId = new anchor.BN(randomBytes(8))
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
        let betId = new anchor.BN(randomBytes(8))
        let [market, _] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("market"), owner.publicKey.toBuffer(), seed.toArrayLike(Buffer, "le", 8)],
            program.programId
        )
        const bet = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("bet"), market.toBuffer(), owner.publicKey.toBuffer(), betId.toArrayLike(Buffer, "le", 8)],
            program.programId
        )[0]
        const amount = new anchor.BN(anchor.web3.LAMPORTS_PER_SOL * 5)
        const option_id = 0 // lets assume 0 to be tails
        const tx = await program.methods.stake(betId, option_id, amount)
            .accountsPartial({
                signer: owner.publicKey,
                market: market,
                bet: bet
            }).signers([owner]).rpc()
        console.log("Sucessfully staked on tails: ", tx)
    })
    it("can create a new polymarket bet with 5 options", async () => {
        let [market, _] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("market"), owner.publicKey.toBuffer(), seed2.toArrayLike(Buffer, "le", 8)],
            program.programId
        )
        console.log("market: ", market.toBase58())
        console.log("owner: ", owner.publicKey.toBase58())
        let startTime = new anchor.BN(Date.now())
        let endTime = new anchor.BN(Date.now() + 1000)
        let totalOptions = 5 // number of options for this market is 5 options
        const ix1 = await program.methods.createPredictionMarket(seed2, totalOptions, startTime, endTime, marketParams)
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
    it("can stake on option 4", async () => {
        let betId = new anchor.BN(randomBytes(8))
        let [market, _] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("market"), owner.publicKey.toBuffer(), seed2.toArrayLike(Buffer, "le", 8)],
            program.programId
        )
        const bet = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("bet"), market.toBuffer(), owner.publicKey.toBuffer(), betId.toArrayLike(Buffer, "le", 8)],
            program.programId
        )[0]
        console.log("THIS IS BET ACCOUNT: ", bet.toBase58())
        const amount = new anchor.BN(anchor.web3.LAMPORTS_PER_SOL * 10)
        const option_id = 3 //lets assume 1 to be heads in coin toss
        const tx = await program.methods.stake(betId, option_id, amount)
            .accountsPartial({
                signer: owner.publicKey,
                market: market,
                bet: bet,

            }).signers([owner]).rpc()
        console.log("Sucessfully staked on option 4: ", tx)
    })
    it("can stake on option 2", async () => {
        let betId = new anchor.BN(randomBytes(8))
        let [market, _] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("market"), owner.publicKey.toBuffer(), seed2.toArrayLike(Buffer, "le", 8)],
            program.programId
        )
        const bet = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("bet"), market.toBuffer(), owner.publicKey.toBuffer(), betId.toArrayLike(Buffer, "le", 8)],
            program.programId
        )[0]
        const amount = new anchor.BN(anchor.web3.LAMPORTS_PER_SOL * 5)
        const option_id = 1 // lets assume 0 to be tails
        const tx = await program.methods.stake(betId, option_id, amount)
            .accountsPartial({
                signer: owner.publicKey,
                market: market,
                bet: bet
            }).signers([owner]).rpc()
        console.log("Sucessfully staked on option 2: ", tx)
    })
})