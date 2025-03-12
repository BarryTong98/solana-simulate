use {
    solana_sdk::{
        bs58,
        instruction::{AccountMeta, Instruction},
        message::Message,
        pubkey::Pubkey,
        signature::Signer,
        transaction::{MessageHash, SanitizedTransaction, Transaction},
    },
    solana_simulate::{Simulator, SimulatorConfig},
    std::{
        collections::HashSet,
        path::PathBuf,
        str::FromStr,
    },
};

use std::fs::File;
use std::io::Read;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = SimulatorConfig {
        accounts_path: PathBuf::from("./accounts-pre.json"),
    };

    let simulator = Simulator::new(config);

    // Read and parse the JSON file
    let mut file = File::open("./tx.json")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let json_data: serde_json::Value = serde_json::from_str(&contents)?;

    // Parse instructions from JSON
    let instructions_json = json_data["instructions"].as_array()
        .ok_or("Instructions field is not an array")?;

    let mut instructions = Vec::new();

    for instr_json in instructions_json {
        // Get program ID
        let program_id_str = instr_json["programIdIndex"].as_str()
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
            let pubkey_str = acc_json["PublicKey"].as_str()
                .ok_or("PublicKey is not a string")?;
            let pubkey = Pubkey::from_str(pubkey_str)?;

            let is_writable = acc_json["IsWritable"].as_bool()
                .ok_or("IsWritable is not a boolean")?;

            let is_signer = acc_json["IsSigner"].as_bool()
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

    // Create signer
    let signer = Pubkey::from_str("H7GCUaJMUgdQiNYyoQTTmwG4fSYMV8W8ECmATZ2kyNTJ")?;

    // Create transaction
    let message = Message::new(&instructions, Some(&signer));
    let transaction = Transaction::new_unsigned(message);

    // Convert to sanitized transaction
    let sanitized_transaction = SanitizedTransaction::try_create(
        transaction.into(),
        MessageHash::Compute,
        None,
        simulator.clone(),
        &HashSet::new(),
    )?;

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

    Ok(())
}

