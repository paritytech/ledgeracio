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

mod derivation;
mod hardstore;
mod keys;
mod mock;
mod softstore;
mod stash;
mod validator;

use codec::Encode;
use derivation::{AccountType, LedgeracioPath};
use keys::KeyStore;
use softstore::SoftKeyStore;
use sp_core::crypto::AccountId32 as AccountId;
use std::fmt::Debug;
use structopt::StructOpt;
use substrate_subxt::{sp_core,
                      sp_core::crypto::{Ss58AddressFormat, Ss58Codec},
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
    #[structopt(short, long, default_value = "wss://kusama-rpc.polkadot.io")]
    host: String,
    /// Network
    #[structopt(long, default_value = "polkadot")]
    network: String,
    /// Subcommand
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt, Debug)]
enum KeySource {
    /// Hardware device
    Hardware,
}

#[derive(StructOpt, Debug)]
enum Command {
    /// Stash operations
    Stash(stash::Stash),
    /// Validator operations
    Validator(validator::Validator),
    /// Show a public key
    Address { t: AccountType, index: u32 },
}

type Runtime = substrate_subxt::KusamaRuntime;

/// Parse an SS58 address
pub fn parse_address<T: Ss58Codec>(arg: &str) -> Result<(T, u8), String> {
    Ss58Codec::from_string_with_version(arg)
        .map_err(|e| format!("{:?}", e))
        .map(|(x, y)| (x, y.into()))
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use std::{convert::TryFrom, fs::File, io::Read};
    let Ledgeracio {
        dry_run,
        host,
        network,
        secret_file,
        cmd,
    } = Ledgeracio::from_args();
    let network = Ss58AddressFormat::try_from(&*network)
        .map_err(|()| format!("unsupported network {:?}", network))?;
    let client = async { ClientBuilder::<Runtime>::new().set_url(host).build().await };
    let keystore: Box<dyn KeyStore<Runtime, _, _> + Send + Sync> = match secret_file {
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
        None => Box::new(hardstore::HardStore::new()?),
    };
    if dry_run {
        return Ok(())
    }
    match cmd {
        Command::Stash(s) => stash::main(s, client.await?, network, &*keystore).await,
        Command::Validator(v) => validator::main(v, client.await?, network, &*keystore).await,
        Command::Address { index, t } => {
            let path = LedgeracioPath::new(network, t, index)?;
            let signer = keystore.signer(path).await.unwrap();
            let account_id: &AccountId = signer.account_id();
            println!(
                "{}",
                <AccountId as Ss58Codec>::to_ss58check_with_version(account_id, network)
            );
            return Ok(())
        }
    }?;
    Ok(())
}
