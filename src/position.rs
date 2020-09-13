use rust_decimal::prelude::*;

#[derive(Debug, Clone)]
pub struct Position {
    pub asset: String,
    pub lots: isize,
    pub cost_basis: Decimal,
}