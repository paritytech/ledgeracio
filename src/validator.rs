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

use super::{parse_reward_destination, AccountType, Error, LedgeracioPath, StructOpt};
use codec::Decode;
use substrate_subxt::{session::SetKeysCallExt,
                      sp_core::crypto::Ss58AddressFormat,
                      sp_runtime::Perbill,
                      staking::{LedgerStore, RewardDestination, SetPayeeCallExt, ValidateCallExt,
                                ValidatorPrefs},
                      system::System,
                      Client, KusamaRuntime, SessionKeys};

#[derive(StructOpt, Debug)]
pub(crate) enum Validator {
    /// Show status of all Validator Controller keys
    Status { index: Option<u32> },
    /// Announce intention to validate
    Announce { index: u32, commission: Option<u32> },
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
        target: RewardDestination,
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

pub(crate) async fn main(
    cmd: Validator,
    client: Client<KusamaRuntime>,
    network: Ss58AddressFormat,
    keystore: &super::keys::KeyStore,
) -> Result<<KusamaRuntime as System>::Hash, Error> {
    match cmd {
        Validator::Announce { index, commission } => {
            let path = LedgeracioPath::new(network, AccountType::Validator, index)?;
            let prefs = ValidatorPrefs {
                commission: Perbill::from_parts(commission.unwrap_or(u32::max_value())),
            };
            let signer = keystore.signer(path).await?;
            Ok(client.validate(&signer, prefs).await?)
        }
        Validator::ReplaceKey { index, keys } => {
            let path = LedgeracioPath::new(network, AccountType::Validator, index)?;
            let signer = keystore.signer(path).await?;
            Ok(client.set_keys(&signer, keys, vec![]).await?)
        }
        Validator::Status { index } => {
            let validators = crate::common::fetch_validators(
                &client,
                network,
                AccountType::Validator,
                keystore,
                index,
            )
            .await?;
            for controller in validators {
                let validators = client.fetch(LedgerStore { controller }, None).await?;
                println!("Validator status: {:#?}", validators);
            }
            Ok(Default::default())
        }
        Validator::SetPayee { index, target } => {
            let path = LedgeracioPath::new(network, AccountType::Validator, index)?;
            let signer = keystore.signer(path).await?;
            Ok(client.set_payee(&signer, target).await?)
        }
        Validator::Address { index } => {
            crate::display_path(AccountType::Validator, keystore, network, index).await?;
            Ok(Default::default())
        }
    }
}
