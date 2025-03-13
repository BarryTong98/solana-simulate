mod simulator;

use std::fs::File;
use std::io::Read;
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
        path::PathBuf,
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