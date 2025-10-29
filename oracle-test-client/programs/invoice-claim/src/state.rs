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
    pub authority: Pubkey,              // Invoice owner
    pub vendor: Pubkey,                 // Links to VendorAccount
    #[max_len(50)]
    pub vendor_name: String,
    pub amount: u64,
    pub due_date: i64,
    #[max_len(64)]
    pub ipfs_hash: String,
    pub status: InvoiceStatus,
    pub timestamp: i64,
}

//A singleton state that manages the full protocol
#[account]
#[derive(InitSpace)]
pub struct OrgConfig {
    pub authority: Pubkey,
    pub oracle_signer: Pubkey,
    pub treasury_vault: Pubkey,
    pub mint: Pubkey,
    pub per_invoice_cap: u64,
    pub daily_cap: u64,
    pub daily_spent: u64,               // Track daily spending
    pub last_reset_day: i64,            // Last day caps were reset
    pub audit_rate_bps: u16,            // Basis points (e.g., 500 = 5%)
    pub paused: bool,
    pub invoice_counter: u64,
    pub version: u8,
}


#[account]
#[derive(InitSpace)]
pub struct VendorAccount {
    pub org: Pubkey,                    // Links to OrgConfig
    #[max_len(50)]
    pub vendor_name: String,            // Vendor identifier (matches invoice.vendor_name)
    pub wallet: Pubkey,                 // Where to send payments
    pub total_paid: u64,                // Lifetime payment tracking
    pub last_payment: i64,              // Unix timestamp of last payment
    pub is_active: bool,                // Can be disabled to block payments
    pub currency_preference: Pubkey,    // Preferred mint (for multi-currency)
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
    #[msg("Payment is not yet due")]
    PaymentNotDue,
    #[msg("Organization is paused")]
    OrgPaused,
    #[msg("Per-invoice cap exceeded")]
    CapExceeded,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Wrong mint for this organization")]
    WrongMint,
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Invalid vendor name")]
    InvalidVendor,
    #[msg("Vendor is not active")]
    VendorInactive,
    #[msg("Due date must be in the future")]
    InvalidDueDate,
    #[msg("Arithmetic overflow")]
    Overflow,
    #[msg("Invalid audit rate (must be 0-10000 bps)")]
    InvalidAuditRate,
    #[msg("Wrong organization")]
    WrongOrg,
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
#[instruction(vendor_name: String)]  //needed for vendor PDA derivation
pub struct ProcessResult<'info> {
    // OrgConfig for oracle authorization and invoice counter
    #[account(mut)]
    pub org_config: Account<'info, OrgConfig>,

    // VendorAccount to validate vendor is registered and active
    #[account(
        seeds = [b"vendor", org_config.key().as_ref(), vendor_name.as_bytes()],
        bump
    )]
    pub vendor_account: Account<'info, VendorAccount>,

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
        has_one = authority @ InvoiceError::Unauthorized,
    )]
    pub org_config: Account<'info, OrgConfig>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct SetPause<'info> {
    #[account(
        mut,
        has_one = authority @ InvoiceError::Unauthorized,
    )]
    pub org_config: Account<'info, OrgConfig>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct SetOracleSigner<'info> {
    #[account(
        mut,
        has_one = authority @ InvoiceError::Unauthorized,
    )]
    pub org_config: Account<'info, OrgConfig>,
    pub authority: Signer<'info>,
}

// Escrow MVP

#[derive(Accounts)]
pub struct FundEscrow<'info> {
    #[account(
        mut
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
        mut
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


/// ALL VENDOR IX STATE GOES HERE
/// 
/// 
/// ALL VENDOR IX STATE GOES HERE
#[derive(Accounts)]
#[instruction(vendor_name: String)]
pub struct RegisterVendor<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + VendorAccount::INIT_SPACE,
        seeds = [b"vendor", org_config.key().as_ref(), vendor_name.as_bytes()],
        bump
    )]
    pub vendor_account: Account<'info, VendorAccount>,

    #[account(
        has_one = authority
    )]
    pub org_config: Account<'info, OrgConfig>,

    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ManageVendor<'info> {
    #[account(
        mut,
        seeds = [b"vendor", org_config.key().as_ref(), vendor_account.vendor_name.as_bytes()],
        bump
    )]
    pub vendor_account: Account<'info, VendorAccount>,

    #[account(
        has_one = authority @ InvoiceError::Unauthorized
    )]
    pub org_config: Account<'info, OrgConfig>,

    pub authority: Signer<'info>,
}
