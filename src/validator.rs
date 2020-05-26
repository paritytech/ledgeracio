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

use super::{AccountId, Error, StructOpt};
use std::marker::PhantomData;
use substrate_subxt::{balances::Balances,
                      sp_runtime::{generic::SignedPayload, traits::SignedExtension, Perbill},
                      staking::{Staking, ValidateCall, ValidatorPrefs},
                      system::System,
                      Client, Encoded, SignedExtra};

#[derive(StructOpt, Debug)]
pub(crate) enum Validator {
    /// Show status of all Validator Controller keys
    Status { index: Option<u32> },
    /// Announce intention to validate
    Announce { index: u32, commission: u32 },
    /// Replace a session key
    ReplaceKey { index: u32 },
    /// Generate new controller keys
    GenerateKeys { count: u32 },
}

pub(crate) async fn main<
    T: System<AccountId = AccountId> + Balances + Send + Sync + Staking + 'static,
    S: 'static,
    E: SignedExtension + SignedExtra<T> + 'static,
>(
    cmd: Validator,
    client: &Client<T, S, E>,
    keystore: &dyn crate::keys::KeyStore,
) -> Result<SignedPayload<Encoded, E::Extra>, Error> {
    match cmd {
        Validator::Announce { index, commission } => {
            let call = ValidateCall {
                prefs: ValidatorPrefs {
                    commission: Perbill::from_parts(commission),
                },
                _runtime: PhantomData,
            };
            let account_id =
                keystore
                    .get(index as _)
                    .await?
                    .ok_or(Box::new(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "No key found",
                    )))?;
            Ok(client.create_raw_payload(&account_id, call).await?)
        }
        Validator::ReplaceKey { index } => unimplemented!("replacing key {}", index),
        Validator::GenerateKeys { count } => unimplemented!("deriving a new key {}", count),
        Validator::Status { index } => unimplemented!("showing the status of key {:?}", index),
    }
}
