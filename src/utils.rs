use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Opts {
    #[structopt(long, default_value = "ws://127.0.0.1:9944")]
    pub ws: String,

    #[structopt(subcommand)]
    pub command: Command,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    /// Traverse the chain in reverse order, from the start_block to its parent, continuing until end_block is reached.
    /// Records the storage values for each block's slot number during the traversal.
    Traverse {
        #[structopt(help = "Start block number")]
        start_block: u32,
        #[structopt(help = "End block number")]
        end_block: u32,
    },
    /// Fetch number of blocks produced in each epoch for the last `n` epochs
    EpochBlocks {
        #[structopt(help = "Number of epochs to fetch")]
        epochs: u32,
    },
    /// Determine secondary slot authors for an epoch
    SecondaryAuthors {
        #[structopt(help = "Block number at which epoch started")]
        block_id: u32,
    },
}

#[subxt::subxt(runtime_metadata_path = "./artifacts/polkadot_metadata.scale")]
pub mod api {}
