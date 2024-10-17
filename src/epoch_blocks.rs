#![allow(dead_code)]

use crate::utils::{
    api,
    api::{session::events::NewSession, staking::events::EraPaid},
    Opts,
};
use anyhow::Result;
use structopt::StructOpt;
use subxt::{
    backend::{legacy::LegacyRpcMethods, rpc::RpcClient},
    client::OnlineClient,
    config::PolkadotConfig,
};

/// Determines number of blocks produced in an epoch for last `n` epochs
pub async fn fetch_blocks_in_epochs(n: u32) -> Result<()> {
    let args = Opts::from_args();

    let rpc_client = RpcClient::from_url(args.ws.clone()).await?;

    let epoch_data = blocks_in_epoch(rpc_client, n).await?;
    for (epoch, blocks) in epoch_data {
        println!("Epoch {} produced {} blocks", epoch, blocks);
    }

    Ok(())
}

async fn blocks_in_epoch(rpc_client: RpcClient, n: u32) -> Result<Vec<(u64, u32)>> {
    let rpc = LegacyRpcMethods::<PolkadotConfig>::new(rpc_client.clone());
    // We can use the same client to drive our full Subxt interface too:
    let client = OnlineClient::<PolkadotConfig>::from_rpc_client(rpc_client).await?;

    // Fetch current epoch start data from the babe pallet
    let mut current_epoch_start = client
        .storage()
        .at_latest()
        .await?
        .fetch(&api::storage().babe().epoch_start())
        .await?
        .ok_or_else(|| anyhow::anyhow!("Failed to fetch current epoch start"))?;

    let mut epoch_data: Vec<(u64, u32)> = Vec::new();

    // Fetch previous epoch start blocks for last n epochs
    for _ in 0..n {
        // Get block hash for the current epoch start
        let block_hash = rpc
            .chain_get_block_hash(Some(current_epoch_start.0.into()))
            .await?
            .ok_or_else(|| anyhow::anyhow!("failed to fetch block hash"))?;

        // Get the epoch index
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

        // Calculate the number of blocks produced in the epoch
        let blocks_in_epoch = current_epoch_start.1 - prev_epoch_start.1;
        epoch_data.push((epoch, blocks_in_epoch));

        current_epoch_start = prev_epoch_start;
    }

    Ok(epoch_data)
}

/// Monitors the chain and prints block counts when an epoch/era ends
// TODO: handle epoch 0
pub async fn monitor_chain() -> Result<()> {
    let args = Opts::from_args();
    let rpc_client = RpcClient::from_url(args.ws.clone()).await?;
    let client = OnlineClient::<PolkadotConfig>::from_rpc_client(rpc_client.clone()).await?;

    let constant_query = api::constants().staking().sessions_per_era();
    let session_per_era = client.constants().at(&constant_query)?;
    // Subscribe to all finalized blocks:
    let mut blocks_sub = client.blocks().subscribe_finalized().await?;

    // For each block, print a bunch of information about it:
    while let Some(block) = blocks_sub.next().await {
        let block = block?;

        let events = block.events().await?;
        if let Some(new_session) = events.find_first::<NewSession>().ok().flatten() {
            let epoch_index = new_session.session_index;
            println!("New epoch started: {}", epoch_index);

            let epoch_data = blocks_in_epoch(rpc_client.clone(), 1).await?;
            let last_epoch = epoch_data.get(0).expect("hope it exist");
            println!(
                "Epoch {} ended! Total blocks
                 produced: {}",
                last_epoch.0, last_epoch.1
            );
        }

        if let Some(era_paid) = events.find_first::<EraPaid>().ok().flatten() {
            let era_index = era_paid.era_index;
            let epoch_data = blocks_in_epoch(rpc_client.clone(), session_per_era).await?;
            let total_blocks = epoch_data.iter().fold(0, |acc, e| acc + e.1);
            println!(
                "Era {} ended! Total blocks produced: {}",
                era_index, total_blocks
            );
        }
    }

    Ok(())
}
