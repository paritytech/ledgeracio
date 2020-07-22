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

//! Routines for handling approved validators

use super::{hardstore::HardStore, parse_reward_destination, AccountType, Error, LedgeracioPath,
            StructOpt};
use codec::Decode;
use std::future::Future;

#[derive(StructOpt, Debug)]
pub(crate) enum ACL {
    /// Upload a new approved validator list.  This list must be signed.
    Upload { path: std::path::PathBuf },
    /// Set the validator list signing key.  This will fail if a signing key has
    /// already been set.
    SetKey {
        #[structopt(parse(try_from_str = hex::FromHex::from_hex))]
        key: [u8; 32],
    },
    /// Get the validator list signing key.  This will fail unless a signing key
    /// has been set.
    GetKey,
}

pub(crate) async fn main(acl: ACL, hardware: HardStore) -> Result<(), Error> {
    match acl {
        ACL::GetKey => {
            let s = hardware.get_pubkey().await?;
            println!("Public key is {}", hex::encode(s));
            Ok(())
        }
        ACL::SetKey { key } => hardware.set_pubkey(&key).await,
        ACL::Upload { .. } => unimplemented!(),
    }
}
