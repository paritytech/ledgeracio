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

use super::{AccountId, AccountType, Encode, Error, KeyStore};
use async_std::prelude::*;
use ed25519_bip32::{DerivationScheme::V2, XPrv};
use futures::future::{err, ok};
use std::pin::Pin;
use substrate_subxt::{sp_core::ed25519::Signature,
                      sp_runtime::generic::{SignedPayload, UncheckedExtrinsic},
                      system::System,
                      Encoded, SignedExtra, Signer};
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

type Signed<T, S, E> = Pin<
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
impl<
        T: System<AccountId = AccountId, Address = AccountId> + Send + Sync + 'static,
        S: Encode + Send + Sync + std::convert::From<Signature> + 'static,
        E: SignedExtra<T> + 'static,
    > KeyStore<T, S, E> for SoftKeyStore
{
    fn signer(
        &self,
        index: usize,
    ) -> Pin<Box<dyn Future<Output = Result<Box<dyn Signer<T, S, E> + Send + Sync>, Error>>>> {
        Box::pin(if index >= HARDENED as usize {
            err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid index {}", index),
            )) as Box<dyn std::error::Error>)
        } else {
            ok(Box::new(Self(self.0.derive(V2, index as u32 | 1u32 << 31))) as _)
        })
    }
}

impl<T, S, E> Signer<T, S, E> for SoftKeyStore
where
    T: System<AccountId = AccountId, Address = AccountId> + Send + Sync + 'static,
    S: Encode + Send + Sync + 'static + std::convert::From<Signature>,
    E: SignedExtra<T> + 'static,
{
    fn account_id(&self) -> &AccountId { unimplemented!() }

    fn nonce(&self) -> Option<T::Index> { None }

    fn sign(&self, extrinsic: SignedPayload<Encoded, E::Extra>) -> Signed<T, S, E> {
        let signature = Signature(*self.0.sign::<T>(&extrinsic.encode()).to_bytes());
        let (call, extra, _) = extrinsic.deconstruct();
        let account_id = <Self as Signer<T, S, E>>::account_id(self);
        Box::pin(ok(UncheckedExtrinsic::new_signed(
            call,
            account_id.clone().into(),
            signature.into(),
            extra,
        )))
    }
}
