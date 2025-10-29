use anchor_lang::prelude::*;
use crate::state::*;

pub fn request_invoice_extraction(
    ctx: Context<RequestExtraction>,
    ipfs_hash: String,
    amount: u64
) -> Result<()> {
    let request = &mut ctx.accounts.invoice_request;
    request.authority = ctx.accounts.authority.key();
    request.ipfs_hash = ipfs_hash.clone();
    request.status = RequestStatus::Pending;
    request.timestamp = Clock::get()?.unix_timestamp;
    request.amount = amount;
    msg!("Invoice extraction requested for IPFS: {}", ipfs_hash);
    Ok(())
}

pub fn process_extraction_result(
    ctx: Context<ProcessResult>,
    vendor_name: String,
    amount: u64,
    due_date: i64,
) -> Result<()> {
    let invoice = &mut ctx.accounts.invoice_account;
    let request = &mut ctx.accounts.invoice_request;

    invoice.authority = request.authority;
    invoice.vendor_name = vendor_name;
    invoice.amount = amount;
    invoice.due_date = due_date;
    invoice.ipfs_hash = request.ipfs_hash.clone();
    invoice.status = InvoiceStatus::Validated;
    invoice.timestamp = Clock::get()?.unix_timestamp;

    request.status = RequestStatus::Completed;
    msg!("Invoice processed: {} - ${}", invoice.vendor_name, invoice.amount);
    Ok(())
}
