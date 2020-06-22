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

//! A secure hardware keystore.  Unlike [`SoftStore`], this is considered
//! production-quality.
//!
//! To use this keystore, a Ledger device with the Kusama and/or Polkadot apps
//! installed must be connected, and the process must have permission to use it.

use super::{keys::Signed, AccountId, Encode, Error, KeyStore, LedgeracioPath};
use codec::Decode;
use futures::future::{err, ok};
use ledger_substrate::SubstrateApp;
use std::{convert::From,
          sync::{Arc, Mutex}};
use substrate_subxt::{sp_runtime::{generic::{SignedPayload, UncheckedExtrinsic},
                                   traits::SignedExtension,
                                   MultiSignature as Signature},
                      system::System,
                      Encoded, SignedExtra, Signer};

/// Hardware keystore
pub struct HardStore {
    inner: Arc<Mutex<SubstrateApp>>,
}

impl HardStore {
    pub fn new() -> Result<Self, crate::Error> {
        Ok(Self {
            inner: Arc::new(Mutex::new(SubstrateApp::new(ledger_substrate::APDUTransport {
                transport_wrapper: ledger::TransportNativeHID::new()?,
            }, 0))),
        })
    }
}

struct HardSigner {
    app: Arc<Mutex<SubstrateApp>>,
    path: LedgeracioPath,
    ss58: String,
    address: AccountId,
}

impl<
        T: System<AccountId = AccountId, Address = AccountId> + Send + Sync + 'static,
        S: Encode + Decode + Send + Sync + From<Signature> + 'static,
        E: SignedExtra<T> + 'static,
    > KeyStore<T, S, E> for HardStore
where
    <<E as SignedExtra<T>>::Extra as SignedExtension>::AdditionalSigned:
        Send + Sync + std::fmt::Debug,
{
    fn signer(
        &self,
        path: LedgeracioPath,
    ) -> Result<Box<dyn Signer<T, S, E> + Send + Sync>, Error> {
        let app = self.inner.clone();
        let ledger_address = {
            let inner_app = app.lock().unwrap();
            futures::executor::block_on(inner_app.get_address(path.as_ref(), false))
        };

        let res = {
            let ledger_address = match ledger_address {
                Ok(e) => e,
                Err(e) => {
                    eprintln!(
                        "Failed to obtain a signer for path {:?}: {}.\n\nCheck that your Ledger \
                         device is connected, and that you have the correct app\nopen for the \
                         network you are using. ",
                        path, e
                    );
                    return Err(Box::new(e) as _)
                }
            };
            Ok(Box::new(HardSigner {
                app,
                ss58: ledger_address.ss58,
                path,
                address: ledger_address.public_key.into(),
            })
                as Box<dyn Signer<T, S, E> + Send + Sync + 'static>)
        };
        res
    }
}

impl<T, S, E> Signer<T, S, E> for HardSigner
where
    T: System<AccountId = AccountId, Address = AccountId> + Send + Sync + 'static,
    S: Encode + Decode + Send + Sync + 'static,
    E: SignedExtra<T> + 'static,
    <<E as SignedExtra<T>>::Extra as SignedExtension>::AdditionalSigned:
        Send + Sync + std::fmt::Debug,
{
    fn account_id(&self) -> &AccountId { &self.address }

    fn nonce(&self) -> Option<T::Index> { None }

    fn sign(&self, extrinsic: SignedPayload<Encoded, E::Extra>) -> Signed<T, S, E> {
        let app = self.app.clone();
        let path = self.path.clone();
        let encoded = extrinsic.encode();
        let (call, extra, _) = extrinsic.deconstruct();
        let signature =
            match futures::executor::block_on(app.lock().unwrap().sign(path.as_ref(), &encoded)) {
                Ok(e) => e,
                Err(e) => return Box::pin(err(e.to_string())),
            };
        let signature = match Decode::decode(&mut &signature[..]) {
            Ok(e) => e,
            Err(e) => return Box::pin(err(e.to_string())),
        };
        let account_id = <Self as Signer<T, S, E>>::account_id(self);
        let res = ok(UncheckedExtrinsic::new_signed(
            call,
            account_id.clone().into(),
            signature,
            extra,
        ));
        Box::pin(res)
    }
}
