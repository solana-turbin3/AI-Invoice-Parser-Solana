use anchor_lang::prelude::*;
use anchor_spl::token::{self};
use crate::state::*;

pub fn fund_escrow(ctx: Context<FundEscrow>) -> Result<()> {
    let cfg = &ctx.accounts.org_config;
    require!(!cfg.paused, InvoiceError::OrgPaused);

    let inv = &mut ctx.accounts.invoice_account;
    let amount = inv.amount;
    require!(inv.status == InvoiceStatus::Validated, InvoiceError::InvalidStatus);
    require!(amount <= cfg.per_invoice_cap, InvoiceError::CapExceeded);

    // Ensure mint matches configuration
    require_keys_eq!(ctx.accounts.mint.key(), cfg.mint, InvoiceError::WrongMint);

    // Transfer tokens from payer to escrow
    let cpi_accounts = anchor_spl::token::Transfer {
        from: ctx.accounts.payer_ata.to_account_info(),
        to: ctx.accounts.escrow_ata.to_account_info(),
        authority: ctx.accounts.payer.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
    token::transfer(cpi_ctx, amount)?;

    inv.status = InvoiceStatus::InEscrow;
    Ok(())
}

pub fn settle_to_vendor(ctx: Context<SettleToVendor>) -> Result<()> {
    let cfg = &ctx.accounts.org_config;
    require!(!cfg.paused, InvoiceError::OrgPaused);

    let inv = &mut ctx.accounts.invoice_account;
    require!(inv.status == InvoiceStatus::InEscrow, InvoiceError::InvalidStatus);
    let amount = inv.amount;

    // Sign with escrow authority PDA derived from invoice key
    let bump = ctx.bumps.escrow_authority;
    let invoice_key = inv.key();
    let bump_seed = [bump];
    let signer_seeds: &[&[u8]] = &[b"escrow_auth", invoice_key.as_ref(), &bump_seed];
    let signer = &[signer_seeds];

    let cpi_accounts = anchor_spl::token::Transfer {
        from: ctx.accounts.escrow_ata.to_account_info(),
        to: ctx.accounts.vendor_ata.to_account_info(),
        authority: ctx.accounts.escrow_authority.to_account_info(),
    };
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
        signer,
    );
    token::transfer(cpi_ctx, amount)?;

    inv.status = InvoiceStatus::Paid;
    Ok(())
}
