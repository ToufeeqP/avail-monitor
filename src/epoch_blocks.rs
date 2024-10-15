#![allow(dead_code)]

use crate::utils::{api, Opts};
use anyhow::Result;
use structopt::StructOpt;
use subxt::backend::legacy::LegacyRpcMethods;
use subxt::{backend::rpc::RpcClient, client::OnlineClient, config::PolkadotConfig};

/// Determines number of blocks produced in an epoch for last `n` epochs 
pub async fn fetch_blocks_in_epochs(n: u32) -> Result<()> {
    let args = Opts::from_args();
    // First, create a raw RPC client:
    let rpc_client = RpcClient::from_url(args.ws.clone()).await?;

    let rpc = LegacyRpcMethods::<PolkadotConfig>::new(rpc_client.clone());
    // We can use the same client to drive our full Subxt interface too:
    let client = OnlineClient::<PolkadotConfig>::from_rpc_client(rpc_client.clone()).await?;

    // Fetch current epoch start data from the babe pallet
    let mut current_epoch_start = client
        .storage()
        .at_latest()
        .await?
        .fetch(&api::storage().babe().epoch_start())
        .await?
        .ok_or_else(|| anyhow::anyhow!("Failed to fetch current epoch start"))?;

    println!(
        "{:<10} | {:<10} | {:<10} | {:<10}",
        "epoch id #", "start #", "end #", "total"
    );
    // Fetch previous epoch start blocks for last n epochs
    for _ in 0..n {
        let block_hash = rpc
            .chain_get_block_hash(Some(current_epoch_start.0.into()))
            .await?
            .ok_or_else(|| anyhow::anyhow!("failed to fetch block hash"))?;

        let epoch = client
            .storage()
            .at(block_hash)
            .fetch(&api::storage().babe().epoch_index())
            .await?
            .ok_or_else(|| anyhow::anyhow!("Failed to fetch epoch"))?;

        // Fetch the block where the previous epoch started
        let prev_epoch_start = client
            .storage()
            .at(block_hash)
            .fetch(&api::storage().babe().epoch_start())
            .await?
            .ok_or_else(|| anyhow::anyhow!("Failed to fetch previous epoch start"))?;

        println!(
            "{:<10} | {:<10} | {:<10} | {:<10}",
            epoch,
            prev_epoch_start.1,
            current_epoch_start.1,
            current_epoch_start.1 - prev_epoch_start.1
        );
        current_epoch_start = prev_epoch_start;
    }

    Ok(())
}
