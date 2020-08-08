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
use core::{future::Future, pin::Pin};
use substrate_subxt::{sp_core::crypto::{AccountId32 as AccountId, Ss58AddressFormat, Ss58Codec},
                      staking::{BondedStore, LedgerStore, NominateCallExt, PayeeStore,
                                RewardDestination, SetPayeeCallExt},
                      Client, KusamaRuntime};

#[derive(StructOpt, Debug)]
pub(crate) enum Nominator {
    /// Show the given address
    ShowAddress {
        #[structopt(parse(try_from_str = parse_address))]
        address: (AccountId, u8),
    },
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
    /// Display the address of the given index
    Address { index: u32 },
}

async fn display_nominators(
    controller: AccountId,
    client: &Client<KusamaRuntime>,
    network: Ss58AddressFormat,
) -> Result<(), Error> {
    use substrate_subxt::staking::{NominatorsStore, StakingLedger};
    let store = LedgerStore {
        controller: controller.clone(),
    };
    let StakingLedger {
        stash,
        total,
        active,
        unlocking,
        claimed_rewards: _, // not updated for nominators
    } = client
        .fetch(&store, None)
        .await?
        .ok_or_else(|| format!("No nominator account found for controller {}", controller))?;
    let payee = client
        .fetch(
            &PayeeStore {
                stash: stash.clone(),
            },
            None,
        )
        .await?
        .ok_or_else(|| {
            format!(
                "No payee found for controller {} (this is a bug)",
                controller
            )
        })?;
    println!(
        "Nominator account: {}\nStash balance: {}\nAmount at stake: {}\nAmount unlocking: \
         {:?}\nPayee: {:?}",
        stash.to_ss58check_with_version(network),
        total,
        active,
        unlocking,
        payee
    );
    let nominations = match client.fetch(&NominatorsStore { stash }, None).await? {
        None => {
            println!("Nominations: None (yet)");
            return Ok(())
        }
        Some(n) => n,
    };
    println!(
        "Era nominations submitted: {}\nNominations suppressed: {}\nTargets:\n",
        nominations.submitted_in, nominations.suppressed
    );
    for stash in nominations.targets.iter().cloned() {
        match client
            .fetch(
                &BondedStore {
                    stash: stash.clone(),
                },
                None,
            )
            .await?
        {
            Some(controller) => {
                crate::common::display_validators(client, &[controller], network).await?
            }
            None => println!(
                "controller not found for stash {}\n",
                stash.to_ss58check_with_version(network)
            ),
        }
    }
    Ok(())
}

pub(crate) async fn main<T: FnOnce() -> Result<super::HardStore, Error>>(
    cmd: Nominator,
    client: Pin<Box<dyn Future<Output = Result<Client<KusamaRuntime>, Error>>>>,
    network: Ss58AddressFormat,
    keystore: T,
) -> Result<(), Error> {
    use std::convert::{TryFrom, TryInto};
    match cmd {
        Nominator::ShowAddress {
            address: (stash, provided_network),
        } => {
            super::validate_network("", provided_network, network)?;
            let client = client.await?;
            let controller = match client.fetch(&BondedStore { stash }, None).await? {
                Some(controller) => controller,
                None => return Err("Controller not found for stash".to_owned().into()),
            };
            display_nominators(controller, &client, network).await?;
            Ok(())
        }
        Nominator::Show { index } => {
            let client = client.await?;
            let nominators = crate::common::fetch_validators(
                &client,
                crate::AddressSource::Device(index, &keystore()?),
                network,
                AccountType::Nominator,
            )
            .await?;
            for controller in nominators {
                display_nominators(controller, &client, network).await?
            }
            Ok(Default::default())
        }

        Nominator::Claim { index } => unimplemented!("claiming payment for {:?}", index),
        Nominator::Nominate { index, set } => {
            let path = LedgeracioPath::new(network, AccountType::Nominator, index)?;
            let signer = keystore()?.signer(path).await?;
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
            client.await?.nominate(&signer, new_set).await?;
            Ok(())
        }
        Nominator::SetPayee { index, target } => {
            let path = LedgeracioPath::new(network, AccountType::Nominator, index)?;
            let signer = keystore()?.signer(path).await?;
            client.await?.set_payee(&signer, target).await?;
            Ok(())
        }
        Nominator::Address { index } => {
            crate::display_path(AccountType::Nominator, &keystore()?, network, index).await?;
            Ok(Default::default())
        }
    }
}
