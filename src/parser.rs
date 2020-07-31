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

//! Ledgeracio allowlist parser/signer

use ed25519_dalek::{ExpandedSecretKey, PublicKey};
use std::{convert::TryFrom,
          io::{prelude::*, BufReader, Error, ErrorKind}};
use substrate_subxt::sp_core::crypto::{AccountId32 as AccountId, Ss58AddressFormat, Ss58Codec};

fn parse<T: BufRead, U: Ss58Codec>(
    reader: T,
    network: Ss58AddressFormat,
    sk: Option<&ExpandedSecretKey>,
) -> std::io::Result<Vec<u8>> {
    let mut v = vec![0; 68];
    for (l, i) in reader.lines().enumerate() {
        let i = i?;
        let trimmed = i.trim_start().trim_end();
        if trimmed.starts_with(';') || trimmed.starts_with('#') || trimmed.is_empty() {
            continue
        }
        let (_address, address_type): (AccountId, _) =
            crate::parse_address(trimmed).map_err(|i| {
                Error::new(
                    ErrorKind::InvalidData,
                    format!("parse error on line {}: {}", l, i),
                )
            })?;
        let () = crate::validate_network(trimmed, address_type, network).map_err(|i| {
            Error::new(
                ErrorKind::InvalidData,
                format!("invalid network on line {}: {}", l, i),
            )
        })?;
        let bytes = trimmed.as_bytes();
        let current_len = v.len();
        v.extend_from_slice(&[0u8; 64]);
        v[current_len..current_len + bytes.len()].copy_from_slice(bytes);
    }
    let total_len_bytes = ((v.len() - 68) >> 6).to_le_bytes();
    v[..4].copy_from_slice(&total_len_bytes);
    if let Some(sk) = sk {
        let digest = blake2b_simd::Params::new()
            .hash_length(32)
            .to_state()
            .update(&total_len_bytes)
            .update(&v[68..])
            .finalize();
        let pk = PublicKey::from(sk);
        let signature = sk.sign(&digest.as_bytes(), &pk);
        v[4..68].copy_from_slice(&signature.to_bytes()[..]);
    }
    Ok(v)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn parse_test() {
        assert_eq!(
            parse::<_, AccountId>(BufReader::new(
                &mut &br#"
# this is a comment
  ; this is also a comment
"#[..]
            ))
            .unwrap(),
            vec![]
        )
    }
}
