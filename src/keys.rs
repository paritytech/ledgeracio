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
use std::{future::Future, pin::Pin};

use substrate_subxt::{sp_core::crypto::AccountId32 as AccountId,
                      sp_runtime::{generic::{SignedPayload, UncheckedExtrinsic},
                                   traits::SignedExtension,
                                   MultiSignature},
                      system::System,
                      Encoded, Runtime, SignedExtra};

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
            + Sync
            + 'static,
    >,
>;

/// A keystore, backed by software or hardware.
pub(crate) enum KeyStore {
    #[cfg(feature = "insecure_software_keystore")]
    /// An insecure software keystore. For development purposes only.
    Soft(super::softstore::SoftKeyStore),
    /// A secure hardware keystore.
    Hard(super::hardstore::HardStore),
}

impl KeyStore {
    /// Get a signer for `path`
    pub(crate) async fn signer(&self, path: LedgeracioPath) -> Result<Signer, Error> {
        match self {
            #[cfg(feature = "insecure_software_keystore")]
            KeyStore::Soft(e) => Ok(Signer::Soft(e.signer(path).await?)),
            KeyStore::Hard(e) => Ok(Signer::Hard(e.signer(path).await?)),
        }
    }
}

/// A signer, backed by software or hardware.
#[derive(Clone)]
pub(crate) enum Signer {
    #[cfg(feature = "insecure_software_keystore")]
    /// An insecure software signer. For development purposes only.
    Soft(super::softstore::SoftSigner),
    /// A secure hardware signer.
    Hard(super::hardstore::HardSigner),
}
impl Signer {
    pub fn account_id(&self) -> &AccountId {
        match self {
            #[cfg(feature = "insecure_software_keystore")]
            Signer::Soft(e) => e.account_id(),
            Signer::Hard(e) => e.account_id(),
        }
    }
}

impl<
        T: Runtime<Signature = MultiSignature>
            + System<AccountId = AccountId, Address = AccountId>
            + Send
            + Sync
            + Runtime
            + 'static,
    > substrate_subxt::Signer<T> for Signer
where
    <<<T as Runtime>::Extra as SignedExtra<T>>::Extra as SignedExtension>::AdditionalSigned:
        Send + Sync + 'static,
{
    fn account_id(&self) -> &AccountId {
        match self {
            #[cfg(feature = "insecure_software_keystore")]
            Signer::Soft(e) => e.account_id(),
            Signer::Hard(e) => e.account_id(),
        }
    }

    fn nonce(&self) -> Option<T::Index> { None }

    fn sign(
        &self,
        extrinsic: SignedPayload<Encoded, <T::Extra as SignedExtra<T>>::Extra>,
    ) -> Signed<T> {
        let s: Self = (*self).clone();
        match s {
            #[cfg(feature = "insecure_software_keystore")]
            Signer::Soft(e) => Box::pin(async move { e.sign::<T>(extrinsic).await }),
            Signer::Hard(e) => Box::pin(async move { e.sign::<T>(extrinsic).await }),
        }
    }
}
