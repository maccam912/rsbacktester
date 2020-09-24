use num_rational::Ratio;

#[derive(Debug, Clone)]
pub struct Position {
    pub asset: String,
    pub lots: i64,
    pub costbasis: Ratio<i64>,
}