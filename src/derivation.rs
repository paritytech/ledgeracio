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

//! Polkadot and Kusama derivation paths

use substrate_subxt::sp_core::crypto::Ss58AddressFormat;
use zx_bip44::BIP44Path;

/// A derivation path that can be used with Ledgeracio
#[derive(Debug)]
pub struct LedgeracioPath(BIP44Path);

/// A type of account: stash or validator
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum AccountType {
    /// Stash account
    Stash,
    /// Validator account
    Validator,
}

impl std::str::FromStr for AccountType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "stash" => Ok(Self::Stash),
            "validator" => Ok(Self::Validator),
            _ => Err("Account type must be `stash` or `validator`"),
        }
    }
}

/// Errors that can occur when creating a derivation path
#[derive(::thiserror::Error, Debug)]
pub enum DerivationPathError {
    /// Unsupported network (not Polkadot or Kusama)
    #[error("Unsupported network {0:?}")]
    UnsupportedNetwork(DebuggableFormat),
    /// Index out of range (greater than `1u32 << 31`)
    #[error("Index too large (greater than 2**31): {0}")]
    IndexTooLarge(u32),
}

pub struct DebuggableFormat(Ss58AddressFormat);
impl std::fmt::Debug for DebuggableFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(&*String::from(self.0))
    }
}

/// The MSB of indexes for hardened derivation paths
pub const HARDENED: u32 = 1 << 31;

/// The [SLIP-0044] code for Polkadot
///
/// [SLIP-O044]: https://github.com/satoshilabs/slips/blob/master/slip-0044.md
pub const POLKADOT: u32 = 0x80000162;

/// The [SLIP-0044] code for Kusama
///
/// [SLIP-O044]: https://github.com/satoshilabs/slips/blob/master/slip-0044.md
pub const KUSAMA: u32 = 0x800001b2;

impl LedgeracioPath {
    /// Create a new Ledgeracio derivation path, or return an error if the path
    /// is not valid.
    pub fn new(
        network: Ss58AddressFormat,
        account_type: AccountType,
        account_index: u32,
    ) -> Result<Self, DerivationPathError> {
		assert_eq!(HARDENED, 0x80000000);
        let slip_0044_code = match network {
            Ss58AddressFormat::PolkadotAccount => POLKADOT,
            Ss58AddressFormat::KusamaAccount => KUSAMA,
            bad_network => {
                return Err(DerivationPathError::UnsupportedNetwork(DebuggableFormat(
                    bad_network,
                )))
            }
        };
        if account_index > HARDENED {
            return Err(DerivationPathError::IndexTooLarge(account_index))
        }
        Ok(Self(BIP44Path([
            HARDENED | 44,
            slip_0044_code,
            account_type as u32,
            HARDENED,
            HARDENED | account_index,
        ])))
    }

    /// Returns the account type of this derivation path.
    pub fn account_type(&self) -> AccountType {
        match (self.0).0[2] {
            0 => AccountType::Stash,
            1 => AccountType::Validator,
            _ => unreachable!("account type was checked in Self::new()"),
        }
    }

    /// Returns the index of this derivation path.
    pub fn index(&self) -> u32 { (self.0).0[4] }
}

impl Clone for LedgeracioPath {
    fn clone(&self) -> Self { Self(BIP44Path((self.0).0)) }
}

impl AsRef<[u32]> for LedgeracioPath {
    fn as_ref(&self) -> &[u32] { &(self.0).0 }
}

impl AsRef<zx_bip44::BIP44Path> for LedgeracioPath {
    fn as_ref(&self) -> &zx_bip44::BIP44Path { &self.0 }
}
