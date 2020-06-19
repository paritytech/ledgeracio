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

use super::{AccountId, AccountType, Error, LedgeracioPath, StructOpt};
use codec::{Decode, Encode};
use substrate_subxt::{balances::Balances,
                      session::{Session, SetKeysCallExt},
                      sp_core::crypto::Ss58AddressFormat,
                      sp_runtime::{traits::SignedExtension, Perbill},
                      staking::{LedgerStore, Staking, ValidateCallExt, ValidatorPrefs},
                      system::System,
                      Client, SessionKeys, SignedExtra};

#[derive(StructOpt, Debug)]
pub(crate) enum Validator {
    /// Show status of all Validator Controller keys
    Status { index: Option<u32> },
    /// Announce intention to validate
    Announce { index: u32, commission: u32 },
    /// Replace a session key
    ReplaceKey {
        index: u32,
        #[structopt(parse(try_from_str = parse_keys))]
        keys: SessionKeys,
    },
}

fn parse_keys(buffer: &str) -> Result<SessionKeys, Error> {
    let buffer: &[u8] = buffer.as_ref();
    if !buffer.starts_with(b"0x") {
        return Err("Hex data must start with ‘0x’".to_owned().into())
    }
    let bytes = ::hex::decode(&buffer[2..])?;
    Decode::decode(&mut &*bytes).map_err(|e| Box::new(e) as _)
}

pub(crate) async fn main<
    T: System<AccountId = AccountId>
        + Balances
        + Send
        + Sync
        + Staking
        + 'static
        + Session<Keys = SessionKeys>
        + std::fmt::Debug,
    S: Encode + Send + Sync + 'static,
    E: SignedExtension + SignedExtra<T> + 'static,
>(
    cmd: Validator,
    client: Client<T, S, E>,
    network: Ss58AddressFormat,
    keystore: &(dyn crate::keys::KeyStore<T, S, E> + Send + Sync),
) -> Result<T::Hash, Error>
where
    <<E as SignedExtra<T>>::Extra as SignedExtension>::AdditionalSigned: Send + Sync,
{
    match cmd {
        Validator::Announce { index, commission } => {
            let path = LedgeracioPath::new(network, AccountType::Validator, index)?;
            let prefs = ValidatorPrefs {
                commission: Perbill::from_parts(commission),
            };
            let signer = keystore.signer(path)?;
            Ok(client.validate(&*signer, prefs).await?)
        }
        Validator::ReplaceKey { index, keys } => {
            let path = LedgeracioPath::new(network, AccountType::Validator, index)?;
            let signer = keystore.signer(path)?;
            Ok(client.set_keys(&*signer, keys, vec![]).await?)
        }
        Validator::GenerateKeys { count } => unimplemented!("deriving a new key {}", count),
        Validator::Status { index } => {
            let path = LedgeracioPath::new(
                network,
                AccountType::Validator,
                index.expect("account enumeration not implemented"),
            )?;
            let controller: AccountId = keystore.signer(path)?.account_id().clone();
            let nominators = client.fetch(LedgerStore { controller }, None).await?;
            println!("Validator status: {:#?}", nominators);
            Ok(Default::default())
        }
    }
}
