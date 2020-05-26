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

//! Stash commands

use super::{AccountId, Error, StructOpt};
use substrate_subxt::{balances::Balances,
                      sp_runtime::{generic::SignedPayload, traits::SignedExtension},
                      staking::Staking,
                      system::System,
                      Client, Encoded, SignedExtra};

#[derive(StructOpt, Debug)]
pub(crate) enum Stash {
    /// Show the specified stash controller
    Show { index: u32 },
    /// Show the status of all stash controllers
    Status,
    /// Claim a validation payout
    Claim { index: Option<u32> },
    /// Submit a new validator set
    #[structopt(name = "submit-validator-set")]
    SubmitValidatorSet,
    /// Add a new controller key
    #[structopt(name = "add-controller-key")]
    AddControllerKey,
}

pub(crate) async fn main<
    T: System<AccountId = AccountId> + Balances + Send + Sync + Staking + 'static,
    S: 'static,
    E: SignedExtension + SignedExtra<T> + 'static,
>(
    cmd: Stash,
    _client: &Client<T, S, E>,
    _keystore: &dyn crate::keys::KeyStore,
) -> Result<SignedPayload<Encoded, E::Extra>, Error> {
    match cmd {
        Stash::Status => unimplemented!("showing validator status"),
        Stash::Show { index } => unimplemented!("getting validator status for index {}", index),
        Stash::Claim { index } => unimplemented!("claiming payment for {:?}", index),
        Stash::SubmitValidatorSet => unimplemented!("submitting a validator set"),
        Stash::AddControllerKey => unimplemented!("adding a controller key"),
    }
}
