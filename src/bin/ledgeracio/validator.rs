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

//! Validator commands: implements operations on the allowed validators,
//! e.g. rotating session keys, set payment target, announcing
//! intention to validate etc. Requires a network connection.

use super::{common::parse_ppb, parse_reward_destination, AccountType, AddressSource, Error,
            LedgeracioPath, StructOpt};
use codec::Decode;
use core::{future::Future, pin::Pin};
use ledgeracio::parse_address;
use substrate_subxt::{session::SetKeysCallExt,
                      sp_core::{crypto::{AccountId32 as AccountId, Ss58AddressFormat},
                                H256},
                      sp_runtime::Perbill,
                      staking::{BondedStore, ChillCallExt, RewardDestination, SetPayeeCallExt,
                                ValidateCallExt, ValidatorPrefs},
                      Client, KusamaRuntime, SessionKeys};

#[derive(StructOpt, Debug)]
pub(crate) enum Validator {
    /// Show the status of the given validator address.  This does not require a
    /// Ledger device.
    ShowAddress {
        #[structopt(parse(try_from_str = parse_address))]
        address: (AccountId, u8),
    },
    /// Show status of the given Validator Controller key, or all if none is
    /// specified.
    Show { index: Option<u32> },
    /// Announce intention to validate
    Announce {
        index: u32,
        #[structopt(parse(try_from_str = parse_ppb))]
        commission: Option<u32>,
    },
    /// Chill (announce intention to cease validation)
    Chill { index: u32 },
    /// Replace a session key
    ReplaceKey {
        index: u32,
        #[structopt(parse(try_from_str = parse_keys))]
        keys: SessionKeys,
    },
    /// Set payment target
    #[structopt(name = "set-payee")]
    SetPayee {
        index: u32,
        #[structopt(parse(try_from_str = parse_reward_destination))]
        target: RewardDestination<AccountId>,
    },
    /// Display the address of the given index
    Address { index: u32 },
}

fn parse_keys(buffer: &str) -> Result<SessionKeys, Error> {
    let buffer: &[u8] = buffer.as_ref();
    if !buffer.starts_with(b"0x") {
        return Err("Hex data must start with ‘0x’".to_owned().into())
    }
    let bytes = ::hex::decode(&buffer[2..])?;
    Decode::decode(&mut &*bytes).map_err(|e| Box::new(e) as _)
}

pub(crate) async fn main<T: FnOnce() -> Result<super::HardStore, Error>>(
    cmd: Validator,
    client: Pin<Box<dyn Future<Output = Result<Client<KusamaRuntime>, Error>>>>,
    network: Ss58AddressFormat,
    keystore: T,
) -> Result<Option<H256>, Error> {
    match cmd {
        Validator::ShowAddress {
            address: (stash, provided_network),
        } => {
            ledgeracio::validate_network("", provided_network, network)?;
            let client = client.await?;
            let controller = match client.fetch(&BondedStore { stash }, None).await? {
                Some(controller) => controller,
                None => return Err("Controller not found for stash".to_owned().into()),
            };
            crate::common::display_validators(&client, &[controller], network).await?;
            Ok(None)
        }
        Validator::Announce { index, commission } => {
            let path = LedgeracioPath::new(network, AccountType::Validator, index)?;
            let commission = commission.unwrap_or(1_000_000_000);
            if commission > 1_000_000_000 {
                return Err(format!("Commission {} too large (limit is 10⁹)", commission).into())
            }
            let prefs = ValidatorPrefs {
                commission: Perbill::from_parts(commission),
            };
            let signer = keystore()?.signer(path).await?;
            Ok(Some(client.await?.validate(&signer, prefs).await?))
        }
        Validator::Chill { index } => {
            let path = LedgeracioPath::new(network, AccountType::Validator, index)?;
            let signer = keystore()?.signer(path).await?;
            Ok(Some(client.await?.chill(&signer).await?))
        }
        Validator::ReplaceKey { index, keys } => {
            let path = LedgeracioPath::new(network, AccountType::Validator, index)?;
            let signer = keystore()?.signer(path).await?;
            Ok(Some(client.await?.set_keys(&signer, keys, vec![]).await?))
        }
        Validator::Show { index } => {
            let client = client.await?;
            // These are *controller*, not *stash*, accounts.
            let validators = crate::common::fetch_validators(
                &client,
                AddressSource::Device(index, &keystore()?),
                network,
                AccountType::Validator,
            )
            .await?;
            crate::common::display_validators(&client, &*validators, network).await?;
            Ok(None)
        }
        Validator::SetPayee { index, target } => {
            let path = LedgeracioPath::new(network, AccountType::Validator, index)?;
            let signer = keystore()?.signer(path).await?;
            client.await?.set_payee(&signer, target).await?;
            Ok(None)
        }
        Validator::Address { index } => {
            crate::display_path(AccountType::Validator, &keystore()?, network, index).await?;
            Ok(None)
        }
    }
}
