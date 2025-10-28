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
    let cfg = &mut ctx.accounts.org_config;
    cfg.authority = ctx.accounts.authority.key();
    cfg.oracle_signer = ctx.accounts.authority.key();
    cfg.treasury_vault = treasury_vault;
    cfg.mint = mint;
    cfg.per_invoice_cap = per_invoice_cap;
    cfg.daily_cap = daily_cap;
    cfg.audit_rate_bps = audit_rate_bps;
    cfg.paused = false;
    cfg.invoice_counter = 0;
    cfg.bump = ctx.bumps.org_config;
    cfg.version = 1;
    Ok(())
}

pub fn set_caps(ctx: Context<SetCaps>, per_invoice_cap: u64, daily_cap: u64) -> Result<()> {
    let cfg = &mut ctx.accounts.org_config;
    require_keys_eq!(cfg.authority, ctx.accounts.authority.key(), InvoiceError::Unauthorized);
    cfg.per_invoice_cap = per_invoice_cap;
    cfg.daily_cap = daily_cap;
    Ok(())
}

pub fn set_pause(ctx: Context<SetPause>, paused: bool) -> Result<()> {
    let cfg = &mut ctx.accounts.org_config;
    require_keys_eq!(cfg.authority, ctx.accounts.authority.key(), InvoiceError::Unauthorized);
    cfg.paused = paused;
    Ok(())
}

pub fn set_oracle_signer(ctx: Context<SetOracleSigner>, oracle_signer: Pubkey) -> Result<()> {
    let cfg = &mut ctx.accounts.org_config;
    require_keys_eq!(cfg.authority, ctx.accounts.authority.key(), InvoiceError::Unauthorized);
    cfg.oracle_signer = oracle_signer;
    Ok(())
}
