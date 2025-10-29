use anchor_lang::prelude::*;
use crate::{InvoiceError, ManageVendor, RegisterVendor};

pub fn register_vendor(
    ctx: Context<RegisterVendor>,
    vendor_name: String,
    wallet: Pubkey,
) -> Result<()> {
    require!(!vendor_name.is_empty(), InvoiceError::InvalidVendor);
    require!(vendor_name.len() <= 50, InvoiceError::InvalidVendor);

    let vendor = &mut ctx.accounts.vendor_account;
    vendor.org = ctx.accounts.org_config.key();
    vendor.vendor_name = vendor_name.clone();
    vendor.wallet = wallet;
    vendor.total_paid = 0;
    vendor.last_payment = 0;
    vendor.is_active = true;
    vendor.currency_preference = ctx.accounts.org_config.mint;

    msg!("Vendor registered: {}", vendor_name);
    Ok(())
}


pub fn deactivate_vendor(ctx: Context<ManageVendor>) -> Result<()> {
    let vendor = &mut ctx.accounts.vendor_account;
    require!(vendor.is_active, InvoiceError::VendorInactive);

    vendor.is_active = false;
    msg!("Vendor deactivated: {}", vendor.vendor_name);
    Ok(())
}

pub fn activate_vendor(ctx: Context<ManageVendor>) -> Result<()> {
    let vendor = &mut ctx.accounts.vendor_account;
    require!(!vendor.is_active, InvoiceError::VendorInactive);

    vendor.is_active = true;
    msg!("Vendor activated: {}", vendor.vendor_name);
    Ok(())
}

pub fn update_vendor_wallet(
    ctx: Context<ManageVendor>,
    new_wallet: Pubkey,
) -> Result<()> {
    let vendor = &mut ctx.accounts.vendor_account;
    vendor.wallet = new_wallet;
    msg!("Vendor wallet updated for: {}", vendor.vendor_name);
    Ok(())
}
