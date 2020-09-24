use chrono::{DateTime, Utc};
use num_rational::Ratio;

pub struct Bar {
    timestamp: DateTime<Utc>,
    open: Ratio<i64>,
    high: Ratio<i64>,
    low: Ratio<i64>,
    close: Ratio<i64>,
}