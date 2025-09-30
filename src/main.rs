// sol - usdc
// 1 sol --> 200 usdc
// token_a_in ==> token_b_out

use std::{collections::HashMap, sync::Arc};
mod raydium_clmm;
mod raydium_math;
use anyhow::Context;
use futures_util::StreamExt;
use tokio::sync::Mutex;
use yellowstone_grpc_client::{ClientTlsConfig, GeyserGrpcClient, Interceptor};
use yellowstone_grpc_proto::geyser::{
    SubscribeRequest, SubscribeRequestFilterAccounts, SubscribeRequestFilterAccountsFilter,
    SubscribeRequestFilterAccountsFilterMemcmp, subscribe_request_filter_accounts_filter::Filter,
    subscribe_request_filter_accounts_filter_memcmp, subscribe_update::UpdateOneof,
};

use crate::{
    raydium_clmm::PoolState,
    raydium_math::{sqrt_price_math::get_next_sqrt_price_from_input, swap_math::compute_swap_step},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DexStruct {
    token_in: u64,  // sol
    token_out: u64, // usdc
}

pub struct CexStruct {
    best_bid: u64,
    best_ask: u64,
}
#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let dex_struct = Arc::new(Mutex::new(DexStruct {
        token_in: 0u64,
        token_out: 0u64,
    }));
    let cex_struct = Arc::new(Mutex::new(CexStruct {
        best_bid: 0u64,
        best_ask: 0u64,
    }));

    let dex_struct_grpc_clone = dex_struct.clone();
    let cex_struct_grpc_clone = cex_struct.clone();

    let dex_struct_bp_clone = dex_struct.clone();
    let cex_struct_bp_clone = cex_struct.clone();

    // streaming grpc
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
                owner: vec![],                                                             // TODO
                account: vec!["3ucNos4NbumPLZNWztqGHNFFgkHeRMBQAVemeeomsUxv".to_string()], // TODO
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
            loop {
                let message = stream.next().await.unwrap();

                match message {
                    Ok(r) => {
                        if let Some(UpdateOneof::Account(r)) = r.update_oneof {
                            if let Some(account) = r.account {
                                let pool: PoolState =
                                    bincode::deserialize(&account.data[8..]).unwrap();
                                let is_base_input = true;
                                let zero_for_one = true;
                                let sqrt_price_current = pool.sqrt_price_x64;
                                let liquidity = pool.liquidity;
                                let amount_remaining: u64 = 1 * 1_000_000_000;

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

                                let mut dex_struct = dex_struct_grpc_clone.lock().await;
                                let swap = swap.unwrap();
                                dex_struct.token_in = swap.amount_in;
                                dex_struct.token_out = swap.amount_out;
                                print!("swap state {:?}", dex_struct);
                            }
                        }
                    }
                    Err(e) => {
                        eprint!("Error: unable to parse message")
                    }
                }
            }
            // Handle the client here
        }
    });

    // streaming backpack api
    let j2 = tokio::spawn(async move {});

    // math logic
    let j3 = tokio::spawn(async move {});

    j1.await;
    Ok(())
}

pub fn handle_arb_txs(
    dex_struct: Arc<Mutex<DexStruct>>,
    cex_struct: Arc<Mutex<CexStruct>>,
) -> Result<(), anyhow::Error> {
    // tokio::task(async move {});

    Ok(())
}
