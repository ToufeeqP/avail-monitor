#![allow(dead_code)]

use crate::utils::{
    api,
    api::{session::events::NewSession, staking::events::EraPaid},
    AvailConfig, Opts,
};
use anyhow::Result;
use log::{error, info};
use reqwest::Client;
use serde_json::json;
use sp_core::H256;
use std::{collections::HashSet, env};
use structopt::StructOpt;
use subxt::{
    backend::{legacy::LegacyRpcMethods, rpc::RpcClient},
    client::OnlineClient,
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
    let rpc = LegacyRpcMethods::<AvailConfig>::new(rpc_client.clone());
    // We can use the same client to drive our full Subxt interface too:
    let client = OnlineClient::<AvailConfig>::from_rpc_client(rpc_client).await?;

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
pub async fn monitor_chain(channel_id: Option<String>) -> Result<()> {
    let args = Opts::from_args();
    let rpc_client = RpcClient::from_url(args.ws.clone()).await?;
    let client = OnlineClient::<AvailConfig>::from_rpc_client(rpc_client.clone()).await?;

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
            info!("New epoch started: {}", epoch_index);

            let epoch_data = blocks_in_epoch(rpc_client.clone(), 1).await?;
            let last_epoch = epoch_data.first().expect("we know it exist");
            let message = format!(
                "Epoch {} ended! Total blocks produced: {}",
                last_epoch.0, last_epoch.1
            );
            if let Some(ref channel) = channel_id {
                post_to_slack(&message, channel).await?;
            }
            info!("{}", message);
        }

        if let Some(era_paid) = events.find_first::<EraPaid>().ok().flatten() {
            let era_index = era_paid.era_index;
            let epoch_data = blocks_in_epoch(rpc_client.clone(), session_per_era).await?;
            let total_blocks = epoch_data.iter().fold(0, |acc, e| acc + e.1);
            let message = format!(
                "Era {} ended! Total blocks produced: {}",
                era_index, total_blocks
            );
            if let Some(ref channel) = channel_id {
                post_to_slack(&message, channel).await?;
            }
            info!("{}", message);

            // Check if there are any changes in the active set has happened
            let current_validators = fetch_validators(client.clone(), block.hash()).await?;
            let previous_validators =
                fetch_validators(client.clone(), block.header().parent_hash).await?;

            let added_validators: HashSet<_> = current_validators
                .difference(&previous_validators)
                .cloned()
                .collect();
            let removed_validators: HashSet<_> = previous_validators
                .difference(&current_validators)
                .cloned()
                .collect();
            if !added_validators.is_empty() || !removed_validators.is_empty() {
                let change_message = format!(
                    "Era {} validator set changes:\nAdded: {:?}\nRemoved: {:?}",
                    era_index + 1, added_validators, removed_validators
                );
                if let Some(ref channel) = channel_id {
                    post_to_slack(&change_message, channel).await?;
                }
                info!("{}", change_message);
            } else {
                let change_message = format!("No validator changes in era {}", era_index + 1);
                if let Some(ref channel) = channel_id {
                    post_to_slack(&change_message, channel).await?;
                }
                info!("{}", change_message);
            }
        }
    }

    Ok(())
}

async fn fetch_validators(
    client: OnlineClient<AvailConfig>,
    block_hash: H256,
) -> Result<HashSet<String>> {
    let validators = client
        .storage()
        .at(block_hash)
        .fetch(&api::storage().session().validators())
        .await?
        .ok_or_else(|| anyhow::anyhow!("Failed to fetch validators"))?;
    let validator_hash_set: HashSet<String> =
        validators.into_iter().map(|a| a.to_string()).collect();
    Ok(validator_hash_set)
}

async fn post_to_slack(message: &str, channel_id: &str) -> Result<()> {
    let slack_token = env::var("SLACK_TOKEN").unwrap_or_else(|_| "MAYBE_DEFAULT".to_string());

    let client = Client::new();

    let payload = json!({
        "channel": channel_id,
        "text": message,
    });

    // Send the POST request to Slack Web API
    let response = client
        .post("https://slack.com/api/chat.postMessage")
        .bearer_auth(slack_token)
        .json(&payload)
        .send()
        .await?;

    let status = response.status();
    if status.is_success() {
        info!("Message posted successfully!");
    } else {
        let body = response.text().await?;
        error!(
            "Failed to post message. Status: {:?}, Body: {}",
            status, body
        );
    }

    Ok(())
}
