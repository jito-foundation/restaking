import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { GlobalCounterAvs } from "../target/types/global_counter_avs";

describe("global_counter_avs", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.GlobalCounterAvs as Program<GlobalCounterAvs>;

  it("Count", async () => {

    const globalCounterAddress = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("global_counter")], program.programId)[0];

    // Add your test here.
    {
      const tx = await program.methods.count().rpc();
    }
    {
      const tx = await program.methods.count().rpc();
    }

    const globalCounterAccount = await program.account.globalCounter.fetch(globalCounterAddress);

    console.log("Global counter is", globalCounterAccount.count.toString());
  });
});
