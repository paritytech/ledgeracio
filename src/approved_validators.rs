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

//! Routines for handling approved validators

use super::{Error, StructOpt};
use std::{os::unix::ffi::OsStrExt, path::PathBuf};

#[derive(StructOpt, Debug)]
pub(crate) enum ACL {
    /// Upload a new approved validator list.  This list must be signed.
    Upload { path: PathBuf },
    /// Set the validator list signing key.  This will fail if a signing key has
    /// already been set.
    SetKey {
        #[structopt(parse(try_from_str = hex::FromHex::from_hex))]
        key: [u8; 32],
    },
    /// Get the validator list signing key.  This will fail unless a signing key
    /// has been set.
    GetKey,
    /// Generate a new signing key.
    GenKey {
        #[structopt(short = "p", long = "public")]
        public: PathBuf,
        #[structopt(short = "s", long = "secret")]
        secret: PathBuf,
    },
}

fn write(buf: &[u8], path: &std::path::Path) -> std::io::Result<()> {
    use std::{fs::OpenOptions, io::Write, os::unix::fs::OpenOptionsExt};
    OpenOptions::new()
        .mode(0o600)
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?
        .write_all(buf)
}

pub(crate) async fn main<T: FnOnce() -> Result<super::HardStore, Error>>(
    acl: ACL,
    hardware: T,
) -> Result<(), Error> {
    use ed25519_dalek::{ExpandedSecretKey, Keypair, PublicKey};

    match acl {
        ACL::GetKey => {
            let s = hardware()?.get_pubkey().await?;
            println!("Public key is {}", hex::encode(s));
            Ok(())
        }
        ACL::SetKey { key } => hardware()?.set_pubkey(&key).await,
        ACL::Upload { path } => {
            let allowlist = std::fs::read(path)?;
            hardware()?.allowlist_upload(&allowlist).await
        }
        ACL::GenKey { public, secret } => {
            let pub_bytes = public.as_os_str().as_bytes();
            let sec_bytes = secret.as_os_str().as_bytes();
            let len = pub_bytes.len();
            if !pub_bytes.ends_with(b".pub")
                || !sec_bytes.ends_with(b".sec")
                || len != sec_bytes.len()
                || pub_bytes[..len - 4] != sec_bytes[..len - 4]
            {
                return Err(
                    "Public and secret key filenames must match, except that the public key file \
                     must have extension .pub and secret key file must have extension .sec"
                        .to_owned()
                        .into(),
                )
            }
            let keypair = Keypair::generate(&mut rand::rngs::OsRng {});
            let secretkey = keypair.secret.to_bytes();
            let publickey = keypair.public.to_bytes();
            write(&publickey, &public)?;
            write(&secretkey, &secret)?;
            Ok(())
        }
    }
}
