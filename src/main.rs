mod avail_api;
mod epoch_blocks;
mod secondary_authors;
mod traverse_chain;
pub mod utils;

use log::info;
use std::net::SocketAddr;
use structopt::StructOpt;
use utils::{Command, Opts};
use warp::Filter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = Opts::from_args();

    // Health check endpoint
    let health_route = warp::path!("health").map(|| warp::reply::json(&"OK"));

    // Start the health check server
    let mut health_port = opts.health_port;
    let mut addr: SocketAddr = ([0, 0, 0, 0], health_port).into();
    let mut bound = false;

    while !bound {
        if let Ok(_) = tokio::net::TcpListener::bind(addr).await {
            info!("Health check server running on port {}", health_port);
            tokio::spawn(warp::serve(health_route.clone()).run(addr));
            bound = true;
        } else {
            health_port += 1;
            addr = ([0, 0, 0, 0], health_port).into();
        }
    }

    match opts.command {
        Command::Traverse {
            start_block,
            end_block,
        } => {
            traverse_chain::traverse(start_block, end_block).await?;
        }
        Command::EpochBlocks { epochs } => {
            epoch_blocks::fetch_blocks_in_epochs(epochs).await?;
        }
        Command::SecondaryAuthors { block_id } => {
            secondary_authors::find_secondary_authors(block_id).await?;
        }
        Command::ChainMonitor { channel_id } => {
            epoch_blocks::monitor_chain(channel_id).await?;
        }
    }

    Ok(())
}
