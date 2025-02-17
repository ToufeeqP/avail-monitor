#![allow(dead_code)]

use crate::utils::{api, AvailConfig, Opts};
use anyhow::Result;
use api::runtime_types::{sp_consensus_babe::app::Public, sp_consensus_slots::Slot};
use codec::Encode;
use sp_consensus_babe::{BabeAuthorityWeight, Randomness};
use structopt::StructOpt;
use subxt::{
    backend::{legacy::LegacyRpcMethods, rpc::RpcClient},
    client::OnlineClient,
    config::substrate::U256,
};

/// Determines secondary slot authors for all slots in an epoch given a first block number of epoch.
pub async fn find_secondary_authors(block_id: u32) -> Result<()> {
    let args = Opts::from_args();
    // First, create a raw RPC client:
    let rpc_client = RpcClient::from_url(args.ws.clone()).await?;

    // Use this to construct our RPC methods:
    let rpc = LegacyRpcMethods::<AvailConfig>::new(rpc_client.clone());

    // We can use the same client to drive our full Subxt interface too:
    let client = OnlineClient::<AvailConfig>::from_rpc_client(rpc_client).await?;

    let block_hash = rpc
        .chain_get_block_hash(Some(block_id.into()))
        .await?
        .ok_or_else(|| anyhow::anyhow!("Block hash not found for number: {}", block_id))?;

    // Fetch the validator authorities
    let authorities = client
        .storage()
        .at(block_hash)
        .fetch(&api::storage().babe().authorities())
        .await?
        .ok_or_else(|| anyhow::anyhow!("Failed to fetch authorities"))?
        .0;

    println!("Got {} authorities!", authorities.len());

    // Fetch the current slot
    let slot: api::runtime_types::sp_consensus_slots::Slot = client
        .storage()
        .at(block_hash)
        .fetch(&api::storage().babe().current_slot())
        .await?
        .ok_or_else(|| anyhow::anyhow!("Failed to fetch current slot"))?;

    println!("Slot: {:?}", slot);

    // Fetch the randomness
    let randomness: Randomness = client
        .storage()
        .at(block_hash)
        .fetch(&api::storage().babe().randomness())
        .await?
        .ok_or_else(|| anyhow::anyhow!("Failed to fetch randomness"))?;

    println!("Randomness: {:?}", randomness);

    // Fetch the epoch duration in slots
    let constant_query = api::constants().babe().epoch_duration();
    let epoch_duration_in_slots = client.constants().at(&constant_query)?;

    // Get secondary slot owners
    let secondary_authors =
        get_secondary_slot_owners(slot, &authorities[..], randomness, epoch_duration_in_slots);

    // Using session validators will save lot of state queries
    let validators = client
        .storage()
        .at(block_hash)
        .fetch(&api::storage().session().validators())
        .await?
        .ok_or_else(|| anyhow::anyhow!("Failed to fetch validators"))?;
    println!("Got {} validators!", validators.len());

    // Fetch and print the owner of each secondary slot
    for (slot_number, authority_index) in secondary_authors.iter() {
        // println!("Slot: {}, AuthIndex: {}", slot_number, authority_index);
        println!(
            "Slot: {}, Owner: {}",
            slot_number,
            validators
                .get(*authority_index as usize)
                .expect("Length of both babe & session auths is same: qed")
        );
    }

    Ok(())
}

/// This function returns the secondary slot author for every slot from `start_slot` to `end_slot`.
fn get_secondary_slot_owners(
    start_slot: Slot,
    authorities: &[(Public, BabeAuthorityWeight)],
    epoch_randomness: Randomness,
    epoch_duration_in_slots: u64,
) -> Vec<(u64, u32)> {
    let mut authors = Vec::with_capacity(epoch_duration_in_slots as usize);

    // Iterate over each slot from start_slot to start_slot + epoch_duration_in_slots
    for s in start_slot.0..=start_slot.0.saturating_add(epoch_duration_in_slots) {
        let expected_author = secondary_slot_author(Slot(s), authorities, epoch_randomness);
        authors.push((s, expected_author));
    }

    authors
}

/// Get the expected secondary author for the given slot and authorities.
fn secondary_slot_author(
    slot: Slot,
    authorities: &[(Public, BabeAuthorityWeight)],
    randomness: Randomness,
) -> u32 {
    let rand = U256::from((randomness, slot).using_encoded(sp_crypto_hashing::blake2_256));
    let authorities_len = U256::from(authorities.len());
    (rand % authorities_len).as_u32()
}
