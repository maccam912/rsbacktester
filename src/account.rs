use hashbrown::HashMap;
use rust_decimal::prelude::*;
use anyhow::anyhow;

use crate::position::Position;

#[derive(Debug, Clone)]
pub struct Account {
    pub cash: Decimal,
    pub portfolio: HashMap<String, Position>,
    pub trades: Vec<Position>,
}

impl Account {
    pub fn order(self: &mut Self, p: Position) -> anyhow::Result<()> {
        let response = self._order(p)?;

        match response {
            Some(asset) => {
                self.portfolio.remove(&asset);
            },
            None => {},
        };
        Ok(())
    }

    fn _order(self: &mut Self, p: Position) -> anyhow::Result<Option<String>> {
        // Check if account can support position
        let cost = p.cost_basis.checked_mul(Decimal::new(p.lots, 0)).unwrap();
        if cost.gt(&self.cash) {
            return Err(anyhow!("Not enough cash in account to open position"));
        }

        let maybe_pos = self.portfolio.get_mut(&p.asset);

        match maybe_pos {
            Some(pos) => {
                let current_equity = pos.cost_basis.checked_mul(Decimal::new(pos.lots, 0)).unwrap();
                let new_equity = cost;
                let total_equity = current_equity.checked_add(new_equity).unwrap();
                pos.lots += p.lots;
                if pos.lots != 0 {
                    let new_cb = total_equity.checked_div(Decimal::new(pos.lots, 0)).unwrap();
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