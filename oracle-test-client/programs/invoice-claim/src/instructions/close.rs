use anchor_lang::prelude::*;
use crate::state::*;

pub fn close_invoice(ctx: Context<CloseInvoice>) -> Result<()> {
    let invoice = &ctx.accounts.invoice_account;
    msg!("Closing invoice account for vendor: {}", invoice.vendor_name);
    msg!("Rent returned to: {}", ctx.accounts.authority.key());
    Ok(())
}

pub fn close_request(ctx: Context<CloseRequest>) -> Result<()> {
    let request = &ctx.accounts.invoice_request;
    msg!("Closing request account for IPFS: {}", request.ipfs_hash);
    msg!("Rent returned to: {}", ctx.accounts.authority.key());
    Ok(())
}
