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

    let org_config = &mut ctx.accounts.org_config;

    // only authorized oracle can submit(this may backfire during testing :( )
    require_keys_eq!(
        ctx.accounts.payer.key(),
        org_config.oracle_signer,
        InvoiceError::Unauthorized
    );

    // Validate extracted data
    require!(amount > 0, InvoiceError::InvalidAmount);
    require!(amount <= org_config.per_invoice_cap, InvoiceError::CapExceeded);
    require!(!vendor_name.is_empty(), InvoiceError::InvalidVendor);
    require!(vendor_name.len() <= 50, InvoiceError::InvalidVendor);

    let current_time = Clock::get()?.unix_timestamp;
    require!(due_date > current_time, InvoiceError::InvalidDueDate);

    // Verify vendor is registered and active (CRITICAL for whitelist)
    let vendor = &ctx.accounts.vendor_account;
    require!(vendor.is_active, InvoiceError::VendorInactive);
    require_keys_eq!(vendor.org, org_config.key(), InvoiceError::WrongOrg);
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
