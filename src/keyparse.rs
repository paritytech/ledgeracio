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
use ed25519_dalek::{ExpandedSecretKey, Keypair, PublicKey};
use regex::bytes::Regex;
use std::{convert::{TryFrom, TryInto},
          str};
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

pub(crate) fn parse_public(unparsed: &[u8]) -> Result<(PublicKey, Ss58AddressFormat), Error> {
    let re = Regex::new(r"^Ledgeracio version ([1-9][0-9]*) public key for network ([[:alpha:]]+)\n([[:alnum:]/+]+=)\n$").unwrap();
    let captures = re
        .captures(&unparsed)
        .ok_or_else(|| "Invalid public key".to_owned())?;
    let (version, network, data) = (
        str::from_utf8(&captures[1]).unwrap(),
        str::from_utf8(&captures[2]).unwrap(),
        str::from_utf8(&captures[3]).unwrap(),
    );
    if version != "1" {
        return Err("Only version 1 keys are supported".to_owned().into())
    }
    if data.len() != 44 {
        return Err("base64-encoded ed25519 public keys are 44 bytes"
            .to_owned()
            .into())
    }
    let network = Ss58AddressFormat::try_from(&*network.to_ascii_lowercase())
        .map_err(|()| format!("invalid network {}", network))?;
    let mut pk = [0_u8; 32];
    assert_eq!(
        base64::decode_config_slice(&*data, base64::STANDARD, &mut pk)?,
        pk.len()
    );
    let pk = ed25519_dalek::PublicKey::from_bytes(&pk[..])?;
    Ok((pk, network))
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
    const BAD_KEY: &[u8] = &[
        0x4c, 0x65, 0x64, 0x67, 0x65, 0x72, 0x61, 0x63, 0x69, 0x6f, 0x20, 0x53, 0x65, 0x63, 0x72,
        0x65, 0x74, 0x20, 0x4b, 0x65, 0x79, 0x01, 0x00, 0x00, 0xbf, 0x5b, 0x4a, 0x93, 0x49, 0xfe,
        0x8b, 0x41, 0xdd, 0x62, 0x6a, 0x62, 0xb2, 0x5e, 0xca, 0xc5, 0x08, 0x90, 0x1b, 0x1b, 0x3f,
        0x43, 0x4c, 0x7f, 0x12, 0x81, 0x79, 0x12, 0x5c, 0xdc, 0x52, 0xf0, 0xb4, 0xea, 0xb4, 0x80,
        0x03, 0xcf, 0x12, 0xc6, 0x62, 0xa2, 0xe9, 0x78, 0x60, 0x05, 0x6d, 0xd3, 0x9f, 0x57, 0x2f,
        0x39, 0x0b, 0xd0, 0x60, 0x4a, 0x12, 0xd2, 0x9f, 0xca, 0xe9, 0x77, 0x23, 0xd4,
    ];

    #[test]
    #[should_panic(expected = "Public and secret keys don’t match")]
    fn rejects_key_mismatch() {
        parse_secret(BAD_KEY, Ss58AddressFormat::PolkadotAccount).unwrap();
    }

    const GOOD_KEY: &[u8] = &[
        0x4c, 0x65, 0x64, 0x67, 0x65, 0x72, 0x61, 0x63, 0x69, 0x6f, 0x20, 0x53, 0x65, 0x63, 0x72,
        0x65, 0x74, 0x20, 0x4b, 0x65, 0x79, 0x01, 0x00, 0x00, 0xea, 0xcf, 0x97, 0x59, 0x4d, 0x16,
        0x32, 0x6e, 0x18, 0x63, 0x61, 0xd7, 0xee, 0x50, 0xb8, 0xde, 0x51, 0x5b, 0x7a, 0xe3, 0x1b,
        0x5e, 0xbb, 0xc4, 0xbb, 0x00, 0xc0, 0xb1, 0x0c, 0xf2, 0xad, 0xaa, 0x15, 0x0e, 0x21, 0xcf,
        0xe5, 0x96, 0x2e, 0x93, 0x37, 0xe2, 0x0f, 0xfc, 0x9c, 0x93, 0xad, 0x62, 0x05, 0xd5, 0xb1,
        0x5a, 0x67, 0x1c, 0x05, 0xc7, 0x05, 0x8b, 0xfd, 0xee, 0xcc, 0x4c, 0x59, 0xeb,
    ];

    #[test]
    fn accepts_good_key() { parse_secret(GOOD_KEY, Ss58AddressFormat::PolkadotAccount).unwrap(); }

    #[test]
    #[should_panic(
        expected = "Expected a key for network KusamaAccount, but got a key for network \
                    PolkadotAccount"
    )]
    fn rejects_wrong_network() {
        parse_secret(GOOD_KEY, Ss58AddressFormat::KusamaAccount).unwrap();
    }
}
