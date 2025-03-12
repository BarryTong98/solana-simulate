use std::fs::File;
use std::io::Read;
use solana_simulate::simulate_transaction_with_accounts;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read accounts.json file
    let mut accounts_file = File::open("./accounts.json")?;
    let mut accounts_json = String::new();
    accounts_file.read_to_string(&mut accounts_json)?;

    // Read tx.json file
    let mut tx_file = File::open("./tx.json")?;
    let mut tx_json = String::new();
    tx_file.read_to_string(&mut tx_json)?;

    // Call the simulation function with the JSON content
    let success = simulate_transaction_with_accounts(&accounts_json, &tx_json);

    if success {
        println!("Transaction simulation succeeded!");
    } else {
        println!("Transaction simulation failed!");
    }
    Ok(())
}