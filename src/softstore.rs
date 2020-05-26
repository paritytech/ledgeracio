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

use super::{AccountId, AccountType, Error, KeyStore};
use async_std::prelude::*;
use ed25519_bip32::{DerivationScheme::V2, XPrv};
use std::pin::Pin;
const HARDENED: u32 = 1u32 << 31;

/// This is meant for development and testing, and should not be used in
/// production.  Hardware-backed keystores should be used in production.
pub struct SoftKeyStore(XPrv);

impl SoftKeyStore {
    pub fn new(bytes: &[u8; 32], account_type: AccountType, chain_code: &[u8; 32]) -> Self {
        Self(
            XPrv::from_nonextended_force(bytes, chain_code)
                .derive(V2, 0x8000002Cu32)
                .derive(V2, 0x80000162u32)
                .derive(V2, account_type as u32 | 1u32 << 31)
                .derive(V2, 0),
        )
    }
}

impl KeyStore for SoftKeyStore {
    fn get(&self, index: usize) -> Pin<Box<dyn Future<Output = Result<Option<AccountId>, Error>>>> {
        Box::pin(async_std::future::ready(if index >= HARDENED as usize {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid index {}", index),
            )) as Box<dyn std::error::Error>)
        } else {
            Ok(Some(
                self.0
                    .derive(V2, index as u32 | 1u32 << 31)
                    .public()
                    .public_key()
                    .into(),
            ))
        }))
    }

    fn sign(
        &self,
        index: u32,
        message: &[u8],
    ) -> Pin<Box<dyn Future<Output = Result<[u8; 64], Error>>>> {
        if index >= 1u32 << 31 as usize {
            Box::pin(async_std::future::ready(Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid index {}", index),
            ))
                as Box<dyn std::error::Error>)))
        } else {
            Box::pin(async_std::future::ready(Ok(*self
                .0
                .derive(V2, index as u32 | 1u32 << 31)
                .sign::<()>(message)
                .to_bytes())))
        }
    }
}
