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

use super::{Encode, Error, LedgeracioPath};
use async_std::sync::Mutex;
use codec::Decode;
use ledger_substrate::SubstrateApp;
use std::sync::Arc;
use substrate_subxt::{sp_core::crypto::AccountId32 as AccountId,
                      sp_runtime::{generic::{SignedPayload, UncheckedExtrinsic},
                                   MultiSignature},
                      system::System,
                      Encoded, Runtime, SignedExtra};

/// Hardware keystore
pub struct HardStore {
    inner: Arc<Mutex<SubstrateApp>>,
}

impl HardStore {
    pub(crate) fn new(network: super::Network) -> Result<Self, crate::Error> {
        let transport = ledger_substrate::APDUTransport {
            transport_wrapper: ledger::TransportNativeHID::new()?,
        };
        let app = match network {
            super::Network::Polkadot => ledger_substrate::new_polkadot_app,
            super::Network::Kusama => ledger_substrate::new_kusama_app,
        }(transport);
        Ok(Self {
            inner: Arc::new(Mutex::new(app)),
        })
    }
}

#[derive(Clone)]
pub struct HardSigner {
    app: Arc<Mutex<SubstrateApp>>,
    path: LedgeracioPath,
    address: AccountId,
}

impl HardStore {
    pub async fn signer(&self, path: LedgeracioPath) -> Result<HardSigner, Error> {
        let app = self.inner.clone();
        let ledger_address = {
            let inner_app = app.lock().await;
            inner_app.get_address(path.as_ref(), false).await
        };

        let res = {
            let ledger_address = match ledger_address {
                Ok(e) => e,
                Err(e) => {
                    eprintln!(
                        "Failed to obtain a signer for path {}: {}.\n\nCheck that your Ledger \
                         device is connected, and that you have the correct app\nopen for the \
                         network you are using.",
                        path, e
                    );
                    return Err(Box::new(e) as _)
                }
            };
            let address = ledger_address.public_key.into();
            Ok(HardSigner { app, path, address })
        };
        res
    }
}

impl HardSigner {
    pub fn account_id(&self) -> &AccountId { &self.address }

    pub async fn sign<T: Runtime<Signature = MultiSignature>>(
        &self,
        extrinsic: SignedPayload<Encoded, <<T as Runtime>::Extra as SignedExtra<T>>::Extra>,
    ) -> Result<
        UncheckedExtrinsic<
            <T as System>::Address,
            Encoded,
            MultiSignature,
            <<T as Runtime>::Extra as SignedExtra<T>>::Extra,
        >,
        String,
    >
    where
        T: System<AccountId = AccountId, Address = AccountId> + Send + Sync + 'static,
    {
        let app = self.app.clone();
        let path = self.path.clone();
        let encoded = extrinsic.encode();
        let (call, extra, _) = extrinsic.deconstruct();
        let app = app.lock().await;
        let signature = match app.sign(path.as_ref(), &encoded).await {
            Ok(e) => e,
            Err(e) => return Err(e.to_string()),
        };
        let signature = match Decode::decode(&mut &signature[..]) {
            Ok(e) => e,
            Err(e) => return Err(e.to_string()),
        };
        Ok(UncheckedExtrinsic::new_signed(
            call,
            self.address.clone().into(),
            signature,
            extra,
        ))
    }
}
