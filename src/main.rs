mod epoch_blocks;
mod secondary_authors;
mod traverse_chain;
pub mod utils;

use structopt::StructOpt;
use utils::{Command, Opts};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = Opts::from_args();

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
    }

    Ok(())
}
