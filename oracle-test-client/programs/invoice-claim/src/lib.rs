#![allow(unexpected_cfgs)]
#![allow(deprecated)]
use anchor_lang::prelude::*;

declare_id!("5zUiSUHNQCtxcSYtrbx7QqxCHLFBZy6Pgxt6w1bLKa9u");

mod state;
mod instructions;

pub use crate::state::*;
pub use crate::instructions::*;


#[program]
pub mod invoice_claim {
    use super::*;

    // Invoice request + OCR fulfillment
    pub fn request_invoice_extraction(ctx: Context<RequestExtraction>, ipfs_hash: String,amount: u64) -> Result<()> {
        instructions::invoice::request_invoice_extraction(ctx, ipfs_hash,amount)
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

    // Update Org Config
    pub fn update_org_config(ctx: Context<UpdateOrgConfig>, update_args: UpdateOrgConfigArgs) -> Result<()> {
        instructions::org::update_org_config(ctx, update_args)
    }

    // Escrow MVP
    pub fn fund_escrow(ctx: Context<FundEscrow>) -> Result<()> {
        instructions::escrow::fund_escrow(ctx)
    }

    pub fn settle_to_vendor(ctx: Context<SettleToVendor>) -> Result<()> {
        instructions::escrow::settle_to_vendor(ctx)
    }

    //Vendor Management
    pub fn register_vendor(
        ctx: Context<RegisterVendor>,
        vendor_name: String,
        wallet: Pubkey,
    ) -> Result<()> {
        instructions::vendor::register_vendor(ctx, vendor_name, wallet)
    }

    pub fn deactivate_vendor(ctx: Context<ManageVendor>) -> Result<()> {
        instructions::vendor::deactivate_vendor(ctx)
    }

    pub fn activate_vendor(ctx: Context<ManageVendor>) -> Result<()> {
        instructions::vendor::activate_vendor(ctx)
    }

    pub fn update_vendor_wallet(ctx: Context<ManageVendor>, new_wallet: Pubkey) -> Result<()> {
        instructions::vendor::update_vendor_wallet(ctx, new_wallet)
    }


}
