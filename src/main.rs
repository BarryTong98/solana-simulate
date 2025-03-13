use std::fs::File;
use std::io::Read;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use solana_program_runtime::__private::ReadableAccount;
use solana_sdk::bpf_loader_upgradeable::get_program_data_address;
use solana_sdk::instruction::AccountMeta;
use solana_sdk::pubkey::Pubkey;
use solana_simulate::{create_program_accounts, create_simulation_environment, simulate_program, simulate_transaction_with_accounts};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Read program data
    let program_data = std::fs::read("/Users/barry/binance/solana-simulate/src/program.so")?;

    // 2. Create program ID
    let program_id = Pubkey::new_unique();

    // 3. Create program accounts
    let (program_account, programdata_account) = create_program_accounts(&program_id, &program_data);

    // 4. Read accounts.json file
    let mut accounts_file = File::open("./accounts.json")?;
    let mut accounts_json = String::new();
    accounts_file.read_to_string(&mut accounts_json)?;

    // 5. Parse JSON account data
    let mut accounts_data: serde_json::Value = serde_json::from_str(&accounts_json)?;

    // 6. Add program accounts to JSON data
    let programdata_address = get_program_data_address(&program_id);

    // Add Program Account
    let program_account_json = serde_json::json!({
        "pubkey": program_id.to_string(),
        "account": {
            "lamports": program_account.lamports(),
            "data": [BASE64_STANDARD.encode(program_account.data()), "base64"],
            "owner": program_account.owner().to_string(),
            "executable": program_account.executable(),
            "rentEpoch": program_account.rent_epoch(),
            "space": program_account.data().len()
        }
    });

    // Add Program Data Account
    let programdata_account_json = serde_json::json!({
        "pubkey": programdata_address.to_string(),
        "account": {
            "lamports": programdata_account.lamports(),
            "data": [BASE64_STANDARD.encode(programdata_account.data()), "base64"],
            "owner": programdata_account.owner().to_string(),
            "executable": programdata_account.executable(),
            "rentEpoch": programdata_account.rent_epoch(),
            "space": programdata_account.data().len()
        }
    });

    // Add new accounts to existing account list
    if let Some(accounts) = accounts_data.get_mut("accounts").and_then(|a| a.as_array_mut()) {
        accounts.push(program_account_json);
        accounts.push(programdata_account_json);
    }

    // 7. Read and modify tx.json file
    let mut tx_file = File::open("./tx.json")?;
    let mut tx_json = String::new();
    tx_file.read_to_string(&mut tx_json)?;

    // Parse and modify transaction JSON
    let mut tx_data: serde_json::Value = serde_json::from_str(&tx_json)?;

    // Update programId for all instructions
    if let Some(instructions) = tx_data.get_mut("instructions").and_then(|i| i.as_array_mut()) {
        for instruction in instructions {
            if let Some(program_id_field) = instruction.get_mut("programId") {
                *program_id_field = serde_json::Value::String(program_id.to_string());
            }
        }
    }

    // 8. Call simulation function
    let success = simulate_transaction_with_accounts(
        &serde_json::to_string(&accounts_data)?,
        &serde_json::to_string(&tx_data)?
    );

    if success {
        println!("Transaction simulation succeeded!");
    } else {
        println!("Transaction simulation failed!");
    }

    Ok(())
}