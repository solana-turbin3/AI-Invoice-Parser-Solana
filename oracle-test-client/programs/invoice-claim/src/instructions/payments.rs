use anchor_lang::prelude::*;
use crate::state::*;

pub fn process_invoice_payment(ctx: Context<ProcessPayment>) -> Result<()> {
    let invoice = &ctx.accounts.invoice_account;
    require!(invoice.status == InvoiceStatus::Validated, InvoiceError::InvalidStatus);

    let current_time = Clock::get()?.unix_timestamp;
    require!(current_time <= invoice.due_date, InvoiceError::PaymentOverdue);

    msg!("Processing payment for invoice:");
    msg!("  Vendor: {}", invoice.vendor_name);
    msg!("  Amount: {}", invoice.amount);
    msg!("  Due Date: {}", invoice.due_date);

    let invoice_mut = &mut ctx.accounts.invoice_account;
    invoice_mut.status = InvoiceStatus::InEscrow;
    msg!("Invoice moved to escrow");
    Ok(())
}

pub fn complete_payment(ctx: Context<CompletePayment>) -> Result<()> {
    let invoice = &mut ctx.accounts.invoice_account;
    require!(invoice.status == InvoiceStatus::InEscrow, InvoiceError::InvalidStatus);
    invoice.status = InvoiceStatus::Paid;
    msg!("Payment completed for vendor: {}", invoice.vendor_name);
    Ok(())
}