// fn main() {
//     let config = SimulatorConfig {
//         accounts_path: PathBuf::from("./accounts-pre.json"),
//     };
//
//     let simulator = Simulator::new(config);
//
//     let signer = Pubkey::from_str("H7GCUaJMUgdQiNYyoQTTmwG4fSYMV8W8ECmATZ2kyNTJ").unwrap();
//
//     // Create instruction data
//     let instruction1_data = bs58::decode("3ipZX7g9NBXycb5v9QjqWwuhh8PxV9WL3HbJRPdURtmm5W1r5t7QtWMbGWB7mQgB8itRgPTMomJoFW7k4WhmYdYLDyWW5WMHN9M2TPGB2xFoTt3tkD87ECGUXNUzp7WskoNcjTtM9nVZMxZDcAGN1GAD82P9vhnSsQKiE5Kh2").into_vec().unwrap();
//     let instruction1 = Instruction::new_with_bytes(
//         Pubkey::from_str("11111111111111111111111111111111").unwrap(),
//         &instruction1_data,
//         vec![
//             AccountMeta::new(Pubkey::from_str("H7GCUaJMUgdQiNYyoQTTmwG4fSYMV8W8ECmATZ2kyNTJ").unwrap(), true),
//             AccountMeta::new(Pubkey::from_str("JBxmvDYWetwnND8z1ppEuVWpXjpds77J2DgR2hD4Qmhg").unwrap(), false),
//             AccountMeta::new(Pubkey::from_str("JBxmvDYWetwnND8z1ppEuVWpXjpds77J2DgR2hD4Qmhg").unwrap(), false),
//         ],
//     );
//
//     let instruction2_data = bs58::decode("2").into_vec().unwrap();
//     let instruction2 = Instruction::new_with_bytes(
//         Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap(),
//         &instruction2_data,
//         vec![
//             AccountMeta::new(Pubkey::from_str("JBxmvDYWetwnND8z1ppEuVWpXjpds77J2DgR2hD4Qmhg").unwrap(), false),
//             AccountMeta::new_readonly(Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap(), false),
//             AccountMeta::new(Pubkey::from_str("H7GCUaJMUgdQiNYyoQTTmwG4fSYMV8W8ECmATZ2kyNTJ").unwrap(), true),
//             AccountMeta::new_readonly(Pubkey::from_str("SysvarRent111111111111111111111111111111111").unwrap(), false),
//         ],
//     );
//
//     let instruction3_data = bs58::decode("6FL8fBmJqzqeUnA28wVdrto").into_vec().unwrap();
//     let instruction3 = Instruction::new_with_bytes(
//         Pubkey::from_str("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8").unwrap(),
//         &instruction3_data,
//         vec![
//             AccountMeta::new_readonly(Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap(), false),
//             AccountMeta::new(Pubkey::from_str("A3TiDsQgQFKSLXcj51Jiigm4Fd4F27GGrsXAsHaXh3E1").unwrap(), false),
//             AccountMeta::new(Pubkey::from_str("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1").unwrap(), false),
//             AccountMeta::new(Pubkey::from_str("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1").unwrap(), false),
//             AccountMeta::new(Pubkey::from_str("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1").unwrap(), false),
//             AccountMeta::new(Pubkey::from_str("EvFmWAGp82Kfenmh8xFzSBGYChtmWXmqqTK9QSWW9BqB").unwrap(), false),
//             AccountMeta::new(Pubkey::from_str("A9M4vMERK54sEpGefBVnvxJhJRa9U6tUGbkYgYbjci1B").unwrap(), false),
//             AccountMeta::new(Pubkey::from_str("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1").unwrap(), false),
//             AccountMeta::new(Pubkey::from_str("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1").unwrap(), false),
//             AccountMeta::new(Pubkey::from_str("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1").unwrap(), false),
//             AccountMeta::new(Pubkey::from_str("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1").unwrap(), false),
//             AccountMeta::new(Pubkey::from_str("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1").unwrap(), false),
//             AccountMeta::new(Pubkey::from_str("EvFmWAGp82Kfenmh8xFzSBGYChtmWXmqqTK9QSWW9BqB").unwrap(), false),
//             AccountMeta::new(Pubkey::from_str("A9M4vMERK54sEpGefBVnvxJhJRa9U6tUGbkYgYbjci1B").unwrap(), false),
//             AccountMeta::new(Pubkey::from_str("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1").unwrap(), false),
//             AccountMeta::new(Pubkey::from_str("JBxmvDYWetwnND8z1ppEuVWpXjpds77J2DgR2hD4Qmhg").unwrap(), false),
//             AccountMeta::new(Pubkey::from_str("Cg1sa7AgfqVTQYREXGv4KwB9qBq5ymNddGTd1CdShjxZ").unwrap(), false),
//             AccountMeta::new(Pubkey::from_str("H7GCUaJMUgdQiNYyoQTTmwG4fSYMV8W8ECmATZ2kyNTJ").unwrap(), true),
//         ],
//     );
//
//     let instruction4_data = bs58::decode("A").into_vec().unwrap();
//     let instruction4 = Instruction::new_with_bytes(
//         Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap(),
//         &instruction4_data,
//         vec![
//             AccountMeta::new(Pubkey::from_str("JBxmvDYWetwnND8z1ppEuVWpXjpds77J2DgR2hD4Qmhg").unwrap(), false),
//             AccountMeta::new(Pubkey::from_str("H7GCUaJMUgdQiNYyoQTTmwG4fSYMV8W8ECmATZ2kyNTJ").unwrap(), true),
//             AccountMeta::new(Pubkey::from_str("H7GCUaJMUgdQiNYyoQTTmwG4fSYMV8W8ECmATZ2kyNTJ").unwrap(), true),
//         ],
//     );
//
//     // Create transaction
//     let message = Message::new(&[instruction1, instruction2,
//         instruction3, instruction4], Some(&signer));
//     let transaction = Transaction::new_unsigned(
//         message,
//     );
//
//     // Convert to sanitized transaction
//     let sanitized_transaction = SanitizedTransaction::try_create(
//         transaction.into(),
//         MessageHash::Compute,
//         None,
//         simulator.clone(),
//         &HashSet::new(),
//     ).unwrap();
//
//     // Execute transaction simulation
//     let simulation_result = simulator.simulate_transaction_unchecked(
//         &sanitized_transaction,
//         true, // Enable CPI recording
//     );
//
//     println!("Simulation logs:");
//     for log in  &simulation_result.logs {
//         println!("{}", log);
//     }
//
//     println!("Simulation result: {:?}", simulation_result.result);
// }


