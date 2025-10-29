use anchor_lang::prelude::*;

declare_id!("5zUiSUHNQCtxcSYtrbx7QqxCHLFBZy6Pgxt6w1bLKa9u");

mod state;
mod instructions;

pub use crate::state::*;
// contexts live in state.rs to keep just `state` + `instructions`

#[program]
pub mod invoice_claim {
    use super::*;

    // Invoice request + OCR fulfillment
    pub fn request_invoice_extraction(ctx: Context<RequestExtraction>, ipfs_hash: String) -> Result<()> {
        instructions::invoice::request_invoice_extraction(ctx, ipfs_hash)
    }

    pub fn process_extraction_result(
        ctx: Context<ProcessResult>,
        vendor_name: String,
        amount: u64,
        due_date: i64,
    ) -> Result<()> {
        instructions::invoice::process_extraction_result(ctx, vendor_name, amount, due_date)
    }

    // Status-only payment flow
    pub fn process_invoice_payment(ctx: Context<ProcessPayment>) -> Result<()> {
        instructions::payments::process_invoice_payment(ctx)
    }

    pub fn complete_payment(ctx: Context<CompletePayment>) -> Result<()> {
        instructions::payments::complete_payment(ctx)
    }

    // Close accounts
    pub fn close_invoice(ctx: Context<CloseInvoice>) -> Result<()> {
        instructions::close::close_invoice(ctx)
    }

    pub fn close_request(ctx: Context<CloseRequest>) -> Result<()> {
        instructions::close::close_request(ctx)
    }

    // Org config
    pub fn org_init(
        ctx: Context<OrgInit>,
        treasury_vault: Pubkey,
        mint: Pubkey,
        per_invoice_cap: u64,
        daily_cap: u64,
        audit_rate_bps: u16,
    ) -> Result<()> {
        instructions::org::org_init(ctx, treasury_vault, mint, per_invoice_cap, daily_cap, audit_rate_bps)
    }

    pub fn set_caps(ctx: Context<SetCaps>, per_invoice_cap: u64, daily_cap: u64) -> Result<()> {
        instructions::org::set_caps(ctx, per_invoice_cap, daily_cap)
    }

    pub fn set_pause(ctx: Context<SetPause>, paused: bool) -> Result<()> {
        instructions::org::set_pause(ctx, paused)
    }

    pub fn set_oracle_signer(ctx: Context<SetOracleSigner>, oracle_signer: Pubkey) -> Result<()> {
        instructions::org::set_oracle_signer(ctx, oracle_signer)
    }

    // Escrow MVP
    pub fn fund_escrow(ctx: Context<FundEscrow>, amount: u64) -> Result<()> {
        instructions::escrow::fund_escrow(ctx, amount)
    }

    pub fn settle_to_vendor(ctx: Context<SettleToVendor>, amount: u64) -> Result<()> {
        instructions::escrow::settle_to_vendor(ctx, amount)
    }
}
