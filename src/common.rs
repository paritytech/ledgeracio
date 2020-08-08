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
                      system::AccountStoreExt,
                      Client, KusamaRuntime, Signer};

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
    let mut index = 0u32;
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

pub(crate) enum AddressSource<'a> {
    Device(Option<u32>, &'a crate::HardStore),
}

pub(crate) async fn display_validators(
    client: &Client<KusamaRuntime>,
    nominations: &[AccountId],
    network: Ss58AddressFormat,
) -> Result<(), Error> {
    use substrate_subxt::staking::{LedgerStore, StakingLedger, ValidatorsStore};
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
                claimed_rewards,
            }) => {
                println!(
                    "    Validator account: {}\n    Stash balance: {}\n    Amount at stake: \
                     {}\n    Amount unlocking: {:?}\n    Rewards claimed: {:?}",
                    stash.to_ss58check_with_version(network), total, active, unlocking, claimed_rewards
                );
                let store = ValidatorsStore {
                    stash: stash.clone(),
                };
                match client.fetch(&store, None).await? {
                    None => println!(
                        "    validator {} has no preferences (this is a bug)\n",
                        stash.to_ss58check_with_version(network)
                    ),
                    Some(prefs) => println!("    Prefs: {:?}\n", prefs),
                }
            }
        }
    }
    Ok(())
}
