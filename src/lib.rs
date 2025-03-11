mod simulator;

pub use simulator::{
    Simulator,
    SimulatorConfig,
    TransactionSimulationResult,
};

use {
    solana_program_runtime::__private::Hash,
    solana_sdk::{
        bs58,
        instruction::{AccountMeta, Instruction},
        message::Message,
        pubkey::Pubkey,
        signature::{Keypair, Signer},
        transaction::{MessageHash, SanitizedTransaction, Transaction},
    },
    std::{
        collections::HashSet,
        path::PathBuf,
        str::FromStr,
        ffi::{c_void, CStr, c_char},
    },
};

// 接受accounts_json参数的新函数
pub fn simulate_transaction_with_accounts(accounts_json: &str) -> bool {
    let config = SimulatorConfig {
        accounts_path: PathBuf::from(accounts_json),
    };

    let simulator = Simulator::new(config);

    let signer = Pubkey::from_str("H7GCUaJMUgdQiNYyoQTTmwG4fSYMV8W8ECmATZ2kyNTJ").unwrap();

    // Create instruction data
    let instruction1_data = match bs58::decode("3ipZX7g9NBXycb5v9QjqWwuhh8PxV9WL3HbJRPdURtmm5W1r5t7QtWMbGWB7mQgB8itRgPTMomJoFW7k4WhmYdYLDyWW5WMHN9M2TPGB2xFoTt3tkD87ECGUXNUzp7WskoNcjTtM9nVZMxZDcAGN1GAD82P9vhnSsQKiE5Kh2").into_vec() {
        Ok(data) => data,
        Err(_) => return false,
    };

    let instruction1 = Instruction::new_with_bytes(
        Pubkey::from_str("11111111111111111111111111111111").unwrap(),
        &instruction1_data,
        vec![
            AccountMeta::new(Pubkey::from_str("H7GCUaJMUgdQiNYyoQTTmwG4fSYMV8W8ECmATZ2kyNTJ").unwrap(), true),
            AccountMeta::new(Pubkey::from_str("JBxmvDYWetwnND8z1ppEuVWpXjpds77J2DgR2hD4Qmhg").unwrap(), false),
            AccountMeta::new(Pubkey::from_str("JBxmvDYWetwnND8z1ppEuVWpXjpds77J2DgR2hD4Qmhg").unwrap(), false),
        ],
    );

    let instruction2_data = match bs58::decode("2").into_vec() {
        Ok(data) => data,
        Err(_) => return false,
    };

    let instruction2 = Instruction::new_with_bytes(
        Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap(),
        &instruction2_data,
        vec![
            AccountMeta::new(Pubkey::from_str("JBxmvDYWetwnND8z1ppEuVWpXjpds77J2DgR2hD4Qmhg").unwrap(), false),
            AccountMeta::new_readonly(Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap(), false),
            AccountMeta::new(Pubkey::from_str("H7GCUaJMUgdQiNYyoQTTmwG4fSYMV8W8ECmATZ2kyNTJ").unwrap(), true),
            AccountMeta::new_readonly(Pubkey::from_str("SysvarRent111111111111111111111111111111111").unwrap(), false),
        ],
    );

    let instruction3_data = match bs58::decode("6FL8fBmJqzqeUnA28wVdrto").into_vec() {
        Ok(data) => data,
        Err(_) => return false,
    };

    let instruction3 = Instruction::new_with_bytes(
        Pubkey::from_str("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8").unwrap(),
        &instruction3_data,
        vec![
            AccountMeta::new_readonly(Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap(), false),
            AccountMeta::new(Pubkey::from_str("A3TiDsQgQFKSLXcj51Jiigm4Fd4F27GGrsXAsHaXh3E1").unwrap(), false),
            AccountMeta::new(Pubkey::from_str("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1").unwrap(), false),
            AccountMeta::new(Pubkey::from_str("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1").unwrap(), false),
            AccountMeta::new(Pubkey::from_str("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1").unwrap(), false),
            AccountMeta::new(Pubkey::from_str("EvFmWAGp82Kfenmh8xFzSBGYChtmWXmqqTK9QSWW9BqB").unwrap(), false),
            AccountMeta::new(Pubkey::from_str("A9M4vMERK54sEpGefBVnvxJhJRa9U6tUGbkYgYbjci1B").unwrap(), false),
            AccountMeta::new(Pubkey::from_str("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1").unwrap(), false),
            AccountMeta::new(Pubkey::from_str("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1").unwrap(), false),
            AccountMeta::new(Pubkey::from_str("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1").unwrap(), false),
            AccountMeta::new(Pubkey::from_str("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1").unwrap(), false),
            AccountMeta::new(Pubkey::from_str("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1").unwrap(), false),
            AccountMeta::new(Pubkey::from_str("EvFmWAGp82Kfenmh8xFzSBGYChtmWXmqqTK9QSWW9BqB").unwrap(), false),
            AccountMeta::new(Pubkey::from_str("A9M4vMERK54sEpGefBVnvxJhJRa9U6tUGbkYgYbjci1B").unwrap(), false),
            AccountMeta::new(Pubkey::from_str("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1").unwrap(), false),
            AccountMeta::new(Pubkey::from_str("JBxmvDYWetwnND8z1ppEuVWpXjpds77J2DgR2hD4Qmhg").unwrap(), false),
            AccountMeta::new(Pubkey::from_str("Cg1sa7AgfqVTQYREXGv4KwB9qBq5ymNddGTd1CdShjxZ").unwrap(), false),
            AccountMeta::new(Pubkey::from_str("H7GCUaJMUgdQiNYyoQTTmwG4fSYMV8W8ECmATZ2kyNTJ").unwrap(), true),
        ],
    );

    let instruction4_data = match bs58::decode("A").into_vec() {
        Ok(data) => data,
        Err(_) => return false,
    };

    let instruction4 = Instruction::new_with_bytes(
        Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap(),
        &instruction4_data,
        vec![
            AccountMeta::new(Pubkey::from_str("JBxmvDYWetwnND8z1ppEuVWpXjpds77J2DgR2hD4Qmhg").unwrap(), false),
            AccountMeta::new(Pubkey::from_str("H7GCUaJMUgdQiNYyoQTTmwG4fSYMV8W8ECmATZ2kyNTJ").unwrap(), true),
            AccountMeta::new(Pubkey::from_str("H7GCUaJMUgdQiNYyoQTTmwG4fSYMV8W8ECmATZ2kyNTJ").unwrap(), true),
        ],
    );

    // Create transaction
    let message = Message::new(&[instruction1, instruction2,  instruction3, instruction4], Some(&signer));
    let transaction = Transaction::new_unsigned(
        message,
    );

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

// 为CGO导出的C函数，接受C字符串作为参数
#[no_mangle]
pub extern "C" fn simulate_transaction_with_accounts_c(accounts_json_ptr: *const c_char) -> bool {
    // 安全地从C字符串转换为Rust字符串
    let accounts_json = unsafe {
        if accounts_json_ptr.is_null() {
            return false;
        }
        match CStr::from_ptr(accounts_json_ptr).to_str() {
            Ok(s) => s,
            Err(_) => return false,
        }
    };
    
    simulate_transaction_with_accounts(accounts_json)
}