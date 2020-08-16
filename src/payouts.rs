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

use crate::{common::get_stash, Error};
use futures::{future::join3,
              stream::{FuturesUnordered, StreamExt as _}};
use log::trace;
use std::marker::PhantomData;
use substrate_subxt::{sp_core::crypto::{AccountId32 as AccountId, Ss58AddressFormat},
                      sp_runtime::traits::Zero,
                      staking::{CurrentEraStore, ErasRewardPointsStore, HistoryDepthStore,
                                StakingLedger},
                      Client, KusamaRuntime};

static HISTORY_DEPTH: HistoryDepthStore<KusamaRuntime> = HistoryDepthStore {
    _runtime: PhantomData,
};
static CURRENT_ERA: CurrentEraStore<KusamaRuntime> = CurrentEraStore {
    _runtime: PhantomData,
};
pub(crate) async fn display_payouts(
    controller: AccountId,
    client: &Client<KusamaRuntime>,
    network: Ss58AddressFormat,
) -> Result<Vec<u32>, Error> {
    let history_depth = client.fetch_or_default(&HISTORY_DEPTH, None);
    let current_era = client.fetch_or_default(&CURRENT_ERA, None);
    let account_info = get_stash(&client, controller.clone(), network);
    let (history_depth, account_info, current_era) =
        join3(history_depth, account_info, current_era).await;
    let history_depth = history_depth?;
    let StakingLedger {
        stash,
        claimed_rewards,
        ..
    } = account_info?;
    let current_era = current_era?;
    let history_start = current_era.saturating_sub(history_depth);
    let mut futures = FuturesUnordered::new();
    trace!("Claimed rewards: {:?}", claimed_rewards);
    for era in history_start..=current_era {
        if claimed_rewards.binary_search(&era).is_ok() {
            continue
        }
        let stash = stash.clone();
        let future = async move {
            let rewards = ErasRewardPointsStore {
                index: era,
                _phantom: PhantomData,
            };
            let era_reward_points = client.fetch_or_default(&rewards, None).await?;
            let s: Result<_, Error> = Ok((
                era_reward_points
                    .individual
                    .get(&stash)
                    .cloned()
                    .unwrap_or_else(Zero::zero),
                era,
            ));
            s
        };
        futures.push(future);
    }
    let mut eras = vec![];
    while let Some(e) = futures.next().await {
        let (points, era) = e?;
        if points == 0 {
            trace!("Skipping era {} as it has no points", era);
        } else {
            trace!("Found {} points for era {}", points, era);
            eras.push(era);
        }
    }
    Ok(eras)
}
