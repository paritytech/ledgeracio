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

//! Ledgeracio allowlist parser

use std::io::{prelude::*, BufReader};
use substrate_subxt::sp_core::crypto::{Ss58Codec, Ss58AddressFormat, AccountId32 as AccountId};

fn parse<T: BufRead, U: Ss58Codec>(reader: T) -> std::io::Result<Vec<(U, u8)>> {
    let mut v = vec![];
    for (l, i) in reader.lines().enumerate() {
        let i = i?;
        let trimmed = i.trim_start().trim_end();
        if trimmed.starts_with(';') || trimmed.starts_with('#') || trimmed.is_empty() {
            continue
        }
        v.push(crate::parse_address(trimmed).map_err(|i| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("parse error on line {}: {}", l, i),
            )
        })?)
    }
    Ok(v)
}

#[cfg(test)]
mod test {
	use super::*;
	#[test]
	fn parse_test() {
		assert_eq!(parse::<_, AccountId>(BufReader::new(&mut &br#"
# this is a comment
  ; this is also a comment
"#[..])).unwrap(), vec![])
	}
}
