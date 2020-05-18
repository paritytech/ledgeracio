// Copyright 2019-2020 Parity Technologies (UK) Ltd.
// This file is part of substrate-subxt.
//
// subxt is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// subxt is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with substrate-subxt.  If not, see <http://www.gnu.org/licenses/>.
// Copyright 2019-2020 Parity Technologies (UK) Ltd.
use sp_runtime::Perbill;
use std::path::PathBuf;
use structopt::StructOpt;
use substrate_subxt::session::{CurrentIndexStore, QueuedChangedStore, Session, ValidatorsStore};
use substrate_subxt::KusamaRuntime;
mod mock;

/// Output format
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum OutputFormat {
    /// Human-readable formatted text
    Text,
    /// Machine-parsable JSON output
    JSON,
    /// Spreadsheet-importable CSV output
    CSV,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "JSON" => Ok(Self::JSON),
            "CSV" => Ok(Self::CSV),
            "Text" => Ok(Self::Text),
            _ => Err(format!("invalid output format {:?}", s)),
        }
    }
}

use codec::Encode;
use frame_support::Parameter;
use sp_core::storage::StorageKey;
use sp_runtime::traits::{MaybeDisplay, MaybeSerialize, Member};
use std::{fmt::Debug, marker::PhantomData};
use substrate_subxt::{system::System, Call, ClientBuilder, Metadata, MetadataError, Store};

#[derive(Debug, StructOpt)]
#[structopt(name = "Ledgeracio", about = "Ledger CLI for staking")]
struct Ledgeracio {
    /// Enable verbose output
    #[structopt(short, long)]
    verbose: bool,
    /// Dry run.  Do not execute the operation.
    #[structopt(short = "n", long)]
    dry_run: bool,
    /// USB device to use.  Default is to probe for devices.
    #[structopt(short, long)]
    device: Option<String>,
    /// Output format
    #[structopt(short, long, default_value = "Text")]
    format: OutputFormat,
    /// RPC host
    #[structopt(short, long, default_value = "wss://kusama-rpc.polkadot.io")]
    host: String,
    /// Network
    #[structopt(long, default_value = "Polkadot")]
    network: String,
    /// Subcommand
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt, Debug)]
enum Command {
    /// Stash operations
    Stash(Stash),
    /// Validator operations
    Validator(Validator),
}

#[derive(StructOpt, Debug)]
enum Stash {
    /// Show the specified stash controller
    Show { index: u32 },
    /// Show the status of all stash controllers
    Status,
    /// Claim a validation payout
    Claim { index: Option<u32> },
    /// Submit a new validator set
    #[structopt(name = "submit-validator-set")]
    SubmitValidatorSet,
    /// Add a new controller key
    #[structopt(name = "add-controller-key")]
    AddControllerKey,
}

#[derive(StructOpt, Debug)]
enum Validator {
    /// Show status of all Validator Controller keys
    Status { index: Option<u32> },
    /// Announce intention to validate
    Announce { index: u32 },
    /// Replace a session key
    ReplaceKey { index: u32 },
    /// Generate new controller keys
    GenerateKeys { count: u32 },
}

#[async_std::main]
async fn main() {
    let validators = ValidatorsStore::<KusamaRuntime> {
        _runtime: PhantomData,
    };
    let current_index = CurrentIndexStore::<KusamaRuntime> { _r: PhantomData };
    let queued_change = QueuedChangedStore::<KusamaRuntime> { _r: PhantomData };
    let args = Ledgeracio::from_args();
    println!("{:?}", args);
    let builder: ClientBuilder<substrate_subxt::KusamaRuntime> = ClientBuilder::new();
    let client = builder.set_url(args.host).build().await.unwrap();
    println!(
        "Validator set: {:#?}\nCurrent index: {}\nChange queued: {}\nMock validators: {:#?}",
        client.fetch(validators, None).await.unwrap(),
        client.fetch(current_index, None).await.unwrap(),
        client.fetch(queued_change, None).await.unwrap(),
		mock::validator_list(),
    )
}
