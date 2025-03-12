use {
    reqwest::{Client, header::HeaderMap},
    serde::{Deserialize, Serialize},
    serde_json::{json, Value},
    std::{fs, time::Duration},
    tokio::time,
};

#[derive(Debug, Serialize, Deserialize)]
struct AccountInfo {
    data: Vec<Value>,
    executable: bool,
    lamports: u64,
    owner: String,
    #[serde(rename = "rentEpoch")]
    rent_epoch: u64,
    space: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct AccountResponse {
    accounts: Vec<AccountEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AccountEntry {
    pubkey: String,
    account: Option<AccountInfo>, // Change to Option to handle null values
}

#[derive(Debug, Serialize, Deserialize)]
struct RPCRequest {
    jsonrpc: String,
    id: i32,
    method: String,
    params: Vec<Value>,
}

// Modified to handle null results
#[derive(Debug, Serialize, Deserialize)]
struct RPCResponse {
    jsonrpc: String,
    result: RPCResult,
    id: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct RPCResult {
    context: Context,
    value: Option<AccountInfo>, // Change to Option to handle null values
}

#[derive(Debug, Serialize, Deserialize)]
struct Context {
    #[serde(rename = "apiVersion")]
    api_version: String,
    slot: i32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let accounts = vec![
        "DHicUK6e8nXp7UzsGP3zHvFNoKGCswN7GKxKQ1cBEDhX",
        "4GqyEzL5fyM3Bvyy4wuQzYcaq9XZ9qBe4ksPMr5RazG6",
        "9yj3zvLS3fDMqi1F8zhkaWfq8TZpZWHe6cz1Sgt7djXf",
        "uCk125EJf7iCjz43aSdzuyMAT5eii6c5zKVxEnGnosa",
        "F9Vt8r3FiJttff8QWaf9ffdvpKz5iQrF14uGGiRZgsCN",
        "mo7V3zB8pRqrTVnSk9xG2vNTnwPq2ZFTv1DBsg7YGCv",
        "6p6xgHyF7AeE6TZkSmFsko444wqoP15icUSqi2jfGiPN",
        "11111111111111111111111111111111",
        "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "HuTkmnrv4zPnArMqpbMbFhfwzTR7xfWQZHH1aQKzDKFZ",
        "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1",
        "So11111111111111111111111111111111111111112",
        "SysvarRent111111111111111111111111111111111",
        "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8",
        "EDDSpjZHrsFKYTMJDcBqXAjkLcu9EKdvrQR4XnqsXErH",
        "BbNg33EhQ4RcJ2KVmvt6No9uvQmtCS6NfjWKsK1GEBsC",
        "A7ZG7ByDi8DpzT9Ab7CiXhvgYTJQmaDPJkMDoPitaCQV"
    ];

    let mut response = AccountResponse {
        accounts: Vec::new(),
    };

    let client = Client::new();
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse()?);

    for acc in accounts {
        time::sleep(Duration::from_secs(1)).await;

        // First try with base64 encoding
        let rpc_req_base64 = RPCRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "getAccountInfo".to_string(),
            params: vec![
                json!(acc),
                json!({
                    "encoding": "base64"
                }),
            ],
        };

        match fetch_account_info(&client, &headers, &rpc_req_base64, acc).await {
            Ok(entry) => {
                response.accounts.push(entry);
                println!("Successfully fetched info for: {}", acc);
            }
            Err(e) => {
                println!("Error fetching account {} with base64 encoding: {}", acc, e);
                println!("Trying with jsonParsed encoding...");

                // Fallback to jsonParsed encoding
                let rpc_req_json_parsed = RPCRequest {
                    jsonrpc: "2.0".to_string(),
                    id: 1,
                    method: "getAccountInfo".to_string(),
                    params: vec![
                        json!(acc),
                        json!({
                            "encoding": "jsonParsed"
                        }),
                    ],
                };

                match fetch_account_info(&client, &headers, &rpc_req_json_parsed, acc).await {
                    Ok(entry) => {
                        response.accounts.push(entry);
                        println!("Successfully fetched info for {} with jsonParsed encoding", acc);
                    }
                    Err(e) => {
                        println!("Error fetching account {} with jsonParsed encoding: {}", acc, e);
                        // Still add the account to the response, but with null account info
                        response.accounts.push(AccountEntry {
                            pubkey: acc.to_string(),
                            account: None,
                        });
                    }
                }
            }
        }
    }

    let json_data = serde_json::to_string_pretty(&response)?;
    fs::write("accounts.json", json_data)?;

    println!("Account information has been written to accounts.json");
    Ok(())
}

async fn fetch_account_info(
    client: &Client,
    headers: &HeaderMap,
    rpc_req: &RPCRequest,
    acc: &str,
) -> Result<AccountEntry, Box<dyn std::error::Error>> {
    let res = client
        .post("https://api.mainnet-beta.solana.com")
        .headers(headers.clone())
        .json(&rpc_req)
        .send()
        .await?;

    let body = res.text().await?;

    // Optional: Print out response for debugging
    println!("Response for {}: {}", acc, body);

    let rpc_resp: RPCResponse = serde_json::from_str(&body)?;

    Ok(AccountEntry {
        pubkey: acc.to_string(),
        account: rpc_resp.result.value,
    })
}