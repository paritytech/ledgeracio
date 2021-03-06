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
use codec::Decode;
use ledger_substrate::SubstrateApp;
use std::{future::Future, pin::Pin, sync::Arc};
use substrate_subxt::{sp_core::crypto::{AccountId32 as AccountId, Ss58AddressFormat},
                      sp_runtime::{generic::{SignedPayload, UncheckedExtrinsic},
                                   MultiSignature},
                      system::System,
                      Encoded, Runtime, SignedExtra};

/// Hardware keystore
pub struct HardStore {
    inner: Arc<SubstrateApp>,
}

pub type Signed<T> = Pin<
    Box<
        dyn Future<
                Output = Result<
                    UncheckedExtrinsic<
                        <T as System>::Address,
                        Encoded,
                        MultiSignature,
                        <<T as Runtime>::Extra as SignedExtra<T>>::Extra,
                    >,
                    String,
                >,
            > + Send
            + 'static,
    >,
>;

impl HardStore {
    /// Creates an instance of [`HardStore`].
    ///
    /// # Errors
    ///
    /// This will return `Err` if the network is unsupported or an I/O error
    /// occurs.
    pub fn new(network: Ss58AddressFormat) -> Result<Self, crate::Error> {
        let transport = ledger_substrate::APDUTransport {
            transport_wrapper: ledger::TransportNativeHID::new()?,
        };
        let app = match network {
            Ss58AddressFormat::PolkadotAccount => ledger_substrate::new_polkadot_app,
            Ss58AddressFormat::KusamaAccount => ledger_substrate::new_kusama_app,
            _ => return Err(format!("Unsupported network {}", network).into()),
        }(transport);
        Ok(Self {
            inner: Arc::new(app),
        })
    }
}

#[derive(Clone)]
pub struct HardSigner {
    app: Arc<SubstrateApp>,
    path: LedgeracioPath,
    address: AccountId,
}

impl HardStore {
    /// Open a handle to the Ledger app on the attached Ledger device.  Both the
    /// general and Ledgeracio versions of the Kusama and Polkadot apps will
    /// work.
    ///
    /// # Errors
    ///
    /// This function will fail if no Ledger is inserted, the Ledger is not open
    /// to the correct app, or if there is an error communicating with the
    /// Ledger device.
    pub async fn signer(&self, path: LedgeracioPath) -> Result<HardSigner, Error> {
        let app = self.inner.clone();
        let ledger_address = app.get_address(path.as_ref(), false).await;

        let ledger_address = match ledger_address {
            Ok(e) => e,
            Err(e) => {
                eprintln!(
                    "Failed to obtain a signer for path {}: {}.\n\nCheck that your Ledger device \
                     is connected, and that you have the correct app\nopen for the network you \
                     are using.",
                    path, e
                );
                return Err(Box::new(e) as _)
            }
        };
        let address = ledger_address.public_key.into();
        Ok(HardSigner { app, path, address })
    }

    /// Set a public key
    ///
    /// # Errors
    ///
    /// This function will fail if the device refuses the operation, the app is
    /// not the special Ledgeracio app, or an I/O error occurs.
    pub async fn set_pubkey(&self, key: &'_ [u8; 32]) -> Result<(), Error> {
        self.inner
            .allowlist_set_pubkey(key)
            .await
            .map_err(From::from)
    }

    /// Set a public key
    ///
    /// # Errors
    ///
    /// This function will fail if the device refuses the operation, the app is
    /// not the special Ledgeracio app, or an I/O error occurs.
    pub async fn allowlist_upload(&self, allowlist: &[u8]) -> Result<(), Error> {
        self.inner
            .allowlist_upload(allowlist)
            .await
            .map_err(From::from)
    }

    /// Get the public key
    ///
    /// # Errors
    ///
    /// This function will fail if an I/O error occurs.
    pub async fn get_pubkey(&self) -> Result<[u8; 32], Error> {
        self.inner.allowlist_get_pubkey().await.map_err(From::from)
    }
}

impl HardSigner {
    /// Sign a message using the Ledger device.
    ///
    /// # Errors
    ///
    /// This function will fail if the app refuses the operation or an I/O error
    /// occurs.
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
        let call = extrinsic.deconstruct();
        let call_bytes = call.encode();
        let (call, extra, _) = call;
        let signature = match app.sign(path.as_ref(), &*call_bytes).await {
            Ok(e) => e,
            Err(e) => return Err(e.to_string()),
        };
        let signature = match Decode::decode(&mut &signature[..]) {
            Ok(e) => e,
            Err(e) => return Err(e.to_string()),
        };
        Ok(UncheckedExtrinsic::new_signed(
            call,
            self.address.clone(),
            signature,
            extra,
        ))
    }
}

type T = substrate_subxt::KusamaRuntime;

impl substrate_subxt::Signer<substrate_subxt::KusamaRuntime> for HardSigner {
    fn account_id(&self) -> &AccountId { &self.address }

    fn nonce(&self) -> Option<<T as System>::Index> { None }

    fn sign(
        &self,
        extrinsic: SignedPayload<Encoded, <<T as Runtime>::Extra as SignedExtra<T>>::Extra>,
    ) -> Signed<T> {
        let tmp = self.clone();
        Box::pin(async move { tmp.sign::<T>(extrinsic).await })
    }
}
