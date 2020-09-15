use hashbrown::HashMap;
use rust_decimal::prelude::*;
use anyhow::anyhow;

use crate::position::Position;

/// `OrderState` tracks if an order is `Pending`,
/// `Rejected`, or has been `Executed`.
#[derive(Debug, Clone, PartialEq)]
pub enum OrderState {
    Pending,
    Rejected,
    Executed,
}

/// `Order` tracks in progres or recently executed
/// orders.
#[derive(Debug, Clone)]
pub struct Order {
    pub state: OrderState,
    pub asset: String,
    pub lots: isize,
    pub cost_basis: Option<Decimal>,
}

/// `Account` tracks the current state of an account,
/// including remaining `cash`, any `Position`s open,
/// a history of `trade`s, and any pending or recently executed
/// `orders`.
#[derive(Debug, Clone)]
pub struct Account {
    pub cash: Decimal,
    pub portfolio: HashMap<String, Position>,
    pub trades: Vec<Position>,
    pub orders: Vec<Order>,
}

impl Account {

    pub fn submit_order(self: &mut Self, asset: String, lots: isize) {
        let order = Order{state: OrderState::Pending, asset, lots, cost_basis: None};
        self.orders.push(order);
    }

    pub fn clear_executed(self: &mut Self) {
        for i in (0..self.orders.len()).rev() {
            if self.orders[i].state == OrderState::Executed {
                self.orders.remove(i);
            }
        }
    }

    pub fn position(self: &mut Self, p: Position) -> anyhow::Result<()> {
        let response = self._position(p)?;

        match response {
            Some(asset) => {
                self.portfolio.remove(&asset);
            },
            None => {},
        };
        Ok(())
    }

    fn _position(self: &mut Self, p: Position) -> anyhow::Result<Option<String>> {
        // Check if account can support position
        let cost = p.cost_basis.checked_mul(Decimal::new(p.lots as i64, 0)).unwrap();
        if cost.gt(&self.cash) {
            return Err(anyhow!("Not enough cash in account to open position"));
        }

        let maybe_pos = self.portfolio.get_mut(&p.asset);

        match maybe_pos {
            Some(pos) => {
                let current_equity = pos.cost_basis.checked_mul(Decimal::new(pos.lots as i64, 0)).unwrap();
                let new_equity = cost;
                let total_equity = current_equity.checked_add(new_equity).unwrap();
                pos.lots += p.lots;
                if pos.lots != 0 {
                    let new_cb = total_equity.checked_div(Decimal::new(pos.lots as i64, 0)).unwrap();
                    pos.cost_basis = new_cb;
                }
                self.cash = self.cash.checked_sub(cost).unwrap();
                self.trades.push(p);
                if pos.lots == 0 {
                    Ok(Some(pos.asset.clone()))
                } else {
                    Ok(None)
                }
            },
            None => {
                self.cash = self.cash.checked_sub(cost).unwrap();
                self.portfolio.insert(p.asset.clone(), p.clone());
                self.trades.push(p);
                Ok(None)
            }
        }
    }
}