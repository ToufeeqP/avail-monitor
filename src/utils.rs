use crate::avail_api;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Opts {
    #[structopt(long, default_value = "ws://127.0.0.1:9944")]
    pub ws: String,

    #[structopt(long, default_value = "3030")]
    pub health_port: u16,

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
    /// Monitors chain to determine number of blocks produced in an epoch/era when it ends
    ChainMonitor {
        /// Optional Slack channel ID. If specified, chain information will be posted to Slack.
        /// Ensure the SLACK_TOKEN env is set.
        #[structopt(short, long)]
        channel_id: Option<String>,
    },
}

// #[subxt::subxt(runtime_metadata_path = "./artifacts/polkadot_metadata.scale")]
// pub mod api {}

pub use avail_api::api;

use codec::{Decode, Encode};
use core::marker::PhantomData;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sp_core::H256;
use subxt::{
    config::{
        substrate::{BlakeTwo256, Digest, DigestItem, SubstrateExtrinsicParams},
        Hasher, Header,
    },
    utils::{AccountId32, MultiAddress, MultiSignature},
    Config,
};

use api::runtime_types::{
    avail_core::header::{extension::HeaderExtension, Header as ApiHeader},
    sp_runtime::generic::digest::{Digest as ApiDigest, DigestItem as ApiDigestItem},
};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AvailHeader {
    pub parent_hash: H256,
    #[serde(serialize_with = "number_to_hex", deserialize_with = "number_from_hex")]
    #[codec(compact)]
    pub number: u32,
    pub state_root: H256,
    pub extrinsics_root: H256,
    pub digest: Digest,
    pub extension: HeaderExtension,
}

impl AvailHeader {
    pub fn data_root(&self) -> H256 {
        match &self.extension {
            HeaderExtension::V3(ext) => ext.commitment.data_root,
        }
    }
}

impl Header for AvailHeader {
    type Hasher = BlakeTwo256;
    type Number = u32;

    fn number(&self) -> Self::Number {
        self.number
    }

    fn hash(&self) -> <Self::Hasher as Hasher>::Output {
        Self::Hasher::hash_of(self)
    }
}

fn number_to_hex<S>(value: &u32, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let hex_string = format!("{:X}", value);
    serializer.serialize_str(&hex_string)
}

fn number_from_hex<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;
    let without_prefix = buf.trim_start_matches("0x");
    Ok(u32::from_str_radix(without_prefix, 16).unwrap())
}

impl<B, H> From<AvailHeader> for ApiHeader<B, H>
where
    B: From<u32>,
{
    fn from(h: AvailHeader) -> Self {
        Self {
            parent_hash: h.parent_hash,
            number: h.number.into(),
            state_root: h.state_root,
            extrinsics_root: h.extrinsics_root,
            digest: h.digest.into(),
            extension: h.extension,
            __ignore: PhantomData,
        }
    }
}

impl From<Digest> for ApiDigest {
    fn from(d: Digest) -> Self {
        let logs = d
            .logs
            .into_iter()
            .map(|xt_item| xt_item.into())
            .collect::<Vec<_>>();
        Self { logs }
    }
}

impl From<DigestItem> for ApiDigestItem {
    fn from(di: DigestItem) -> Self {
        match di {
            DigestItem::PreRuntime(id, data) => ApiDigestItem::PreRuntime(id, data),
            DigestItem::Consensus(id, data) => ApiDigestItem::Consensus(id, data),
            DigestItem::Seal(id, data) => ApiDigestItem::Seal(id, data),
            DigestItem::Other(data) => ApiDigestItem::Other(data),
            DigestItem::RuntimeEnvironmentUpdated => ApiDigestItem::RuntimeEnvironmentUpdated,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct AvailConfig;

impl Config for AvailConfig {
    type AccountId = AccountId32;
    type Address = MultiAddress<Self::AccountId, u32>;
    type ExtrinsicParams = SubstrateExtrinsicParams<Self>;
    type Hash = H256;
    type Hasher = BlakeTwo256;
    type Header = AvailHeader;
    type Signature = MultiSignature;
    type AssetId = u32;
}
