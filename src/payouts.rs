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

//! Payouts handling

use crate::Error;
use futures::{future::{join, join3},
              stream::{FuturesUnordered, StreamExt as _}};
use std::{marker::PhantomData};
use substrate_subxt::{sp_core::crypto::{AccountId32 as AccountId, Ss58AddressFormat},
                      sp_runtime::traits::Zero,
                      staking::{CurrentEraStore, ErasRewardPointsStore, ErasStakersClippedStore,
                                HistoryDepthStore, LedgerStore,
                                StakingLedger},
                      Client, KusamaRuntime};

pub(crate) async fn display_payouts(
    controller: AccountId,
    client: &Client<KusamaRuntime>,
    network: Ss58AddressFormat,
) -> Result<Vec<u32>, Error> {
    let store = LedgerStore {
        controller: controller.clone(),
    };
    let history_depth = client.fetch(
        &HistoryDepthStore {
            _runtime: PhantomData,
        },
        None,
    );
    let current_era = client.fetch(
        &CurrentEraStore {
            _runtime: PhantomData,
        },
        None,
    );
    let fetch_account_info = async {
        let StakingLedger {
            stash,
            claimed_rewards,
            ..
        } = client
            .fetch(&store, None)
            .await?
            .ok_or_else(|| format!("No nominator account found for controller {}", controller))?;
        let res: Result<_, Error> = Ok((stash, claimed_rewards));
        res
    };
    let (history_depth, account_info, current_era) =
        join3(history_depth, fetch_account_info, current_era).await;
    let history_depth = history_depth?.ok_or_else(|| "No history depth, sorry".to_owned())?;
    let (validator_stash, claimed_rewards): (AccountId, _) = account_info?;
    let current_era = current_era?.ok_or_else(|| "No current era, sorry".to_owned())?;
    let history_start = current_era.saturating_sub(history_depth);
    let mut futures = FuturesUnordered::new();
    for era in history_start..=current_era {
        if claimed_rewards.binary_search(&era).is_ok() {
            continue
        }
        let stakers = ErasStakersClippedStore {
            era,
            validator_stash: validator_stash.clone(),
        };
        let validator_stash = validator_stash.clone();
        let future = async move {
            let rewards = ErasRewardPointsStore {
                index: era,
                _phantom: PhantomData,
            };
            let (exposure, era_reward_points) =
                join(client.fetch(&stakers, None), client.fetch(&rewards, None)).await;
            let (exposure, era_reward_points) = (
                exposure?,
                era_reward_points?.unwrap_or_else(Default::default),
            );
            let s: Result<_, Error> = Ok((
                era_reward_points
                    .individual
                    .get(&validator_stash)
                    .map(|points| *points)
                    .unwrap_or_else(|| Zero::zero()),
                era,
            ));
            s
        };
        futures.push(future);
    }
    let mut eras = vec![];
    while let Some(e) = futures.next().await {
        let (points, era) = e?;
        if points != 0 {
            eras.push(era);
        }
    }
    Ok(eras)
}
