use base64::Engine;
use solana_simulate::{simulate_transaction, simulate_transaction_with_so};
use std::ffi::CString;
use std::fs::File;
use std::io::Read;


// TODO BARRY
// 1. 编译出来linux x86对应的cgo库，然后改写main，可以让他们根据系统正确的索引到对应的函数
// 2. 添加出来一个simulate TX只需要accounts.json & tx.json 我们一共提供两个函数给他们一个带so一个不带so
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Read program.so file
    let program_data = std::fs::read("/Users/barry/binance/solana-simulate/src/program.so")?;
    let program_so_base64 =
        Engine::encode(&base64::engine::general_purpose::STANDARD, program_data);

    // 2. Read accounts.json file
    let mut accounts_file = File::open("./accounts.json")?;
    let mut accounts_json = String::new();
    accounts_file.read_to_string(&mut accounts_json)?;

    // 3. Read tx.json file
    let mut tx_file = File::open("./tx.json")?;
    let mut tx_json = String::new();
    tx_file.read_to_string(&mut tx_json)?;

    // 4. Program ID
    let program_id = "HuTkmnrv4zPnArMqpbMbFhfwzTR7xfWQZHH1aQKzDKFZ";

    // 5. Convert to C strings
    let program_id_c = CString::new(program_id)?;
    let accounts_json_c = CString::new(accounts_json)?;
    let tx_json_c = CString::new(tx_json)?;
    let program_so_base64_c = CString::new(program_so_base64)?;

    // 6. Call simulation function
    let success = unsafe {
        simulate_transaction_with_so(
            program_id_c.as_ptr(),
            accounts_json_c.as_ptr(),
            tx_json_c.as_ptr(),
            program_so_base64_c.as_ptr(),
        )
    };

    if success {
        println!("Transaction simulation succeeded!");
    } else {
        println!("Transaction simulation failed!");
    }

    Ok(())
}
