use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct InvoiceRequest {
    pub authority: Pubkey,
    #[max_len(64)]
    pub ipfs_hash: String,
    pub status: RequestStatus,
    pub timestamp: i64,
}

#[account]
#[derive(InitSpace)]
pub struct InvoiceAccount {
    pub authority: Pubkey,
    #[max_len(50)]
    pub vendor_name: String,
    pub amount: u64,
    pub due_date: i64,
    #[max_len(64)]
    pub ipfs_hash: String,
    pub status: InvoiceStatus,
    pub timestamp: i64,
}

#[account]
#[derive(InitSpace)]
pub struct OrgConfig {
    pub authority: Pubkey,
    pub oracle_signer: Pubkey,
    pub treasury_vault: Pubkey,
    pub mint: Pubkey,
    pub per_invoice_cap: u64,
    pub daily_cap: u64,
    pub audit_rate_bps: u16,
    pub paused: bool,
    pub invoice_counter: u64,
    pub bump: u8,
    pub version: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum RequestStatus {
    Pending,
    Completed,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum InvoiceStatus {
    Validated,
    InEscrow,
    Paid,
}

#[error_code]
pub enum InvoiceError {
    #[msg("Invoice status is invalid for this operation")]
    InvalidStatus,
    #[msg("Payment is overdue")]
    PaymentOverdue,
    #[msg("Organization is paused")]
    OrgPaused,
    #[msg("Per-invoice cap exceeded")]
    CapExceeded,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Wrong mint for this organization")]
    WrongMint,
}

// ========== ACCOUNT CONTEXTS (moved here to keep only state + instructions) ==========

use anchor_spl::token::Token;

#[derive(Accounts)]
#[instruction(ipfs_hash: String)]
pub struct RequestExtraction<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + InvoiceRequest::INIT_SPACE,
        seeds = [b"request", authority.key().as_ref()],
        bump
    )]
    pub invoice_request: Account<'info, InvoiceRequest>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ProcessResult<'info> {
    #[account(
        mut,
        seeds = [b"request", invoice_request.authority.as_ref()],
        bump
    )]
    pub invoice_request: Account<'info, InvoiceRequest>,

    #[account(
        init,
        payer = payer,
        space = 8 + InvoiceAccount::INIT_SPACE,
        seeds = [b"invoice", invoice_request.authority.as_ref()],
        bump
    )]
    pub invoice_account: Account<'info, InvoiceAccount>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ProcessPayment<'info> {
    #[account(
        mut,
        seeds = [b"invoice", authority.key().as_ref()],
        bump,
        has_one = authority
    )]
    pub invoice_account: Account<'info, InvoiceAccount>,

    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct CompletePayment<'info> {
    #[account(
        mut,
        seeds = [b"invoice", authority.key().as_ref()],
        bump,
        has_one = authority
    )]
    pub invoice_account: Account<'info, InvoiceAccount>,

    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct CloseInvoice<'info> {
    #[account(
        mut,
        close = authority,
        seeds = [b"invoice", authority.key().as_ref()],
        bump,
        has_one = authority
    )]
    pub invoice_account: Account<'info, InvoiceAccount>,

    #[account(mut)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct CloseRequest<'info> {
    #[account(
        mut,
        close = authority,
        seeds = [b"request", authority.key().as_ref()],
        bump,
        has_one = authority
    )]
    pub invoice_request: Account<'info, InvoiceRequest>,

    #[account(mut)]
    pub authority: Signer<'info>,
}

// Org config

#[derive(Accounts)]
pub struct OrgInit<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + OrgConfig::INIT_SPACE,
        seeds = [b"org_config", authority.key().as_ref()],
        bump
    )]
    pub org_config: Account<'info, OrgConfig>,

    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetCaps<'info> {
    #[account(
        mut,
        seeds = [b"org_config", authority.key().as_ref()],
        bump
    )]
    pub org_config: Account<'info, OrgConfig>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct SetPause<'info> {
    #[account(
        mut,
        seeds = [b"org_config", authority.key().as_ref()],
        bump
    )]
    pub org_config: Account<'info, OrgConfig>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct SetOracleSigner<'info> {
    #[account(
        mut,
        seeds = [b"org_config", authority.key().as_ref()],
        bump
    )]
    pub org_config: Account<'info, OrgConfig>,
    pub authority: Signer<'info>,
}

// Escrow MVP

#[derive(Accounts)]
pub struct FundEscrow<'info> {
    #[account(
        mut,
        seeds = [b"org_config", org_config.authority.as_ref()],
        bump = org_config.bump
    )]
    pub org_config: Account<'info, OrgConfig>,

    #[account(
        mut,
        seeds = [b"invoice", authority.key().as_ref()],
        bump,
        has_one = authority @ InvoiceError::Unauthorized
    )]
    pub invoice_account: Account<'info, InvoiceAccount>,

    /// CHECK: PDA only used as signing authority
    #[account(
        seeds = [b"escrow_auth", invoice_account.key().as_ref()],
        bump
    )]
    pub escrow_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: must equal invoice_account.authority (validated by constraint above)
    pub authority: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: payer's SPL token account (validated at runtime)
    pub payer_ata: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: escrow SPL token account owned by escrow authority PDA
    pub escrow_ata: UncheckedAccount<'info>,
    /// CHECK: SPL token mint; key checked against OrgConfig
    pub mint: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct SettleToVendor<'info> {
    #[account(
        mut,
        seeds = [b"org_config", org_config.authority.as_ref()],
        bump = org_config.bump
    )]
    pub org_config: Account<'info, OrgConfig>,

    #[account(
        mut,
        seeds = [b"invoice", authority.key().as_ref()],
        bump,
        has_one = authority @ InvoiceError::Unauthorized
    )]
    pub invoice_account: Account<'info, InvoiceAccount>,

    /// CHECK: PDA only used as signing authority
    #[account(
        seeds = [b"escrow_auth", invoice_account.key().as_ref()],
        bump
    )]
    pub escrow_authority: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: vendor's SPL token account (validated at runtime)
    pub vendor_ata: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: escrow SPL token account owned by escrow authority PDA
    pub escrow_ata: UncheckedAccount<'info>,
    /// CHECK: SPL token mint
    pub mint: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,

    /// The invoice owner must authorize settlement
    pub authority: Signer<'info>,
}
