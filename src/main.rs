// Copyright 2020 Parity Technologies (UK) Ltd.
// This file is part of ledgeracio.
//
// ledgeracio is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// ledgeracio is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with ledgeracio.  If not, see <http://www.gnu.org/licenses/>.

//! The main binary of Ledgeracio

mod keys;
mod mock;
mod softstore;
mod stash;
mod validator;

use codec::Encode;
use keys::{AccountType, KeyStore};
use softstore::SoftKeyStore;
use sp_core::crypto::AccountId32 as AccountId;
use std::fmt::Debug;
use structopt::StructOpt;
use substrate_subxt::{sp_core, ClientBuilder};

type Error = Box<dyn std::error::Error>;

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

#[derive(Debug, StructOpt)]
#[structopt(name = "Ledgeracio", about = "Ledger CLI for staking")]
struct Ledgeracio {
    /// Dry run.  Do not execute the operation.
    #[structopt(short = "n", long)]
    dry_run: bool,
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
    Stash(stash::Stash),
    /// Validator operations
    Validator(validator::Validator),
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Ledgeracio {
        dry_run,
        host,
        network: _,
        cmd,
    } = Ledgeracio::from_args();
    let client = ClientBuilder::<substrate_subxt::KusamaRuntime>::new()
        .set_url(host)
        .build()
        .await?;
    let keystore = SoftKeyStore::new(
        &[0; 32],
        match cmd {
            Command::Stash(_) => AccountType::Stash,
            Command::Validator(_) => AccountType::Validator,
        },
        &[0; 32],
    );
    let extrinsic = match cmd {
        Command::Stash(s) => stash::main(s, &client, &keystore).await,
        Command::Validator(v) => validator::main(v, &client, &keystore).await,
    }?;
    if dry_run {
        println!("Transaction to be submitted: {:?}", extrinsic.encode())
    } else {
        let hash = client.submit_extrinsic(extrinsic).await?;
        println!("Transaction hash: {:?}", hash)
    }
    Ok(())
}
