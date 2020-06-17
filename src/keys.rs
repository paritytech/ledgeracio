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

//! A trait representing a key store.
//!
//! Performance should not be considered critical.

use crate::{derivation::LedgeracioPath, Error};
use codec::Encode;
use std::{future::Future, pin::Pin};

use substrate_subxt::{sp_runtime::generic::UncheckedExtrinsic, system::System, Encoded,
                      SignedExtra, Signer};

pub type Signed<T, S, E> = Pin<
    Box<
        dyn Future<
                Output = Result<
                    UncheckedExtrinsic<
                        <T as System>::Address,
                        Encoded,
                        S,
                        <E as SignedExtra<T>>::Extra,
                    >,
                    String,
                >,
            > + Send
            + Sync
            + 'static,
    >,
>;

/// A keystore, backed by software or hardware.
pub trait KeyStore<T: System, S: Encode, E: SignedExtra<T>> {
    /// Get a [`Signer`]
    fn signer(&self, path: LedgeracioPath)
        -> Result<Box<dyn Signer<T, S, E> + Send + Sync>, Error>;
}
