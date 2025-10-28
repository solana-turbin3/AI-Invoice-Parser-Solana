use std::env;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{read_keypair_file, Keypair, Signer};
use solana_sdk::system_program;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::transaction::Transaction;
use regex::Regex;
use solana_client::rpc_client::RpcClient;
use std::str::FromStr;
use std::time::Duration;
use dotenvy::dotenv;

#[derive(Clone, Debug)]
pub struct InvoiceRequest {
    pub authority: Pubkey,
    pub ipfs_hash: String,
    pub status: RequestStatus,
    pub timestamp: i64,
}

impl InvoiceRequest {
    pub fn from_account_data(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        if data.len() < 8 {
            return Err("Data too short".into());
        }

        let mut offset = 8;

        // Read authority (32 bytes)
        if offset + 32 > data.len() {
            return Err("Not enough data for authority".into());
        }
        let authority = Pubkey::try_from(&data[offset..offset + 32])?;
        offset += 32;

        // Read string length (4 bytes)
        if offset + 4 > data.len() {
            return Err("Not enough data for string length".into());
        }
        let str_len = u32::from_le_bytes([
            data[offset], data[offset + 1], data[offset + 2], data[offset + 3]
        ]) as usize;
        offset += 4;

        // Read string data
        if offset + str_len > data.len() {
            return Err("Not enough data for string".into());
        }
        let ipfs_hash = String::from_utf8(data[offset..offset + str_len].to_vec())?;
        offset += str_len;

        // Read status (1 byte)
        if offset >= data.len() {
            return Err("Not enough data for status".into());
        }
        let status = if data[offset] == 0 {
            RequestStatus::Pending
        } else {
            RequestStatus::Completed
        };
        offset += 1;

        // Read timestamp (8 bytes)
        if offset + 8 > data.len() {
            return Err("Not enough data for timestamp".into());
        }
        let timestamp = i64::from_le_bytes([
            data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
            data[offset + 4], data[offset + 5], data[offset + 6], data[offset + 7]
        ]);

        Ok(InvoiceRequest {
            authority,
            ipfs_hash,
            status,
            timestamp,
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RequestStatus {
    Pending,
    Completed,
}

const PROGRAM_ID: &str = "Cu675QqjfKaZiFDsiwJA3Hpa1r9MHdwfLfNKwhyp7TKo";
const RPC_URL: &str = "https://api.devnet.solana.com";

#[tokio::main]
async fn main() {
    println!("Invoice Oracle Backend Starting...");
    dotenv().ok();
    let keypair = read_keypair_file("oracle-keypair.json") //Add your keypair
        .expect("Failed to read keypair file");

    println!("Oracle wallet: {}", keypair.pubkey());

    let program_id = Pubkey::from_str(PROGRAM_ID).unwrap();
    let rpc_client = RpcClient::new(RPC_URL.to_string());

    println!("Watching program: {}", program_id);
    println!("Polling every 5 seconds...\n");

    let mut poll_count = 0;

    loop {
        poll_count += 1;
        println!("Poll #{} - Checking for new requests...", poll_count);

        match process_pending_requests(&rpc_client, &keypair, &program_id).await {
            Ok(processed) => {
                if processed > 0 {
                    println!("Processed {} requests", processed);
                } else {
                    println!("No pending requests found");
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }

        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

async fn process_pending_requests(
    rpc_client: &RpcClient,
    keypair: &Keypair,
    program_id: &Pubkey,
) -> Result<usize, Box<dyn std::error::Error>> {
    println!("Fetching program accounts...");
    // We probably need more filtration here
    let accounts = rpc_client.get_program_accounts(program_id)?;

    println!("Found {} total accounts for this program", accounts.len());

    let mut processed = 0;

    for (pubkey, account) in accounts {
        println!("Checking account: {}", pubkey);
        println!("Data length: {} bytes", account.data.len());

        if account.data.len() < 8 {
            println!("Account too small, skipping");
            continue;
        }

        println!("Discriminator: {:?}", &account.data[..8]);

        match InvoiceRequest::from_account_data(&account.data) {
            Ok(request) => {
                println!("Successfully deserialized");
                println!("Authority: {}", request.authority);
                println!("IPFS: {}", request.ipfs_hash);
                println!("Status: {:?}", request.status);

                if matches!(request.status, RequestStatus::Pending) {
                    println!("\nFound PENDING request!");

                    match extract_and_submit(rpc_client, keypair, program_id, &request, &pubkey).await {
                        Ok(_) => {
                            println!("Successfully processed!");
                            processed += 1;
                        }
                        Err(e) => {
                            eprintln!("Failed: {}", e);
                        }
                    }
                } else {
                    println!("Already completed, skipping");
                }
            }
            Err(e) => {
                println!("Failed to deserialize: {}", e);
            }
        }
    }

    Ok(processed)
}

async fn extract_and_submit(
    rpc_client: &RpcClient,
    keypair: &Keypair,
    program_id: &Pubkey,
    request: &InvoiceRequest,
    request_pubkey: &Pubkey,
) -> Result<(), Box<dyn std::error::Error>> {

    let api_key = env::var("OCR_API_KEY")
        .expect("OCR_API_KEY must be set in .env file");


    let ocr_url = format!(
        "https://api.ocr.space/parse/imageurl?apikey={}&url=https://emerald-abundant-baboon-978.mypinata.cloud/ipfs/{}&language=eng&OCREngine=2",
        api_key,
        request.ipfs_hash
    );

    println!("Calling OCR API...");
    let client = reqwest::Client::new();
    let response = client.get(&ocr_url).send().await?;
    let json: serde_json::Value = response.json().await?;


    println!("\n===== RAW OCR API RESPONSE =====");
    println!("{}", serde_json::to_string_pretty(&json)?);
    println!("================================\n");



    let ocr_text = json["ParsedResults"][0]["ParsedText"]
        .as_str()
        .ok_or("Failed to extract OCR text")?;
    println!("OCR Text extracted");

    let (vendor, amount, due_date) = parse_invoice(ocr_text);
    println!("Vendor: {}", vendor);
    println!("Amount: ${}", amount as f64 / 1_000_000.0);
    println!("Due Date: {}", due_date);

    let (invoice_pda, _) = Pubkey::find_program_address(
        &[b"invoice", request.authority.as_ref()],
        program_id,
    );

    // let mut data = vec![201, 158, 187, 152, 220, 106, 135, 48];
    let mut data = vec![69, 234, 33, 112, 18, 249, 177, 116];

    data.extend_from_slice(&(vendor.len() as u32).to_le_bytes());
    data.extend_from_slice(vendor.as_bytes());
    data.extend_from_slice(&amount.to_le_bytes());
    data.extend_from_slice(&due_date.to_le_bytes());

    let ix = Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new(*request_pubkey, false),
            AccountMeta::new(invoice_pda, false),
            AccountMeta::new(keypair.pubkey(), true),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data,
    };

    println!("Submitting to Solana...");
    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&keypair.pubkey()),
        &[keypair],
        recent_blockhash,
    );

    let signature = rpc_client.send_and_confirm_transaction(&tx)?;
    println!("Transaction: {}", signature);

    Ok(())
}

fn parse_invoice(text: &str) -> (String, u64, i64) {
    println!("\n===== PARSING INVOICE DATA =====");

    // Extract vendor name - look for "Bill to" followed by name on next line
    let vendor = if let Some(bill_to_pos) = text.find("Bill to") {
        // Get text after "Bill to"
        let after_bill_to = &text[bill_to_pos + 7..];

        // Split by newlines and get the first non-empty line
        let lines: Vec<&str> = after_bill_to.lines().collect();

        // The name should be on the next line after "Bill to"
        lines.iter()
            .skip(1) // Skip the "Bill to" line itself
            .find(|line| !line.trim().is_empty() && !line.contains("@")) // Skip empty lines and email
            .map(|line| line.trim().to_string())
            .unwrap_or_else(|| "Unknown Vendor".to_string())
    } else {
        // Fallback: Look for a capitalized name pattern
        let name_re = Regex::new(r"([A-Z][a-z]+\s+[A-Z][a-z]+)").unwrap();
        name_re.find(text)
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "Unknown Vendor".to_string())
    };

    println!("  Vendor Name: '{}'", vendor);

    // Extract amount - look for $XX.XX followed by "due"
    let amount_re = Regex::new(r"\$([0-9]+\.[0-9]{2})\s+due").unwrap();
    let amount_str = amount_re
        .captures(text)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str())
        .unwrap_or("0.00");
    let amount_float: f64 = amount_str.parse().unwrap_or(0.0);
    let amount = (amount_float * 1_000_000.0) as u64;

    println!("  Amount: ${} (raw: {})", amount as f64 / 1_000_000.0, amount_str);

    // Extract due date
    let date_re = Regex::new(r"(January|February|March|April|May|June|July|August|September|October|November|December)\s+(\d{1,2}),\s+(\d{4})").unwrap();
    let due_date = if let Some(caps) = date_re.captures(text) {
        let month = match &caps[1] {
            "January" => 1, "February" => 2, "March" => 3, "April" => 4,
            "May" => 5, "June" => 6, "July" => 7, "August" => 8,
            "September" => 9, "October" => 10, "November" => 11, "December" => 12,
            _ => 1,
        };
        let day: u32 = caps[2].parse().unwrap_or(1);
        let year: i32 = caps[3].parse().unwrap_or(2025);

        chrono::NaiveDate::from_ymd_opt(year, month, day)
            .and_then(|d| d.and_hms_opt(0, 0, 0))
            .map(|dt| dt.and_utc().timestamp())
            .unwrap_or(0)
    } else {
        0
    };

    if due_date > 0 {
        let date_str = chrono::NaiveDateTime::from_timestamp_opt(due_date, 0)
            .map(|dt| dt.format("%B %d, %Y").to_string())
            .unwrap_or_else(|| "Invalid".to_string());
        println!("  Due Date: {} (Unix: {})", date_str, due_date);
    } else {
        println!("  Due Date: Not found");
    }

    println!("================================\n");

    (vendor, amount, due_date)
}
