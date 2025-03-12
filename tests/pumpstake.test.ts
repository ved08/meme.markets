import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { Pumpstake } from "../target/types/pumpstake";
import { randomBytes } from "crypto"
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
    let seed = new anchor.BN(100)

    it("create a new market", async () => {
        let [market, _] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("market"), owner.publicKey.toBuffer(), seed.toArrayLike(Buffer, "le", 8)],
            program.programId
        )
        console.log("market: ", market.toBase58())
        console.log("owner: ", owner.publicKey.toBase58())
        let startTime = new anchor.BN(Date.now())
        let endTime = new anchor.BN(Date.now() + 1000)
        const tx = await program.methods.createPredictionMarket(seed, 10, startTime, endTime, marketParams)
            .accountsPartial({
                signer: owner.publicKey,
                market
            })
            .signers([owner])
            .transaction()
        tx.recentBlockhash = (await provider.connection.getLatestBlockhash()).blockhash
        tx.feePayer = owner.publicKey
        console.log("Tx: ", tx.serializeMessage().toString("base64"))
    })
})