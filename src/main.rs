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

mod approved_validators;
mod common;
mod derivation;
mod hardstore;
mod keys;
mod mock;
mod nominator;
mod softstore;
mod validator;

use clap::arg_enum;
use codec::Encode;
use derivation::{AccountType, LedgeracioPath};
use keys::KeyStore;

#[cfg(feature = "insecure_software_keystore")]
use softstore::SoftKeyStore;
use sp_core::crypto::AccountId32 as AccountId;
use std::fmt::Debug;
use structopt::StructOpt;
use substrate_subxt::{sp_core,
                      sp_core::crypto::{Ss58AddressFormat, Ss58Codec},
                      staking::RewardDestination,
                      ClientBuilder};

type Error = Box<dyn std::error::Error + Send + Sync>;

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
    /// A file containing the secret seed.
    ///
    /// By default, a secure hardware-backed keystore is used.  For testing and
    /// debugging, you can pass a file containing the secret seed with this
    /// flag.  This is less secure and should not be used in production.
    #[structopt(parse(from_os_str), long)]
    secret_file: Option<std::path::PathBuf>,
    /// Dry run.  Do not execute the operation.
    #[structopt(short = "n", long)]
    dry_run: bool,
    /// RPC host
    #[structopt(short, long)]
    host: Option<String>,
    /// Network
    #[structopt(long)]
    network: Network,
    /// Subcommand
    #[structopt(subcommand)]
    cmd: Command,
}

arg_enum! {
    #[derive(Debug)]
    enum Network {
        // The Kusama (canary) network
        Kusama,
        // The Polkadot (live) network
        Polkadot,
    }
}

async fn display_path(
    account_type: AccountType,
    keystore: &KeyStore,
    network: Ss58AddressFormat,
    index: u32,
) -> Result<(), Error> {
    let path = LedgeracioPath::new(network, account_type, index)?;
    let signer = keystore.signer(path).await?;
    let account_id: &AccountId = signer.account_id();
    Ok(print!(
        "{}\n",
        account_id.to_ss58check_with_version(network)
    ))
}

#[derive(StructOpt, Debug)]
enum Command {
    /// Nominator operations
    Nominator(nominator::Nominator),
    /// Validator operations
    Validator(validator::Validator),
    /// Allowlist operations
    Allowlist(approved_validators::ACL),
}

type Runtime = substrate_subxt::KusamaRuntime;

fn parse_reward_destination(arg: &str) -> Result<RewardDestination, &'static str> {
    Ok(match arg {
        "Staked" => RewardDestination::Staked,
        "Stash" => RewardDestination::Stash,
        "Controller" => RewardDestination::Controller,
        _ => return Err("bad reward destination ― must be Staked, Stash, or Controller"),
    })
}

/// Parse an SS58 address
pub fn parse_address<T: Ss58Codec>(arg: &str) -> Result<(T, u8), String> {
    Ss58Codec::from_string_with_version(arg)
        .map_err(|e| format!("{:?}", e))
        .map(|(x, y)| (x, y.into()))
}

#[async_std::main]
async fn main() -> Result<(), Error> {
    #[cfg(feature = "insecure_software_keystore")]
    use std::{fs::File, io::Read};
    let Ledgeracio {
        dry_run,
        host,
        network,
        secret_file,
        cmd,
    } = Ledgeracio::from_args();
    let address_format = match network {
        Network::Kusama => Ss58AddressFormat::KusamaAccount,
        Network::Polkadot => Ss58AddressFormat::PolkadotAccount,
    };
    let host = host.unwrap_or_else(|| {
        match network {
            Network::Kusama => "wss://kusama-rpc.polkadot.io",
            Network::Polkadot => "wss://rpc.polkadot.io",
        }
        .to_owned()
    });

    let client = async { ClientBuilder::<Runtime>::new().set_url(host).build().await };
    let keystore: KeyStore = match secret_file {
        #[cfg(feature = "insecure_software_keystore")]
        Some(input) => {
            let mut fh = File::open(input)?;
            let mut v = vec![];
            let _size = fh.read_to_end(&mut v)?;
            let seed = if v.starts_with(b"0x") {
                hex::decode(&v[2..])?
            } else {
                let s: String = String::from_utf8(v)?;
                let mnemonic = bip39::Mnemonic::from_phrase(&*s, bip39::Language::English)?;
                bip39::Seed::new(&mnemonic, "").as_bytes().to_owned()
            };
            SoftKeyStore::new(&*seed)
        }
        #[cfg(not(feature = "insecure_software_keystore"))]
        Some(_input) => {
            return Err("insecure software keystore disabled at compile time"
                .to_owned()
                .into())
        }
        None => keys::KeyStore::Hard(hardstore::HardStore::new(network)?),
    };
    if dry_run {
        return Ok(())
    }
    match cmd {
        Command::Nominator(s) => nominator::main(s, client.await?, address_format, &keystore)
            .await
            .map(drop),
        Command::Validator(v) => validator::main(v, client.await?, address_format, &keystore)
            .await
            .map(drop),
        Command::Allowlist(l) => {
            let hardware = match keystore {
                #[cfg(feature = "insecure_software_keystore")]
                KeyStore::Soft(_) => {
                    return Err("Cannot use an allowlist with a software keystore"
                        .to_owned()
                        .into())
                }
                KeyStore::Hard(h) => h,
            };
            approved_validators::main(l, hardware).await
        }
    }?;
    Ok(())
}
