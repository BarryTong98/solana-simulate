mod simulator;

use std::fs::File;
use std::io::Read;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use solana_program_runtime::__private::{ReadableAccount, Rent};
use solana_sdk::account::{Account, AccountSharedData, WritableAccount};
use solana_sdk::bpf_loader_upgradeable;
use solana_sdk::bpf_loader_upgradeable::{get_program_data_address, UpgradeableLoaderState};
pub use simulator::{
    Simulator,
    SimulatorConfig,
    TransactionSimulationResult,
};

use {
    solana_sdk::{
        bs58,
        instruction::{AccountMeta, Instruction},
        message::Message,
        pubkey::Pubkey,
        signature::{ Signer},
        transaction::{MessageHash, SanitizedTransaction, Transaction},
    },
    std::{
        collections::HashSet,
        str::FromStr,
        ffi::{CStr, c_char},
    },
};

// Function to parse instructions from JSON
fn parse_instructions_from_json_str(json_content: &str) -> Result<Vec<Instruction>, Box<dyn std::error::Error>> {
    let json_data: serde_json::Value = serde_json::from_str(json_content)?;

    // Parse instructions from JSON
    let instructions_json = json_data["instructions"].as_array()
        .ok_or("Instructions field is not an array")?;

    let mut instructions = Vec::new();

    for instr_json in instructions_json {
        // Get program ID
        let program_id_str = instr_json["programId"].as_str()
            .ok_or("programIdIndex is not a string")?;
        let program_id = Pubkey::from_str(program_id_str)?;

        // Get data
        let data_str = instr_json["data"].as_str()
            .ok_or("data is not a string")?;
        let data = bs58::decode(data_str).into_vec()?;

        // Get accounts
        let accounts_json = instr_json["accounts"].as_array()
            .ok_or("accounts field is not an array")?;

        let mut accounts = Vec::new();

        for acc_json in accounts_json {
            let pubkey_str = acc_json["publicKey"].as_str()
                .ok_or("PublicKey is not a string")?;
            let pubkey = Pubkey::from_str(pubkey_str)?;

            let is_writable = acc_json["isWritable"].as_bool()
                .ok_or("IsWritable is not a boolean")?;

            let is_signer = acc_json["isSigner"].as_bool()
                .ok_or("IsSigner is not a boolean")?;

            let account_meta = if is_writable {
                AccountMeta::new(pubkey, is_signer)
            } else {
                AccountMeta::new_readonly(pubkey, is_signer)
            };

            accounts.push(account_meta);
        }

        // Create instruction
        let instruction = Instruction::new_with_bytes(
            program_id,
            &data,
            accounts,
        );

        instructions.push(instruction);
    }

    Ok(instructions)
}

// Transaction simulation function
pub fn simulate_transaction_with_accounts(accounts_json: &str,tx_json: &str) -> bool {
    let config = SimulatorConfig {
        accounts_json_str: accounts_json.to_string(),
    };

    let simulator = Simulator::new(config);

    // Parse instructions from JSON string
    let instructions = match parse_instructions_from_json_str(tx_json) {
        Ok(instrs) => instrs,
        Err(e) => {
            println!("Error parsing instructions: {}", e);
            return false;
        }
    };

    // Create signer
    let signer = match Pubkey::from_str("DHicUK6e8nXp7UzsGP3zHvFNoKGCswN7GKxKQ1cBEDhX") {
        Ok(pk) => pk,
        Err(_) => return false,
    };

    // Create transaction
    let message = Message::new(&instructions, Some(&signer));
    let transaction = Transaction::new_unsigned(message);

    // Convert to sanitized transaction
    let sanitized_transaction = match SanitizedTransaction::try_create(
        transaction.into(),
        MessageHash::Compute,
        None,
        simulator.clone(),
        &HashSet::new(),
    ) {
        Ok(tx) => tx,
        Err(_) => return false,
    };

    // Execute transaction simulation
    let simulation_result = simulator.simulate_transaction_unchecked(
        &sanitized_transaction,
        true, // Enable CPI recording
    );

    println!("Simulation logs:");
    for log in &simulation_result.logs {
        println!("{}", log);
    }

    println!("Simulation result: {:?}", simulation_result.result);

    simulation_result.result.is_ok()
}

#[no_mangle]
pub extern "C" fn simulate_transaction_with_accounts_c(accounts_json_ptr: *const c_char, tx_json_ptr: *const c_char) -> bool {
    let c_str_to_rust = |ptr: *const c_char| -> Option<&'static str> {
        if ptr.is_null() {
            return None;
        }
        unsafe {
            CStr::from_ptr(ptr).to_str().ok()
        }
    };

    let accounts_json = match c_str_to_rust(accounts_json_ptr) {
        Some(s) => s,
        None => return false,
    };

    let tx_json = match c_str_to_rust(tx_json_ptr) {
        Some(s) => s,
        None => return false,
    };

    simulate_transaction_with_accounts(accounts_json, tx_json)
}
pub fn simulate_program(
    simulator: &Simulator,
    program_id: &Pubkey,
    instruction_data: &[u8],
    accounts: &[AccountMeta],
) -> TransactionSimulationResult {
    // 1. Create instruction
    let instruction = Instruction::new_with_bytes(
        *program_id,
        instruction_data,
        accounts.to_vec(),
    );

    // 2. Create transaction
    let message = Message::new(&[instruction], Some(&program_id));
    let transaction = Transaction::new_unsigned(message);

    // 3. Convert to SanitizedTransaction
    let sanitized_transaction = SanitizedTransaction::try_create(
        transaction.into(),
        MessageHash::Compute,
        None,
        simulator.clone(),
        &HashSet::new(),
    ).unwrap();

    // 4. Execute simulation
    simulator.simulate_transaction_unchecked(
        &sanitized_transaction,
        true, // Enable CPI logging
    )
}

