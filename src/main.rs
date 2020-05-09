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
use std::path::PathBuf;
use structopt::StructOpt;

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
use substrate_subxt::{system::System, ClientBuilder, Metadata, MetadataError, Store};
use substrate_subxt_proc_macro::Store;

/// The trait needed for this module.
pub trait Session: System {
    /// The validator account identifier type for the runtime.
    type ValidatorId: Parameter + Member + MaybeSerialize + Debug + MaybeDisplay + Ord + Default;

    /// The validator account identifier type for the runtime.
    type SessionIndex: Parameter + Member + MaybeSerialize + Debug + MaybeDisplay + Ord + Default;
}

const MODULE: &str = "Session";

/// The current set of validators.
#[derive(Encode, Store)]
#[cfg(feature = "broken-proc-macro")]
pub struct ValidatorsStore<'a, T: Session> {
    /// The current set of validators.
    #[store(returns = Vec<<T as Session>::ValidatorId>)]
    pub account_id: PhantomData<&'a Vec<<T as Session>::ValidatorId>>,
}

/// The current set of validators.
#[cfg(not(feature = "broken-proc-macro"))]
#[derive(Encode)]
pub struct Validators<'a, T: Session>(pub PhantomData<&'a Vec<<T as Session>::ValidatorId>>);
impl<'a, T: Session> Store<T> for Validators<'a, T> {
    type Returns = Vec<<T as Session>::ValidatorId>;

    const FIELD: &'static str = "Validators";
    const MODULE: &'static str = MODULE;

    fn key(&self, metadata: &Metadata) -> Result<StorageKey, MetadataError> {
        Ok(metadata
            .module(Self::MODULE)?
            .storage(Self::FIELD)?
            .plain()?
            .key())
    }
}

/// Current index of the session.
#[derive(Encode, Store)]
pub struct CurrentIndexStore<'a, T: Session> {
    /// Current index of the session.
    #[store(returns = <T as Session>::SessionIndex)]
    pub current_index: PhantomData<&'a T>,
}

/// True if the underlying economic identities or weighting behind the
/// validators has changed in the queued validator set.
#[derive(Encode, Store)]
pub struct QueuedChanged<'a, T: Session> {
    /// True if the underlying economic identities or weighting behind the
    /// validators has changed in the queued validator set.
    #[store(returns = bool)]
    pub queue_changed: PhantomData<&'a T>,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "Ledgeracio", about = "Ledger CLI for staking")]
struct Ledgeracio {
    /// Enable verbose output
    #[structopt(short, long)]
    verbose: bool,
    /// File containing the PIN. Default is to prompt interactively for the PIN.
    ///
    /// If standard input is not a PTY, operations that require a PIN will error
    /// out if this option is not passed.
    #[structopt(short, long, parse(from_os_str))]
    pin_file: Option<PathBuf>,
    /// Dry run.  Do not execute the operation.
    #[structopt(short = "n", long)]
    dry_run: bool,
    /// USB device to use.  Default is to probe for devices.
    #[structopt(short, long)]
    device: Option<String>,
    /// Interactive mode.  Not yet implemented.  This is the default if no
    /// options are specified.
    #[structopt(short, long)]
    interactive: bool,
    /// Output format
    #[structopt(short, long)]
    format: Option<OutputFormat>,
    /// RPC host
    #[structopt(short, long)]
    host: Option<String>,
    /// Network.  Default is “Polkadot”.
    #[structopt(long)]
    network: Option<String>,
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
struct Count {
    count: u32,
}

#[derive(StructOpt, Debug)]
struct ValidatorIndex {
    index: u32,
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

impl Session for substrate_subxt::KusamaRuntime {
    type SessionIndex = u32;
    type ValidatorId = <Self as System>::AccountId;
}

#[async_std::main]
async fn main() {
    let args = Ledgeracio::from_args();
    println!("{:?}", args);
    let builder: ClientBuilder<substrate_subxt::KusamaRuntime> = ClientBuilder::new();
    let client = builder
        .set_url(
            args.host
                .unwrap_or("wss://kusama-rpc.polkadot.io".to_string()),
        )
        .build()
        .await
        .unwrap();
    println!(
        "Fetch result: {:#?}",
        client
            .fetch::<Validators<substrate_subxt::KusamaRuntime>>(Validators(PhantomData), None)
            .await
            .unwrap()
            .unwrap()
    )
}
