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

use crate::{AccountId, Error};
use std::{future::Future, pin::Pin};

/// The account type
#[repr(u8)]
pub enum AccountType {
    /// Stash accounts
    Stash = 0,
    /// Validator accounts
    Validator = 1,
}

/// A keystore, backed by software or hardware.
pub trait KeyStore {
    /// The public keys, as account IDs.
    fn get(&self, index: usize) -> Pin<Box<dyn Future<Output = Result<Option<AccountId>, Error>>>>;
    /// Sign the given message asynchronously.
    ///
    /// This may fail for several reasons, including the operation being refused
    /// by the user.
    ///
    /// The returned error code is meant for human consumption.
    fn sign(
        &self,
        index: u32,
        message: &[u8],
    ) -> Pin<Box<dyn Future<Output = Result<[u8; 64], Error>>>>;
}
