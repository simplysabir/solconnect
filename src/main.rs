use std::collections::{HashMap, VecDeque, HashSet};
use structopt::StructOpt;
use reqwest;
use serde_json::Value;
use solana_sdk::pubkey::Pubkey;

#[derive(StructOpt)]
struct Cli {
    address1: String,
    address2: String,
}

// Add this function at the beginning of your file
fn get_rpc_endpoint() -> String {
    env::var("SOLANA_RPC_ENDPOINT").unwrap_or_else(|_| {
        eprintln!("SOLANA_RPC_ENDPOINT environment variable not set. Using default endpoint.");
        "https://api.mainnet-beta.solana.com".to_string()
    })
}

async fn get_transaction_history(address: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let solana_api_endpoint = get_rpc_endpoint();
    let mut signatures = Vec::new();
    let mut before: Option<String> = None;
    let limit = 1000;
    let max_iterations = 10; // Fetch up to 10,000 transactions
    let mut iteration = 0;

    loop {
        let mut params = serde_json::json!([address, { "limit": limit }]);
        if let Some(ref before_signature) = before {
            params[1]["before"] = serde_json::Value::String(before_signature.clone());
        }

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getConfirmedSignaturesForAddress2",
            "params": params
        });

        let client = reqwest::Client::new();
        let response = client.post(solana_api_endpoint)
            .json(&body)
            .send()
            .await?
            .json::<Value>()
            .await?;

        if let Some(result) = response.get("result").and_then(|r| r.as_array()) {
            if result.is_empty() {
                break;
            }

            for tx in result {
                if let Some(sig) = tx.get("signature").and_then(|s| s.as_str()) {
                    signatures.push(sig.to_string());
                }
            }

            before = result.last().and_then(|tx| tx.get("signature").and_then(|sig| sig.as_str()).map(String::from));
        } else {
            break;
        }

        iteration += 1;
        if iteration >= max_iterations {
            break;
        }
    }

    println!("Fetched {} transactions for address {}", signatures.len(), address);
    Ok(signatures)
}

async fn get_transaction_details(signature: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let solana_api_endpoint = get_rpc_endpoint();
    
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getConfirmedTransaction",
        "params": [
            signature,
            "json"
        ]
    });

    let client = reqwest::Client::new();
    let response = client.post(solana_api_endpoint)
        .json(&body)
        .send()
        .await?
        .json::<Value>()
        .await?;

    if let Some(result) = response.get("result") {
        Ok(result.clone())
    } else {
        Err("Failed to fetch transaction details".into())
    }
}

fn build_transaction_graph(transactions: &[Value]) -> HashMap<String, HashSet<String>> {
    let mut graph = HashMap::new();

    for transaction in transactions {
        if let Some(transaction_info) = transaction.get("transaction") {
            if let Some(message) = transaction_info.get("message") {
                if let Some(account_keys) = message.get("accountKeys").and_then(|ak| ak.as_array()) {
                    let accounts: Vec<String> = account_keys.iter()
                        .filter_map(|key| key.as_str().map(|s| s.to_string()))
                        .collect();

                    if let Some(sender) = accounts.first() {
                        for receiver in accounts.iter().skip(1) {
                            graph.entry(sender.clone()).or_insert_with(HashSet::new).insert(receiver.clone());
                            graph.entry(receiver.clone()).or_insert_with(HashSet::new).insert(sender.clone());
                            
                            // Debug print
                            // println!("Connection: {} <-> {}", sender, receiver);
                        }
                    }
                }
            }
        }
    }

    graph
}

fn find_paths(graph: &HashMap<String, HashSet<String>>, start: &str, end: &str, max_depth: usize) -> Vec<Vec<String>> {
    let mut queue = VecDeque::new();
    queue.push_back((start.to_string(), vec![start.to_string()]));
    let mut paths = Vec::new();
    let mut visited = HashSet::new();

    while let Some((node, path)) = queue.pop_front() {
        if path.len() > max_depth {
            continue;
        }

        if node == end {
            paths.push(path.clone());
            continue;
        }

        if let Some(next_nodes) = graph.get(&node) {
            for next_node in next_nodes {
                if !visited.contains(next_node) {
                    let mut new_path = path.clone();
                    new_path.push(next_node.to_string());
                    queue.push_back((next_node.to_string(), new_path));
                    visited.insert(next_node.clone());
                }
            }
        }
    }

    paths
}

fn is_valid_pubkey(address: &str) -> bool {
    address.parse::<Pubkey>().is_ok()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::from_args();

    println!("Analyzing connection between addresses:");
    println!("Address 1: {}", args.address1);
    println!("Address 2: {}", args.address2);

    if !is_valid_pubkey(&args.address1) || !is_valid_pubkey(&args.address2) {
        println!("Invalid address provided");
        return Ok(());
    }

    let signatures1 = get_transaction_history(&args.address1).await?;
    let signatures2 = get_transaction_history(&args.address2).await?;
    
    let mut all_signatures = signatures1;
    all_signatures.extend(signatures2);
    all_signatures.sort();
    all_signatures.dedup();

    println!("Fetching details for {} unique transactions", all_signatures.len());

    let mut all_transactions = Vec::new();
    for (i, signature) in all_signatures.iter().enumerate() {
        if i % 100 == 0 {
            println!("Processed {} transactions", i);
        }
        if let Ok(transaction) = get_transaction_details(signature).await {
            all_transactions.push(transaction);
        }
    }

    println!("Building transaction graph");
    let graph = build_transaction_graph(&all_transactions);

    // println!("Graph structure:");
    // for (key, value) in &graph {
    //     println!("{}: {:?}", key, value);
    // }
    println!("Number of nodes in graph: {}", graph.len());

    println!("Finding paths between addresses");
    let max_depth = 50; // Increased max depth
    let paths = find_paths(&graph, &args.address1, &args.address2, max_depth);

    println!("Found {} path(s) between the addresses:", paths.len());
    for (i, path) in paths.iter().enumerate() {
        println!("Path {}:", i + 1);
        for (j, address) in path.iter().enumerate() {
            if j > 0 {
                print!(" -> ");
            }
            print!("{}", address);
        }
        println!();
    }

    Ok(())
}