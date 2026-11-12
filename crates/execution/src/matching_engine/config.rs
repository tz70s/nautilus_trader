// -------------------------------------------------------------------------------------------------
//  Copyright (C) 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
//  https://nautechsystems.io
//
//  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
//  You may not use this file except in compliance with the License.
//  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
// -------------------------------------------------------------------------------------------------

use std::{cell::RefCell, fmt, rc::Rc};

use ahash::AHashMap;
use nautilus_model::data::order::OrderId;
use serde::{Deserialize, Serialize};

/// Each L3 book order that a venue placed ahead of an earlier order at its
/// price (Hyperliquid's queue priority, its `insert_before`), by its id: the
/// order it went ahead of. A book delta can't say so, so the replay that
/// streams the deltas fills this beside them, and the matching engine drops an
/// order's entry once it leaves the book.
#[derive(Clone, Default)]
pub struct InsertBefore(pub Rc<RefCell<AHashMap<OrderId, OrderId>>>);

impl fmt::Debug for InsertBefore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "InsertBefore({} orders)", self.0.borrow().len())
    }
}

/// Configuration for `OrderMatchingEngine` instances.
///
/// With `defer_option_settlement`, automatic checks at the exact option expiry
/// timestamp cancel orders but defer settlement to `process_instrument_expiration`.
/// To settle at expiry, the caller schedules that call after processing the
/// timestamp's market data. Automatic checks after expiry can also settle.
/// Explicit contract-close events bypass this deferral.
#[derive(Debug, Clone, Deserialize, Serialize, bon::Builder)]
#[serde(default, deny_unknown_fields)]
pub struct OrderMatchingEngineConfig {
    #[builder(default = true)]
    pub bar_execution: bool,
    #[builder(default)]
    pub bar_adaptive_high_low_ordering: bool,
    #[builder(default = true)]
    pub trade_execution: bool,
    #[builder(default)]
    pub liquidity_consumption: bool,
    #[builder(default = true)]
    pub reject_stop_orders: bool,
    #[builder(default = true)]
    pub support_gtd_orders: bool,
    #[builder(default = true)]
    pub support_contingent_orders: bool,
    #[builder(default = true)]
    pub use_position_ids: bool,
    #[builder(default)]
    pub use_random_ids: bool,
    #[builder(default = true)]
    pub use_reduce_only: bool,
    #[builder(default)]
    pub use_market_order_acks: bool,
    #[builder(default)]
    pub queue_position: bool,
    /// If a resting maker order fills when the book reaches its price, not
    /// only when a trade does. Book data can't tell a taker reaching the
    /// price from a post-only order that the resting one would have
    /// rejected, so with this off only trades fill resting orders.
    #[builder(default = true)]
    pub book_execution: bool,
    /// If set, a new resting order goes ahead of the orders at its price
    /// that arrived within this many milliseconds before it, as one paying a
    /// venue's queue priority fee does (Hyperliquid's: 400 ms), and stays
    /// behind older ones. On an L3 book; on others it joins the front. A
    /// modified order still joins the back.
    pub queue_priority_ms: Option<u64>,
    /// With `queue_priority_ms` on an L3 book, the book orders that went
    /// ahead of others: one that went ahead of an order ahead of an own order
    /// is ahead of it too, and one that arrived within `queue_priority_ms`
    /// before an own order stays ahead of it.
    #[serde(skip)]
    pub insert_before: Option<InsertBefore>,
    #[builder(default)]
    pub oto_full_trigger: bool,
    #[builder(default)]
    pub defer_option_settlement: bool,
    pub price_protection_points: Option<u32>,
}

impl Default for OrderMatchingEngineConfig {
    fn default() -> Self {
        Self::builder().build()
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    fn test_default() {
        let config = OrderMatchingEngineConfig::default();
        assert!(config.bar_execution);
        assert!(!config.bar_adaptive_high_low_ordering);
        assert!(config.trade_execution);
        assert!(!config.liquidity_consumption);
        assert!(config.reject_stop_orders);
        assert!(config.support_gtd_orders);
        assert!(config.support_contingent_orders);
        assert!(config.use_position_ids);
        assert!(!config.use_random_ids);
        assert!(config.use_reduce_only);
        assert!(!config.use_market_order_acks);
        assert!(!config.queue_position);
        assert!(config.book_execution);
        assert_eq!(config.queue_priority_ms, None);
        assert!(!config.oto_full_trigger);
        assert!(!config.defer_option_settlement);
        assert_eq!(config.price_protection_points, None);
    }
}
