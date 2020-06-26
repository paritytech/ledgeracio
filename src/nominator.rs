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
use substrate_subxt::{balances::Balances,
                      sp_core::crypto::{AccountId32 as AccountId, Ss58AddressFormat},
                      sp_runtime::{traits::SignedExtension, MultiSignature},
                      staking::{LedgerStore, NominateCallExt, RewardDestination, SetPayeeCallExt,
                                Staking},
                      system::System,
                      Client, Runtime, SignedExtra};

#[derive(StructOpt, Debug)]
pub(crate) enum Nominator {
    /// Show the specified stash controller
    Show { index: u32 },
    /// Show the status of all stash controllers
    Status,
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

pub(crate) async fn main<
    T: System<AccountId = ::substrate_subxt::sp_core::crypto::AccountId32, Address = AccountId>
        + substrate_subxt::Runtime<Signature = MultiSignature>
        + Balances
        + Send
        + Sync
        + Staking
        + std::fmt::Debug
        + 'static,
>(
    cmd: Nominator,
    client: Client<T>,
    network: Ss58AddressFormat,
    keystore: &crate::keys::KeyStore,
) -> Result<T::Hash, Error>
where
    <<<T as Runtime>::Extra as SignedExtra<T>>::Extra as SignedExtension>::AdditionalSigned:
        Send + Sync,
    <T as Balances>::Balance: std::fmt::Display,
{
    use std::convert::{TryFrom, TryInto};
    match cmd {
        Nominator::Status => unimplemented!("showing validator status"),
        Nominator::Show { index } => {
            let path = LedgeracioPath::new(network, AccountType::Nominator, index)?;
            let signer = keystore.signer(path).await?;
            let controller: &AccountId = ::substrate_subxt::Signer::<T>::account_id(&signer);
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
                    return Err(
                        format!("No nominator account found for controller {}", controller).into(),
                    )
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
