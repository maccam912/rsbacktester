use rust_decimal::prelude::*;

#[derive(Debug, Clone)]
pub struct Position {
    pub asset: String,
    pub lots: i64,
    pub cost_basis: Decimal,
}