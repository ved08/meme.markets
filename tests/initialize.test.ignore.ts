import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { Pumpstake } from "../target/types/pumpstake";
import { setupInitializeTest, initialize } from "./utils";

describe("initialize test", () => {
    anchor.setProvider(anchor.AnchorProvider.env());
    const owner = anchor.Wallet.local().payer;

    const program = anchor.workspace.Pumpstake as Program<Pumpstake>;

    const confirmOptions = {
        skipPreflight: true,
    };

    it("create pool", async () => {
        const { configAddress, token0, token0Program, token1, token1Program } =
            await setupInitializeTest(
                anchor.getProvider().connection,
                owner,
                { transferFeeBasisPoints: 0, MaxFee: 0 },
                confirmOptions
            );

        const initAmount0 = new BN(10000000000);
        const initAmount1 = new BN(20000000000);
        const { poolAddress, cpSwapPoolState, tx } = await initialize(
            program,
            owner.publicKey,
            configAddress,
            token0,
            token0Program,
            token1,
            token1Program,
            { initAmount0, initAmount1 },
            confirmOptions,
        );

        console.log("pool address: ", poolAddress.toString(), " tx:", tx);
        console.log(cpSwapPoolState)

    });
});