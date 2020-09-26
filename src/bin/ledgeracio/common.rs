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

//! Utilities shared by both validator and nominator code

use super::{AccountId, AccountType, Error, LedgeracioPath};
use substrate_subxt::{sp_core::crypto::{Ss58AddressFormat, Ss58Codec},
                      staking::{LedgerStore, StakingLedger, ValidatorsStore},
                      system::AccountStoreExt,
                      Client, KusamaRuntime, Signer, SystemProperties};

pub(crate) async fn fetch_validators(
    client: &Client<KusamaRuntime>,
    source: AddressSource<'_>,
    network: Ss58AddressFormat,
    account_type: AccountType,
) -> Result<Vec<AccountId>, Error> {
    let (index, keystore) = match source {
        AddressSource::Device(index, signer) => (index, signer),
    };
    let mut v = vec![];
    if let Some(index) = index {
        let path = LedgeracioPath::new(network, account_type, index)?;
        let signer = keystore.signer(path).await?;
        return Ok(vec![signer.account_id().clone()])
    }
    let mut index = 0_u32;
    loop {
        index += 1;
        assert!(index > 0);
        let path = LedgeracioPath::new(network, account_type, index)?;
        let signer = keystore.signer(path).await?;
        let account_id = signer.account_id();
        if client.account(account_id, None).await?.data.free == 0 {
            return Ok(v)
        }
        v.push(account_id.clone())
    }
}

pub enum AddressSource<'a> {
    Device(Option<u32>, &'a crate::HardStore),
}

pub(crate) async fn display_validators(
    client: &Client<KusamaRuntime>,
    nominations: &[AccountId],
    network: Ss58AddressFormat,
) -> Result<(), Error> {
    for controller in nominations {
        let store = LedgerStore {
            controller: controller.clone(),
        };
        match client.fetch(&store, None).await? {
            None => println!(
                "validator {} not found",
                controller.to_ss58check_with_version(network)
            ),
            Some(StakingLedger {
                stash,
                total,
                active,
                unlocking,
                claimed_rewards: _,
            }) => {
                let SystemProperties {
                    token_decimals,
                    mut token_symbol,
                    ..
                } = client.properties().clone();
                let mut good_symbol = true;
                for i in token_symbol.bytes() {
                    good_symbol &= i.is_ascii_uppercase()
                }
                if !good_symbol {
                    token_symbol = "".to_owned()
                }
                println!(
                    "    Validator account: {}\n    Stash balance: {} {sym}\n    Amount at stake: \
                     {} {sym}\nEras with unclaimed payouts: {:?}\n    Amount unlocking: {:?}",
                    stash.to_ss58check_with_version(network),
                    pad(token_decimals, total),
                    pad(token_decimals, active),
                    super::payouts::display_payouts(controller.clone(), client).await?,
                    unlocking,
                    sym = token_symbol
                );
                let store = ValidatorsStore {
                    stash: stash.clone(),
                };
                match client.fetch(&store, None).await? {
                    None => println!(
                        "    validator {} has no preferences ― it is probably inactive\n",
                        stash.to_ss58check_with_version(network)
                    ),
                    Some(prefs) => println!(
                        "    Commission: {}%\n",
                        pad(9, u128::from(prefs.commission.deconstruct()) * 100)
                    ),
                }
            }
        }
    }
    Ok(())
}

pub fn pad(mut zeros: u8, value: u128) -> String {
    if value == 0 {
        return "0".to_owned()
    }
    let mut value = value.to_string();
    let len = value.len();
    assert_ne!(len, 0, "stringified numbers are never empty");
    if len <= zeros.into() {
        let mut buf = "0.".to_owned();
        while len < zeros.into() {
            buf.push('0');
            zeros -= 1;
        }
        value = buf + &*value
    } else {
        value.insert(len - usize::from(zeros), '.');
    }
    while value.ends_with('0') {
        value.pop();
    }
    if value.ends_with('.') {
        value.pop();
    }
    value
}

pub fn parse_ppb(data: &str) -> Result<u32, Error> {
    if data.len() > 11 {
        Err("Commission too long.  Check for excess trailing zeroes."
            .to_owned()
            .into())
    } else if data == "1" {
        Ok(1_000_000_000)
    } else if data.starts_with("1.") {
        for i in data[2..].bytes() {
            if i != b'0' {
                return Err("Commission cannot exceed 1".to_owned().into())
            }
        }
        Ok(1_000_000_000)
    } else if data == "0" {
        Ok(0)
    } else if data.starts_with("0.") {
        let mut len = data.len() - 2;
        let mut res: u32 = str::parse(&data[2..])?;
        while len < 9 {
            res *= 10;
            len += 1;
        }
        Ok(res)
    } else {
        Err(
            "Commission must start with 0. or be equal to 1 (the default) or 0"
                .to_owned()
                .into(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn padding_works() {
        assert_eq!(pad(0, 100), "100".to_owned());
        assert_eq!(pad(3, 100), "0.1".to_owned());
        assert_eq!(pad(3, 10000), "10".to_owned());
        assert_eq!(pad(3, 10001), "10.001".to_owned());
        assert_eq!(pad(3, 10010), "10.01".to_owned());
    }

    #[test]
    fn parse_ppb_works() {
        assert_eq!(parse_ppb("1.000000000").unwrap(), 1_000_000_000);
        assert!(parse_ppb("1.0000000000").is_err());
        assert_eq!(parse_ppb("1").unwrap(), 1_000_000_000);
        assert_eq!(parse_ppb("0").unwrap(), 0);
        assert!(parse_ppb("0.").is_err());
        assert_eq!(parse_ppb("0.01").unwrap(), 10_000_000);
        assert!(parse_ppb("0.0100000000").is_err());
        assert_eq!(parse_ppb("0.010000000").unwrap(), 10_000_000);
        assert_eq!(parse_ppb("0.000000000").unwrap(), 0);
        assert_eq!(parse_ppb("0.999999999").unwrap(), 999_999_999);
    }
}
