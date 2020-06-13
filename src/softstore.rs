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

use super::{keys::Signed, AccountId, Encode, Error, KeyStore, LedgeracioPath};
use async_std::prelude::*;
use ed25519_bip32::{DerivationScheme::V2, XPrv};
use futures::future::ok;
use hmac::Hmac;
use sha2::Sha512;
use std::pin::Pin;
use substrate_subxt::{sp_core::ed25519::Signature,
                      sp_runtime::generic::{SignedPayload, UncheckedExtrinsic},
                      system::System,
                      Encoded, SignedExtra, Signer};

/// A software keystore, backed by a secret file on disk.
///
/// While this has no other dependencies and is convenient, it is significantly
/// less secure than hardware-backed keystores.  It is meant for development and
/// testing, and should not be used in production.  Hardware-backed keystores
/// should be used in production.
pub struct SoftKeyStore(XPrv);

impl SoftKeyStore {
    pub fn new(seed: &[u8]) -> Box<Self> {
        use hmac::crypto_mac::{Mac as _, NewMac as _};
        use std::convert::TryInto as _;
        let mut mac = Hmac::<Sha512>::new_varkey(b"Bitcoin seed").expect("key is valid");
        mac.update(seed);
        let code = mac.finalize().into_bytes();
        let bytes: [u8; 32] = code[..32].try_into().unwrap();
        let chain_code: [u8; 32] = code[32..].try_into().unwrap();
        let private = XPrv::from_nonextended_force(&bytes, &chain_code);
        Box::new(Self(private))
    }
}

struct SoftSigner(XPrv, AccountId);

impl<
        T: System<AccountId = AccountId, Address = AccountId> + Send + Sync + 'static,
        S: Encode + Send + Sync + std::convert::From<Signature> + 'static,
        E: SignedExtra<T> + 'static,
    > KeyStore<T, S, E> for SoftKeyStore
{
    fn signer(
        &self,
        path: LedgeracioPath,
    ) -> Pin<Box<dyn Future<Output = Result<Box<dyn Signer<T, S, E> + Send + Sync>, Error>>>> {
        {
            let prv: &[u32] = path.as_ref();
            let prv = prv
                .iter()
                .fold(self.0.clone(), |key, index| key.derive(V2, *index));
            let r#pub = prv.public().public_key().into();
            Box::pin(ok(Box::new(SoftSigner(prv, r#pub)) as _))
        }
    }
}

impl<T, S, E> Signer<T, S, E> for SoftSigner
where
    T: System<AccountId = AccountId, Address = AccountId> + Send + Sync + 'static,
    S: Encode + Send + Sync + 'static + std::convert::From<Signature>,
    E: SignedExtra<T> + 'static,
{
    fn account_id(&self) -> &AccountId { &self.1 }

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
