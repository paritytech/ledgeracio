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
use substrate_subxt::{sp_core::crypto::Ss58AddressFormat, system::AccountStoreExt, Client,
                      KusamaRuntime, Signer};

pub(crate) async fn fetch_validators(
    client: &Client<KusamaRuntime>,
    network: Ss58AddressFormat,
    account_type: AccountType,
    keystore: &super::HardStore,
    index: Option<u32>,
) -> Result<Vec<AccountId>, Error> {
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

pub(crate) async fn display_validators(
    client: &Client<KusamaRuntime>,
    nominations: &[AccountId],
) -> Result<(), Error> {
    use substrate_subxt::staking::{LedgerStore, StakingLedger, ValidatorsStore};
    for controller in nominations {
        let store = LedgerStore {
            controller: controller.clone(),
        };
        match client.fetch(store, None).await? {
            None => println!("validator {} not found", controller),
            Some(StakingLedger {
                stash,
                total,
                active,
                unlocking,
                claimed_rewards,
            }) => {
                println!(
                    "    Validator account: {}\n    Stash balance: {}\n    Amount at stake: \
                     {}\nAmount unlocking: {:?}\nRewards claimed: {:?}\n",
                    stash, total, active, unlocking, claimed_rewards
                );
                let store = ValidatorsStore {
                    stash: stash.clone(),
                };
                match client.fetch(store, None).await? {
                    None => println!("validator {} has no preferences (this is a bug)", stash),
                    Some(prefs) => println!("    Prefs: {:?}\n", prefs),
                }
            }
        }
    }
    Ok(())
}
