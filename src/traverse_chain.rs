#![allow(dead_code)]
use crate::utils::{api, AvailConfig, Opts};
use anyhow::{bail, Result};
use structopt::StructOpt;
use subxt::{
    backend::{legacy::LegacyRpcMethods, rpc::RpcClient},
    client::OnlineClient,
};

/// Traverse the chain in reverse order, from the start_block to its parent, continuing until end_block is reached.
/// Records the storage values for each block's slot number during the traversal.
pub async fn traverse(start_block: u32, end_block: u32) -> Result<()> {
    if start_block < end_block {
        bail!("start_block should be greater than or equal to the end_block.");
    }

    let args = Opts::from_args();
    let rpc_client = RpcClient::from_url(args.ws.clone()).await?;

    // Initialize both RPC and Subxt client once without redundant cloning
    let rpc = LegacyRpcMethods::<AvailConfig>::new(rpc_client.clone());
    let client = OnlineClient::<AvailConfig>::from_rpc_client(rpc_client).await?;

    println!("{:<10} | {:<10}", "block #", "slot #");

    // Fetch the first block hash
    let mut block_hash = rpc
        .chain_get_block_hash(Some(start_block.into()))
        .await?
        .ok_or_else(|| anyhow::anyhow!("Block hash not found for start_block: {}", start_block))?;

    let mut block = client.blocks().at(block_hash).await?;

    // Traverse until end_block is reached
    while block.number() >= end_block {
        let slot = client
            .storage()
            .at(block_hash)
            .fetch(&api::storage().babe().current_slot())
            .await?
            .ok_or_else(|| anyhow::anyhow!("Failed to fetch slot for block #{}", block.number()))?;

        println!("{:<10} | {:<10}", block.number(), slot.0);

        if block.number() == end_block {
            break;
        }

        // Traverse to the parent block
        block_hash = block.header().parent_hash;
        block = client.blocks().at(block_hash).await?;
    }

    Ok(())
}
