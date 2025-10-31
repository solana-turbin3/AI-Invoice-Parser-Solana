import * as anchor from "@coral-xyz/anchor";

async function main() {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = (anchor.workspace as any).InvoiceClaim as any;

  const wallet = provider.wallet as any;

  const [orgConfigPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("org_config"), wallet.publicKey.toBuffer()],
    program.programId
  );
  const [invoicePda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("invoice"), wallet.publicKey.toBuffer()],
    program.programId
  );

  const queueStr = process.env.QUEUE_PUBKEY;
  if (!queueStr) {
    console.error(
      "Missing QUEUE_PUBKEY env var. Set it to the VRF default queue public key (matches on-chain DEFAULT_QUEUE)."
    );
    process.exit(1);
  }
  const queuePk = new anchor.web3.PublicKey(queueStr);

  try {
    const tx = await program.methods
      .requestInvoiceAuditVrf(42)
      .accounts({
        payer: wallet.publicKey,
        orgConfig: orgConfigPda,
        invoiceAccount: invoicePda,
        oracleQueue: queuePk,
      })
      .rpc();
    console.log("VRF requested. Tx:", tx);
    console.log(`Explorer: https://explorer.solana.com/tx/${tx}?cluster=devnet`);
  } catch (e: any) {
    console.error("VRF request failed:", e.message || e);
    process.exit(1);
  }
}

main();