// fn main() {
//     let config = SimulatorConfig {
//         accounts_path: PathBuf::from("./accounts.json"),
//     };
//
//     let simulator = Simulator::new(config);
//
//     let signer = Pubkey::from_str("DHicUK6e8nXp7UzsGP3zHvFNoKGCswN7GKxKQ1cBEDhX").unwrap();
//
//     // Create instruction data
//     let instruction1_data = bs58::decode("3ipZWyostf7mVDZFmAiZQwUemkdqvTxhbpHcKP12Bczi8KmP72yapwSaaab1xyirgt3s5ptUbiqHVYN95ZSjVZ5o1sFKPReBLy5XSkowTc5XFDsgAJdgxZTTPEbc1VfHcVKgENnJyyALdzqZSJ1jtyc1QXxwMCnfd1gxpw2Jk").into_vec().unwrap();
//     let instruction1 = Instruction::new_with_bytes(
//         Pubkey::from_str("11111111111111111111111111111111").unwrap(),
//         &instruction1_data,
//         vec![
//             AccountMeta::new(Pubkey::from_str("DHicUK6e8nXp7UzsGP3zHvFNoKGCswN7GKxKQ1cBEDhX").unwrap(), true),
//             AccountMeta::new(Pubkey::from_str("FmzPaiNTu9CPzXAH1rJfMKpu1URLFK5tfrHE9CH8MyUR").unwrap(), false),
//             AccountMeta::new_readonly(Pubkey::from_str("DHicUK6e8nXp7UzsGP3zHvFNoKGCswN7GKxKQ1cBEDhX").unwrap(), true),
//         ],
//     );
//
//     let instruction2_data = bs58::decode("2").into_vec().unwrap();
//     let instruction2 = Instruction::new_with_bytes(
//         Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap(),
//         &instruction2_data,
//         vec![
//             AccountMeta::new(Pubkey::from_str("Arzw7YwC8h65YTJaFLBygm6MaJhm1rHkZTTBv9i4t37d").unwrap(), false),
//             AccountMeta::new_readonly(Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap(), false),
//             AccountMeta::new_readonly(Pubkey::from_str("DHicUK6e8nXp7UzsGP3zHvFNoKGCswN7GKxKQ1cBEDhX").unwrap(), false),
//             AccountMeta::new_readonly(Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap(), false),
//         ],
//     );
//
//     let instruction3_data = bs58::decode("F2fEQRSZSQd3BWwrUk3J5oAJiW4kuwKWN2potvyGsXMCrVMUQ4YS5mRb2BFquGpsRjuarVVa7N14Vxv5gtTHHmy6Y").into_vec().unwrap();
//     let instruction3 = Instruction::new_with_bytes(
//         Pubkey::from_str("HuTkmnrv4zPnArMqpbMbFhfwzTR7xfWQZHH1aQKzDKFZ").unwrap(),
//         &instruction3_data,
//         vec![
//             AccountMeta::new_readonly(Pubkey::from_str("DHicUK6e8nXp7UzsGP3zHvFNoKGCswN7GKxKQ1cBEDhX").unwrap(), true),
//             AccountMeta::new(Pubkey::from_str("FmzPaiNTu9CPzXAH1rJfMKpu1URLFK5tfrHE9CH8MyUR").unwrap(), false),
//             AccountMeta::new(Pubkey::from_str("4GqyEzL5fyM3Bvyy4wuQzYcaq9XZ9qBe4ksPMr5RazG6").unwrap(), false),
//             AccountMeta::new_readonly(Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap(), false),
//             AccountMeta::new_readonly(Pubkey::from_str("6p6xgHyF7AeE6TZkSmFsko444wqoP15icUSqi2jfGiPN").unwrap(), false),
//             AccountMeta::new(Pubkey::from_str("9yj3zvLS3fDMqi1F8zhkaWfq8TZpZWHe6cz1Sgt7djXf").unwrap(), false),
//             AccountMeta::new_readonly(Pubkey::from_str("11111111111111111111111111111111").unwrap(), false),
//             AccountMeta::new_readonly(Pubkey::from_str("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8").unwrap(), false),
//             AccountMeta::new(Pubkey::from_str("DHicUK6e8nXp7UzsGP3zHvFNoKGCswN7GKxKQ1cBEDhX").unwrap(), false),
//             AccountMeta::new(Pubkey::from_str("FmzPaiNTu9CPzXAH1rJfMKpu1URLFK5tfrHE9CH8MyUR").unwrap(), false),
//             AccountMeta::new(Pubkey::from_str("4GqyEzL5fyM3Bvyy4wuQzYcaq9XZ9qBe4ksPMr5RazG6").unwrap(), false),
//             AccountMeta::new_readonly(Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap(), false),
//             AccountMeta::new(Pubkey::from_str("uCk125EJf7iCjz43aSdzuyMAT5eii6c5zKVxEnGnosa").unwrap(), false),
//             AccountMeta::new_readonly(Pubkey::from_str("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1").unwrap(), false),
//             AccountMeta::new(Pubkey::from_str("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1").unwrap(), false),
//             AccountMeta::new(Pubkey::from_str("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1").unwrap(), false),
//             AccountMeta::new(Pubkey::from_str("F9Vt8r3FiJttff8QWaf9ffdvpKz5iQrF14uGGiRZgsCN").unwrap(), false),
//             AccountMeta::new(Pubkey::from_str("mo7V3zB8pRqrTVnSk9xG2vNTnwPq2ZFTv1DBsg7YGCv").unwrap(), false),
//             AccountMeta::new_readonly(Pubkey::from_str("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1").unwrap(), false),
//             AccountMeta::new(Pubkey::from_str("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1").unwrap(), false),
//             AccountMeta::new(Pubkey::from_str("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1").unwrap(), false),
//             AccountMeta::new(Pubkey::from_str("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1").unwrap(), false),
//             AccountMeta::new(Pubkey::from_str("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1").unwrap(), false),
//             AccountMeta::new(Pubkey::from_str("F9Vt8r3FiJttff8QWaf9ffdvpKz5iQrF14uGGiRZgsCN").unwrap(), false),
//             AccountMeta::new(Pubkey::from_str("mo7V3zB8pRqrTVnSk9xG2vNTnwPq2ZFTv1DBsg7YGCv").unwrap(), false),
//             AccountMeta::new_readonly(Pubkey::from_str("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1").unwrap(), false),
//         ],
//     );
//
//     let instruction4_data = bs58::decode("A").into_vec().unwrap();
//     let instruction4 = Instruction::new_with_bytes(
//         Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap(),
//         &instruction4_data,
//         vec![
//             AccountMeta::new(Pubkey::from_str("CeXVaaAatjw7mtAvF1vJEbLPfNsXvwAQicjJTyNVfLF9").unwrap(), false),
//             AccountMeta::new(Pubkey::from_str("DHicUK6e8nXp7UzsGP3zHvFNoKGCswN7GKxKQ1cBEDhX").unwrap(), false),
//             AccountMeta::new_readonly(Pubkey::from_str("DHicUK6e8nXp7UzsGP3zHvFNoKGCswN7GKxKQ1cBEDhX").unwrap(), true),
//         ],
//     );
//
//     // Create transaction
//     let message = Message::new(&[instruction1, instruction2,
//         instruction3, instruction4], Some(&signer));
//     let transaction = Transaction::new_unsigned(
//         message,
//     );
//
//     // Convert to sanitized transaction
//     let sanitized_transaction = SanitizedTransaction::try_create(
//         transaction.into(),
//         MessageHash::Compute,
//         None,
//         simulator.clone(),
//         &HashSet::new(),
//     ).unwrap();
//
//     // Execute transaction simulation
//     let simulation_result = simulator.simulate_transaction_unchecked(
//         &sanitized_transaction,
//         true, // Enable CPI recording
//     );
//
//     println!("Simulation logs:");
//     for log in  &simulation_result.logs {
//         println!("{}", log);
//     }
//
//     println!("Simulation result: {:?}", simulation_result.result);
// }