// Copyright 2020 Parity Technologies (UK) Ltd.
// This file is part of ledgeracio.
//
// ledgeracio is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free Software
// Foundation, either version 3 of the License, or (at your option) any later
// version.
//
// ledgeracio is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with ledgeracio.  If not, see <http://www.gnu.org/licenses/>.

//! Routines for parsing public and secret keys

use super::Error;
use core::convert::TryInto;
use ed25519_dalek::{ExpandedSecretKey, Keypair, PublicKey};
use substrate_subxt::sp_core::crypto::Ss58AddressFormat;

pub(crate) const MAGIC: &[u8] = &*b"Ledgeracio Secret Key";
pub(crate) fn parse_secret(secret: &[u8], network: Ss58AddressFormat) -> Result<Keypair, Error> {
    if secret.len() != 88 {
        return Err(format!("Ledgeracio secret keys are 88 bytes, not {}", secret.len()).into())
    }
    if &secret[..21] != MAGIC {
        return Err("Not a Ledgeracio secret key ― wrong magic number"
            .to_owned()
            .into())
    }
    if secret[21..23] != [1_u8, 0][..] {
        return Err(format!(
            "Expected a version 1 secret key, but got version {}",
            u16::from_le_bytes(secret[21..23].try_into().unwrap())
        )
        .into())
    }
    if secret[23] != u8::from(network) {
        return Err(format!(
            "Expected a key for network {}, but got a key for network {}",
            network,
            secret[23]
                .try_into()
                .unwrap_or_else(|()| Ss58AddressFormat::Custom(secret[23]))
        )
        .into())
    }

    let keypair = Keypair::from_bytes(&secret[24..88])?;
    let secret_expanded = ExpandedSecretKey::from(&keypair.secret);
    if PublicKey::from(&secret_expanded) != PublicKey::from_bytes(&secret[56..88])? {
        return Err("Public and secret keys don’t match".to_owned().into())
    }
    Ok(keypair)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[should_panic(expected = "Ledgeracio secret keys are 88 bytes, not 87")]
    fn too_short_rejected() { parse_secret(&[0; 87][..], Default::default()).unwrap(); }
    #[test]
    #[should_panic(expected = "Ledgeracio secret keys are 88 bytes, not 89")]
    fn too_long_rejected() { parse_secret(&[0; 89][..], Default::default()).unwrap(); }
}
