use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Opts {
    #[structopt(long, default_value = "ws://127.0.0.1:9944")]
    pub ws: String,
}

#[subxt::subxt(runtime_metadata_path = "./artifacts/polkadot_metadata.scale")]
pub mod api {}
