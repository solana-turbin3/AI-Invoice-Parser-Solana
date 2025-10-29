use anchor_lang::prelude::*;
use crate::state::*;

pub fn org_init(
    ctx: Context<OrgInit>,
    treasury_vault: Pubkey,
    mint: Pubkey,
    per_invoice_cap: u64,
    daily_cap: u64,
    audit_rate_bps: u16,
) -> Result<()> {
    require!(per_invoice_cap > 0, InvoiceError::InvalidAmount);
    require!(daily_cap > 0, InvoiceError::InvalidAmount);
    require!(daily_cap >= per_invoice_cap, InvoiceError::CapExceeded);
    require!(audit_rate_bps <= 10_000, InvoiceError::InvalidAuditRate); // Max is 100%

    let cfg = &mut ctx.accounts.org_config;
    cfg.authority = ctx.accounts.authority.key();
    cfg.oracle_signer = ctx.accounts.authority.key();
    cfg.treasury_vault = treasury_vault;
    cfg.mint = mint;
    cfg.per_invoice_cap = per_invoice_cap;
    cfg.daily_cap = daily_cap;
    cfg.daily_spent = 0;
    cfg.last_reset_day = Clock::get()?.unix_timestamp / 86400;
    cfg.audit_rate_bps = audit_rate_bps;
    cfg.paused = false;
    cfg.invoice_counter = 0;
    cfg.version = 1;

    msg!("Organization initialized - authority: {}", cfg.authority);
    Ok(())
}

pub fn set_caps(ctx: Context<SetCaps>, per_invoice_cap: u64, daily_cap: u64) -> Result<()> {
    require!(per_invoice_cap > 0, InvoiceError::InvalidAmount);
    require!(daily_cap > 0, InvoiceError::InvalidAmount);
    require!(daily_cap >= per_invoice_cap, InvoiceError::CapExceeded);

    let cfg = &mut ctx.accounts.org_config;
    cfg.per_invoice_cap = per_invoice_cap;
    cfg.daily_cap = daily_cap;

    msg!("Caps updated");
    Ok(())
}

pub fn set_pause(ctx: Context<SetPause>, paused: bool) -> Result<()> {
    let cfg = &mut ctx.accounts.org_config;
    cfg.paused = paused;
    msg!("Pause state: {}", paused);
    Ok(())
}

pub fn set_oracle_signer(ctx: Context<SetOracleSigner>, oracle_signer: Pubkey) -> Result<()> {
    let cfg = &mut ctx.accounts.org_config;
    cfg.oracle_signer = oracle_signer;
    msg!("Oracle signer updated to: {}", oracle_signer);
    Ok(())
}
