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
use codec::Encode;
use substrate_subxt::{balances::Balances,
                      sp_runtime::{traits::SignedExtension, Perbill},
                      staking::{Staking, ValidateCallExt, ValidatorPrefs},
                      system::System,
                      Client, SignedExtra};

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
    S: Encode + Send + Sync + 'static,
    E: SignedExtension + SignedExtra<T> + 'static,
>(
    cmd: Validator,
    client: &Client<T, S, E>,
    keystore: &(dyn crate::keys::KeyStore<T, S, E> + Send + Sync),
) -> Result<T::Hash, Error>
where
    <<E as SignedExtra<T>>::Extra as SignedExtension>::AdditionalSigned: Send + Sync,
{
    match cmd {
        Validator::Announce { index, commission } => {
            let prefs = ValidatorPrefs {
                commission: Perbill::from_parts(commission),
            };
            let signer = keystore.signer(index as _).await?;
            Ok(client.validate(&signer, prefs).await?)
        }
        Validator::ReplaceKey { index } => unimplemented!("replacing key {}", index),
        Validator::GenerateKeys { count } => unimplemented!("deriving a new key {}", count),
        Validator::Status { index } => unimplemented!("showing the status of key {:?}", index),
    }
}
