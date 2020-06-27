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

//! Nominator commands

use super::{parse_address, parse_reward_destination, AccountType, Error, LedgeracioPath, StructOpt};
use substrate_subxt::{sp_core::{crypto::{AccountId32 as AccountId, Ss58AddressFormat},
                                H256},
                      staking::{LedgerStore, NominateCallExt, RewardDestination, SetPayeeCallExt},
                      Client, KusamaRuntime};

#[derive(StructOpt, Debug)]
pub(crate) enum Nominator {
    /// Show the specified stash controller, or all if none is specified.
    Show { index: Option<u32> },
    /// Claim a validation payout
    Claim { index: Option<u32> },
    /// Nominate a new validator set
    #[structopt(name = "nominate")]
    Nominate {
        index: u32,
        #[structopt(parse(try_from_str = parse_address))]
        set: Vec<(AccountId, u8)>,
    },
    /// Set payment target
    #[structopt(name = "set-payee")]
    SetPayee {
        index: u32,
        #[structopt(parse(try_from_str = parse_reward_destination))]
        target: RewardDestination,
    },
}

pub(crate) async fn main(
    cmd: Nominator,
    client: Client<KusamaRuntime>,
    network: Ss58AddressFormat,
    keystore: &crate::keys::KeyStore,
) -> Result<H256, Error> {
    use std::convert::{TryFrom, TryInto};
    match cmd {
        Nominator::Show { index } => {
            let nominators = crate::common::fetch_validators(
                &client,
                network,
                AccountType::Nominator,
                keystore,
                index,
            )
            .await?;
            for controller in nominators {
                match client
                    .fetch(
                        LedgerStore {
                            controller: controller.clone(),
                        },
                        None,
                    )
                    .await?
                {
                    Some(stash) => println!(
                        "Nominator account: {}\nStash balance: {}\nAmount at stake: {}",
                        stash.stash, stash.total, stash.active
                    ),
                    None => {
                        return Err(format!(
                            "No nominator account found for controller {}",
                            controller
                        )
                        .into())
                    }
                }
            }
            Ok(Default::default())
        }

        Nominator::Claim { index } => unimplemented!("claiming payment for {:?}", index),
        Nominator::Nominate { index, set } => {
            let path = LedgeracioPath::new(network, AccountType::Nominator, index)?;
            let signer = keystore.signer(path).await?;
            if set.is_empty() {
                return Err("Validator set cannot be empty".to_owned().into())
            }
            let mut new_set = vec![];
            for (address, provided_network) in set.into_iter() {
                if network != provided_network.try_into().unwrap() {
                    return Err(format!(
                        "Network mismatch: address {} is for network {}, but you asked to use \
                         network {}",
                        address,
                        String::from(Ss58AddressFormat::try_from(provided_network).unwrap()),
                        String::from(Ss58AddressFormat::try_from(network).unwrap()),
                    )
                    .into())
                }
                new_set.push(address)
            }
            Ok(client.nominate(&signer, new_set).await?)
        }
        Nominator::SetPayee { index, target } => {
            let path = LedgeracioPath::new(network, AccountType::Nominator, index)?;
            let signer = keystore.signer(path).await?;
            Ok(client.set_payee(&signer, target).await?)
        }
    }
}
