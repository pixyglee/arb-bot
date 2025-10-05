use std::{collections::HashMap, sync::Arc};
mod raydium_clmm;
mod raydium_math;

use futures_util::StreamExt;
use reqwest;
use serde::Deserialize;
use tokio::sync::Mutex;
use yellowstone_grpc_client::{ClientTlsConfig, GeyserGrpcClient};
use yellowstone_grpc_proto::geyser::{
    SubscribeRequest, SubscribeRequestFilterAccounts, subscribe_update::UpdateOneof,
};

use crate::{
    raydium_clmm::PoolState,
    raydium_math::{sqrt_price_math::get_next_sqrt_price_from_input, swap_math::compute_swap_step},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DexStruct {
    token_in: u64,          // sol
    token_out: Option<u64>, // usdc, None until first update
}

#[derive(Debug)]
pub struct CexStruct {
    best_bid: u64,
    best_ask: u64,
}

#[derive(Debug, Deserialize)]
struct DepthResponse {
    bids: Vec<(String, String)>,
    asks: Vec<(String, String)>,
}
// ------------------- CEX fetch -------------------
async fn fetch_cex_data(cex_struct: Arc<Mutex<CexStruct>>) -> Result<(), anyhow::Error> {
    let url = "https://api.backpack.exchange/api/v1/depth?symbol=SOL_USDC";
    loop {
        let resp: DepthResponse = reqwest::get(url).await?.json().await?;

        let best_bid = resp.bids.first().unwrap().0.parse::<f64>().unwrap();
        let best_ask = resp.asks.first().unwrap().0.parse::<f64>().unwrap();

        let mut cex = cex_struct.lock().await;
        cex.best_bid = (best_bid * 1_000_000.0) as u64;
        cex.best_ask = (best_ask * 1_000_000.0) as u64;

        println!("CEX updated: {:?}", *cex);
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
// ------------------- Arb logic -------------------
async fn handle_arb_txs(dex_struct: Arc<Mutex<DexStruct>>, cex_struct: Arc<Mutex<CexStruct>>) {
    loop {
        let dex = dex_struct.lock().await;

        // Wait until DEX has a valid token_out value
        let token_out = match dex.token_out {
            Some(val) => val,
            None => {
                drop(dex);
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                continue;
            }
        };

        // Lock CEX to read best_bid / best_ask
        let cex = cex_struct.lock().await;

        let spread = token_out as i64 - cex.best_ask as i64;

        // Arb check
        if spread > 0 {
            println!(
                "💸 Arb found: Buy on CEX at {} USDC, Sell on DEX for {} USDC (spread: {})",
                cex.best_ask, token_out, spread
            );
        }

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
}
// ------------------- Main -------------------
#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let dex_struct = Arc::new(Mutex::new(DexStruct {
        token_in: 0,
        token_out: None,
    }));
    let cex_struct = Arc::new(Mutex::new(CexStruct {
        best_bid: 0,
        best_ask: 0,
    }));

    let dex_clone = dex_struct.clone();
    let cex_clone = cex_struct.clone();
    let dex_grpc_clone = dex_struct.clone();
    let cex_grpc_clone = cex_struct.clone();
    // -------- DEX streaming task --------
    let j1 = tokio::spawn(async move {
        let tls_config = ClientTlsConfig::new().with_native_roots();
        if let Ok(mut client) = GeyserGrpcClient::build_from_shared(
            "https://solana-yellowstone-grpc.publicnode.com:443",
        )
        .unwrap()
        .keep_alive_while_idle(true)
        .tls_config(tls_config)
        .unwrap()
        .connect()
        .await
        {
            let mut accounts: HashMap<String, SubscribeRequestFilterAccounts> = HashMap::new();
            let filter = SubscribeRequestFilterAccounts {
                owner: vec![],
                account: vec!["3ucNos4NbumPLZNWztqGHNFFgkHeRMBQAVemeeomsUxv".to_string()],
                ..Default::default()
            };
            accounts.insert("client".to_string(), filter);
            let (_tx, mut stream) = client
                .subscribe_with_request(Some(SubscribeRequest {
                    accounts,
                    ..Default::default()
                }))
                .await
                .expect("Error: unable to make grpc connection request");

            while let Some(message) = stream.next().await {
                match message {
                    Ok(r) => {
                        if let Some(UpdateOneof::Account(r)) = r.update_oneof {
                            if let Some(account) = r.account {
                                let pool: PoolState =
                                    bincode::deserialize(&account.data[8..]).unwrap();
                                let sqrt_price_current = pool.sqrt_price_x64;
                                let liquidity = pool.liquidity;
                                let amount_remaining: u64 = 1_000_000_000;
                                let zero_for_one = true;
                                let is_base_input = true;

                                let sqrt_price_target = get_next_sqrt_price_from_input(
                                    sqrt_price_current,
                                    liquidity,
                                    amount_remaining,
                                    zero_for_one,
                                );

                                let swap = compute_swap_step(
                                    sqrt_price_current,
                                    sqrt_price_target,
                                    liquidity,
                                    amount_remaining,
                                    0,
                                    is_base_input,
                                    zero_for_one,
                                );

                                let mut dex = dex_grpc_clone.lock().await;
                                let swap = swap.unwrap();
                                dex.token_in = swap.amount_in;
                                dex.token_out = Some(swap.amount_out);
                                println!("DEX swap state: {:?}", dex);
                            }
                        }
                    }
                    Err(_) => eprintln!("Error parsing DEX message"),
                }
            }
        }
    });
    // -------- CEX fetch task --------
    let j2 = tokio::spawn(fetch_cex_data(cex_clone));
    // -------- Arb logic task --------
    let j3 = tokio::spawn(handle_arb_txs(dex_clone, cex_grpc_clone));
    // Wait for all
    j1.await?;
    j2.await?;
    j3.await?;

    Ok(())
}
