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

use super::StructOpt;
use std::future::Future;
use substrate_subxt::{
    balances::Balances,
    sp_runtime::{generic::SignedPayload, traits::SignedExtension, Perbill},
    staking::{Staking, ValidateCall, ValidatorPrefs},
    system::System,
    Client, Encoded, Error, SignedExtra,
};

#[derive(StructOpt, Debug)]
pub(crate) enum Validator {
    /// Show status of all Validator Controller keys
    Status { index: Option<u32> },
    /// Announce intention to validate
    Announce { index: u32 },
    /// Replace a session key
    ReplaceKey { index: u32 },
    /// Generate new controller keys
    GenerateKeys { count: u32 },
}

/// Announce intention to validate
pub(crate) async fn validate<
    T: System + Balances + Send + Sync + Staking + 'static,
    S: 'static,
    E: SignedExtension + SignedExtra<T> + 'static,
>(
    account_id: &<T as System>::AccountId,
    client: &Client<T, S, E>,
    commission: Perbill,
) -> Result<SignedPayload<Encoded, <E as SignedExtra<T>>::Extra>, Error> {
    let prefs = ValidatorPrefs { commission };
    let _runtime = core::marker::PhantomData;
    client.create_raw_payload(account_id, ValidateCall { prefs, _runtime }).await
}
