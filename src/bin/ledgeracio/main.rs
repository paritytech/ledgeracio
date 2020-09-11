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

#![deny(clippy::all, clippy::pedantic)]
#![allow(clippy::non_ascii_literal)]
#![forbid(unsafe_code)]

mod common;
mod nominator;
mod payouts;
mod validator;

use futures::future::TryFutureExt;
use ledgeracio::{get_network, AccountType, Error, HardSigner, HardStore, LedgeracioPath};

#[cfg(not(unix))]
compile_error!("Only *nix-like platforms are supported");

use common::AddressSource;
use sp_core::crypto::AccountId32 as AccountId;
use std::{fmt::Debug, future::Future, pin::Pin};
use structopt::StructOpt;
use substrate_subxt::{sp_core,
                      sp_core::crypto::{Ss58AddressFormat, Ss58Codec},
                      staking::RewardDestination,
                      Client, ClientBuilder, Signer};

#[derive(StructOpt, Debug)]
#[structopt(name = "Ledgeracio", about = "Ledger CLI for staking")]
struct Ledgeracio {
    /// Dry run.  Do not execute the operation.
    #[structopt(short = "n", long)]
    dry_run: bool,
    /// RPC host
    #[structopt(short, long)]
    host: Option<String>,
    /// Network
    #[structopt(long, parse(try_from_str = get_network))]
    network: Ss58AddressFormat,
    /// Subcommand
    #[structopt(subcommand)]
    cmd: Command,
}

async fn display_path(
    account_type: AccountType,
    keystore: &HardStore,
    network: Ss58AddressFormat,
    index: u32,
) -> Result<(), Error> {
    if index == 0 {
        return Err("Index must not be zero".to_owned().into())
    }
    let path = LedgeracioPath::new(network, account_type, index)?;
    let signer: HardSigner = keystore.signer(path).await?;
    let account_id: &AccountId = signer.account_id();
    println!("{}", account_id.to_ss58check_with_version(network));
    Ok(())
}

#[derive(StructOpt, Debug)]
enum Command {
    /// Nominator operations
    Nominator(nominator::Nominator),
    /// Validator operations
    Validator(validator::Validator),
    /// Pretty-print the chain metadata
    Metadata,
    /// Display the chain properties
    Properties,
}

type Runtime = substrate_subxt::KusamaRuntime;

fn parse_reward_destination(arg: &str) -> Result<RewardDestination, Error> {
    Ok(match &*arg.to_ascii_lowercase() {
        "staked" => RewardDestination::Staked,
        "stash" => RewardDestination::Stash,
        "controller" => RewardDestination::Controller,
        _ => {
            return Err("Arbitrary reward destinations not supported"
                .to_owned()
                .into())
        }
    })
}

async fn inner_main() -> Result<(), Error> {
    env_logger::init();
    let Ledgeracio {
        dry_run,
        host,
        network,
        cmd,
    } = Ledgeracio::from_args();
    let host = match (host, network) {
        (Some(host), _) => host,
        (None, Ss58AddressFormat::KusamaAccount) => "wss://kusama-rpc.polkadot.io".into(),
        (None, Ss58AddressFormat::PolkadotAccount) => "wss://rpc.polkadot.io".into(),
        _ => return Err("Please supply an RPC endpoint".into()),
    };

    let client = ClientBuilder::<Runtime>::new()
        .set_url(host)
        .build()
        .map_err(From::from);
    let client: Pin<Box<dyn Future<Output = Result<Client<Runtime>, _>>>> = Box::pin(client);
    let keystore = || HardStore::new(network);
    if dry_run {
        return Ok(())
    }
    if let Some(hash) = match cmd {
        Command::Nominator(s) => nominator::main(s, client, network, keystore).await?,
        Command::Validator(v) => validator::main(v, client, network, keystore).await?,
        Command::Metadata => {
            println!("{:#?}", client.await?.metadata());
            None
        }
        Command::Properties => {
            println!("{:#?}", client.await?.properties());
            None
        }
    } {
        println!("Transaction hash: {:?}", hash);
    }
    Ok(())
}

fn main() {
    match async_std::task::block_on(inner_main()) {
        Ok(()) => (),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1)
        }
    }
}
