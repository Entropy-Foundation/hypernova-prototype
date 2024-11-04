use alloy::{
    primitives::{address, Address},
    providers::{Provider, ProviderBuilder, WsConnect},
    rpc::types::{BlockNumberOrTag, Filter, ValueOrArray, Log},
};
use chrono::Utc;
use eyre::Result;
use futures_util::stream::StreamExt;
use hex_literal::hex;

#[tokio::main]
async fn main() -> Result<()> {
    let rpc_url_chain1 = "wss://sepolia.infura.io/ws/v3/8dd4a35393c04ed8a4c1c563c8ea099e"; // Replace with your actual URL
    let rpc_url_chain2 = "wss://polygon-amoy.infura.io/ws/v3/8dd4a35393c04ed8a4c1c563c8ea099e"; // Replace with your actual URL

    let ws_chain1 = WsConnect::new(rpc_url_chain1);
    let ws_chain2 = WsConnect::new(rpc_url_chain2);
    
    let provider_chain1 = ProviderBuilder::new().on_ws(ws_chain1).await?;
    let provider_chain2 = ProviderBuilder::new().on_ws(ws_chain2).await?;

    let contract_address: Address = Address::from(hex!("8c6faf12a32462f5f2f5282821fe9e789e6d82e7"));

    let filter_chain1 = Filter::new()
        .address(ValueOrArray::Value(contract_address))
        .event("Generate(uint256,uint256)")
        .from_block(BlockNumberOrTag::Latest);

    let filter_chain2 = Filter::new()
        .address(ValueOrArray::Value(contract_address))
        .event("Generate(uint256,uint256)")
        .from_block(BlockNumberOrTag::Latest);

    let sub_chain1 = provider_chain1.subscribe_logs(&filter_chain1).await?;
    let sub_chain2 = provider_chain2.subscribe_logs(&filter_chain2).await?;

    let mut stream_chain1 = sub_chain1.into_stream();
    let mut stream_chain2 = sub_chain2.into_stream();

    let mut latest_log_chain1: Option<Log> = None;
    let mut latest_log_chain2: Option<Log> = None;

    loop {
        tokio::select! {
            Some(log_chain1) = stream_chain1.next() => {
                latest_log_chain1 = Some(log_chain1.clone());

                if let Some(latest_log_chain2) = &latest_log_chain2 {
                    let current_time = Utc::now();
                        println!("Chain 1 - Before Compare: {}", current_time);
                    if compare_logs(&log_chain1, latest_log_chain2) {
                        let current_time = Utc::now();
                        println!("Chain 1 - Current Time: {}", current_time);
                        println!("Match found between Chain 1 and Chain 2 logs!");
                    }
                }
            },
            Some(log_chain2) = stream_chain2.next() => {
                latest_log_chain2 = Some(log_chain2.clone());

                if let Some(latest_log_chain1) = &latest_log_chain1 {
                    let current_time = Utc::now();
                        println!("Chain 2 - Before Compare: {}", current_time);
                    if compare_logs(latest_log_chain1, &log_chain2) {
                        let current_time = Utc::now();
                        println!("Chain 2 - Current Time: {}", current_time);
                        println!("Match found between Chain 1 and Chain 2 logs!");
                    }
                }
            },
            else => break,
        }
    }

    Ok(())
}

fn compare_logs(log1: &Log, log2: &Log) -> bool {
    let request_id_1 = log1.topics()[1]; 
    let request_id_2 = log2.topics()[1]; 
    request_id_1 == request_id_2
}
