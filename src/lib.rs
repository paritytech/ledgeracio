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

//! Ledgeracio utility library.  Do not depend on this.

#![deny(clippy::all, clippy::pedantic)]
#![forbid(unsafe_code)]
mod derivation;
mod hardstore;

use codec::Encode;
pub use derivation::{AccountType, LedgeracioPath};
pub use hardstore::{HardSigner, HardStore};

#[cfg(not(unix))]
compile_error!("Only *nix-like platforms are supported");

pub use sp_core::crypto::AccountId32 as AccountId;
pub use std::{convert::{TryFrom, TryInto},
              fmt::Debug,
              future::Future,
              pin::Pin};
pub use structopt::StructOpt;
pub use substrate_subxt::{sp_core,
                          sp_core::crypto::{Ss58AddressFormat, Ss58Codec},
                          staking::RewardDestination,
                          Client, ClientBuilder, Signer};

pub type Error = Box<dyn std::error::Error + Send + Sync>;

// pub type Runtime = substrate_subxt::KusamaRuntime;

/// Parse an SS58 address
///
/// # Errors
///
/// Fails if the address is malformed.
pub fn parse_address<T: Ss58Codec>(arg: &str) -> Result<(T, u8), String> {
    Ss58Codec::from_string_with_version(arg)
        .map_err(|e| format!("{:?}", e))
        .map(|(x, y)| (x, y.into()))
}

/// Validate that the address `address`, which parsed to network `provided_network`, is valid for
/// network `network`.
///
/// # Errors
///
/// Fails if the address was for the wrong network.
pub fn validate_network(
    address: &str,
    provided_network: u8,
    network: Ss58AddressFormat,
) -> Result<(), Error> {
    if network == provided_network.try_into().unwrap() {
        Ok(())
    } else {
        Err(format!(
            "Network mismatch: address {} is for network {}, but you asked to use network {}",
            address,
            String::from(Ss58AddressFormat::try_from(provided_network).unwrap()),
            String::from(network),
        )
        .into())
    }
}

/// Converts a network name into an address format.
///
/// # Errors
///
/// Fails if `Ss58AddressFormat::try_from` fails.
pub fn get_network(address: &str) -> Result<Ss58AddressFormat, Error> {
    Ss58AddressFormat::try_from(address).map_err(|_| format!("Unknown network {}", address).into())
}
