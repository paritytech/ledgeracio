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

//! A software keystore.

use super::{keys::KeyStore, AccountId};
use async_std::prelude::*;

/// This is meant for development and testing, and should not be used in
/// production.  Hardware-backed keystores should be used in production.
pub struct SoftKeyStore;

impl KeyStore for SoftKeyStore {
    fn get(
        &self,
        _index: usize,
    ) -> Box<dyn Future<Output = Result<Option<AccountId>, String>> + Unpin> {
        unimplemented!("BIP 32 Derivation")
    }
    fn sign(
        &self,
        _message: &[u8],
    ) -> Box<dyn Future<Output = Result<(Vec<u8>, Vec<u8>), String>> + Unpin> {
        unimplemented!("Signing")
    }
}