pub fn create_simulation_environment(
    program_id: &Pubkey,
    program_account: AccountSharedData,
    programdata_account: AccountSharedData,
) -> Simulator {
    // 1. Create account mapping
    let programdata_address = get_program_data_address(program_id);
    let mut account_map = vec![
        (program_id, program_account),
        (&programdata_address, programdata_account),
    ];

    // 2. Create Simulator configuration
    let config = SimulatorConfig {
        accounts_json_str: serde_json::to_string(&account_map).unwrap(),
    };

    // 3. Create Simulator instance
    Simulator::new(config)
}

pub fn create_program_accounts(program_id: &Pubkey, program_data: &[u8]) -> (AccountSharedData, AccountSharedData) {
    // 1. Create Program Data Account
    let programdata_address = get_program_data_address(program_id);
    let programdata_account = {
        let space = UpgradeableLoaderState::size_of_programdata_metadata() + program_data.len();
        let lamports = Rent::default().minimum_balance(space);
        let mut data = bincode::serialize(&UpgradeableLoaderState::ProgramData {
            slot: 0,
            upgrade_authority_address: Some(Pubkey::default()),
        }).unwrap();
        data.extend_from_slice(program_data);

        AccountSharedData::from(Account {
            lamports,
            data,
            owner: bpf_loader_upgradeable::id(),
            executable: false,
            rent_epoch: 0,
        })
    };

    // 2. Create Program Account
    let program_account = {
        let space = UpgradeableLoaderState::size_of_program();
        let lamports = Rent::default().minimum_balance(space);
        let data = bincode::serialize(&UpgradeableLoaderState::Program {
            programdata_address,
        }).unwrap();

        let mut account = AccountSharedData::from(Account {
            lamports,
            data,
            owner: bpf_loader_upgradeable::id(),
            executable: true,
            rent_epoch: 0,
        });
        account.set_executable(true);
        account
    };

    (program_account, programdata_account)
}

#[no_mangle]
pub extern "C" fn simulate_program_with_so(
    program_id_str: *const c_char,
    accounts_json_ptr: *const c_char,
    tx_json_ptr: *const c_char,
    program_so_base64: *const c_char,
) -> bool {
    // 1. Convert C strings to Rust strings
    let c_str_to_rust = |ptr: *const c_char| -> Option<&'static str> {
        if ptr.is_null() {
            return None;
        }
        unsafe {
            CStr::from_ptr(ptr).to_str().ok()
        }
    };

    let program_id_str = match c_str_to_rust(program_id_str) {
        Some(s) => s,
        None => return false,
    };

    let accounts_json = match c_str_to_rust(accounts_json_ptr) {
        Some(s) => s,
        None => return false,
    };

    let tx_json = match c_str_to_rust(tx_json_ptr) {
        Some(s) => s,
        None => return false,
    };

    let program_so_base64 = match c_str_to_rust(program_so_base64) {
        Some(s) => s,
        None => return false,
    };

    // 2. Decode program_id
    let program_id = match Pubkey::from_str(program_id_str) {
        Ok(pk) => pk,
        Err(_) => return false,
    };

    // 3. Decode program.so file content
    let program_data = match BASE64_STANDARD.decode(program_so_base64) {
        Ok(data) => data,
        Err(_) => return false,
    };

    // 4. Create program accounts
    let (program_account, programdata_account) = create_program_accounts(&program_id, &program_data);

    // 5. Parse JSON account data
    let mut accounts_data: serde_json::Value = match serde_json::from_str(accounts_json) {
        Ok(data) => data,
        Err(_) => return false,
    };

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

    // 7. Parse and modify transaction JSON
    let mut tx_data: serde_json::Value = match serde_json::from_str(tx_json) {
        Ok(data) => data,
        Err(_) => return false,
    };

    // Update all instruction programIds
    if let Some(instructions) = tx_data.get_mut("instructions").and_then(|i| i.as_array_mut()) {
        for instruction in instructions {
            if let Some(program_id_field) = instruction.get_mut("programId") {
                *program_id_field = serde_json::Value::String(program_id.to_string());
            }
        }
    }

    // 8. Call simulation function
    simulate_transaction_with_accounts(
        &serde_json::to_string(&accounts_data).unwrap_or_default(),
        &serde_json::to_string(&tx_data).unwrap_or_default(),
    )
}