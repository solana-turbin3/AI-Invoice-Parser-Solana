use anchor_lang::prelude::*;

declare_id!("Cu675QqjfKaZiFDsiwJA3Hpa1r9MHdwfLfNKwhyp7TKo");

#[program]
pub mod invoice_claim {
    use super::*;

    /// User submits IPFS hash for invoice extraction
    pub fn request_invoice_extraction(
        ctx: Context<RequestExtraction>,
        ipfs_hash: String,
    ) -> Result<()> {
        let request = &mut ctx.accounts.invoice_request;
        request.authority = ctx.accounts.authority.key();
        request.ipfs_hash = ipfs_hash.clone();
        request.status = RequestStatus::Pending;
        request.timestamp = Clock::get()?.unix_timestamp;

        msg!("Invoice extraction requested for IPFS: {}", ipfs_hash);

        Ok(())
    }

    /// Oracle submits extracted data from OCR
    pub fn process_extraction_result(
        ctx: Context<ProcessResult>,
        vendor_name: String,
        amount: u64,
        due_date: i64,
    ) -> Result<()> {
        let invoice = &mut ctx.accounts.invoice_account;
        let request = &mut ctx.accounts.invoice_request;

        // Populate invoice with OCR data
        invoice.authority = request.authority;
        invoice.vendor_name = vendor_name;
        invoice.amount = amount;
        invoice.due_date = due_date;
        invoice.ipfs_hash = request.ipfs_hash.clone();
        invoice.status = InvoiceStatus::Validated;
        invoice.timestamp = Clock::get()?.unix_timestamp;

        // Mark request as completed
        request.status = RequestStatus::Completed;

        msg!("Invoice processed: {} - ${}", invoice.vendor_name, invoice.amount);

        Ok(())
    }

    /// Process invoice payment and move to escrow
    pub fn process_invoice_payment(
        ctx: Context<ProcessPayment>,
    ) -> Result<()> {
        let invoice = &ctx.accounts.invoice_account;

        // Verify invoice is validated
        require!(
            invoice.status == InvoiceStatus::Validated,
            InvoiceError::InvalidStatus
        );

        // Check if payment is overdue
        let current_time = Clock::get()?.unix_timestamp;
        require!(
            current_time <= invoice.due_date,
            InvoiceError::PaymentOverdue
        );

        // Log the invoice details being processed
        msg!("Processing payment for invoice:");
        msg!("  Vendor: {}", invoice.vendor_name);
        msg!("  Amount: {}", invoice.amount);
        msg!("  Due Date: {}", invoice.due_date);

        // Update invoice status to InEscrow
        let invoice_mut = &mut ctx.accounts.invoice_account;
        invoice_mut.status = InvoiceStatus::InEscrow;

        msg!("Invoice moved to escrow");

        Ok(())
    }

    /// Complete payment and mark invoice as paid
    pub fn complete_payment(
        ctx: Context<CompletePayment>,
    ) -> Result<()> {
        let invoice = &mut ctx.accounts.invoice_account;

        // Verify invoice is in escrow
        require!(
            invoice.status == InvoiceStatus::InEscrow,
            InvoiceError::InvalidStatus
        );

        // Mark as paid
        invoice.status = InvoiceStatus::Paid;

        msg!("Payment completed for vendor: {}", invoice.vendor_name);

        Ok(())
    }

    /// Close invoice account and return rent to authority
    pub fn close_invoice(ctx: Context<CloseInvoice>) -> Result<()> {
        let invoice = &ctx.accounts.invoice_account;

        msg!("Closing invoice account for vendor: {}", invoice.vendor_name);
        msg!("Rent returned to: {}", ctx.accounts.authority.key());

        Ok(())
    }

    /// Close request account and return rent to authority
    pub fn close_request(ctx: Context<CloseRequest>) -> Result<()> {
        let request = &ctx.accounts.invoice_request;

        msg!("Closing request account for IPFS: {}", request.ipfs_hash);
        msg!("Rent returned to: {}", ctx.accounts.authority.key());

        Ok(())
    }
}

// ========== ACCOUNT CONTEXTS ==========

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

// ========== DATA STRUCTURES ==========

#[account]
#[derive(InitSpace)]
pub struct InvoiceRequest {
    pub authority: Pubkey,           // 32 bytes
    #[max_len(64)]
    pub ipfs_hash: String,           // 64 bytes
    pub status: RequestStatus,       // 1 byte
    pub timestamp: i64,              // 8 bytes
}

#[account]
#[derive(InitSpace)]
pub struct InvoiceAccount {
    pub authority: Pubkey,           // 32 bytes
    #[max_len(50)]
    pub vendor_name: String,         // 50 bytes
    pub amount: u64,                 // 8 bytes
    pub due_date: i64,               // 8 bytes
    #[max_len(64)]
    pub ipfs_hash: String,           // 64 bytes
    pub status: InvoiceStatus,       // 1 byte
    pub timestamp: i64,              // 8 bytes
}

// ========== ENUMS ==========

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

// ========== ERRORS ==========

#[error_code]
pub enum InvoiceError {
    #[msg("Invoice status is invalid for this operation")]
    InvalidStatus,
    #[msg("Payment is overdue")]
    PaymentOverdue,
}
