#![allow(dead_code)]
use chrono::prelude::*;
use rust_decimal::prelude::*;
use std::path::Path;
use serde::Deserialize;

mod indicators;

#[derive(Debug, Clone, Copy)]
pub struct Tick {
    timestamp: DateTime<Utc>,
    bid:  Decimal,
    ask: Decimal,
}

#[derive(Debug)]
pub struct TS {
    ticks: Vec<Tick>,
}

#[derive(Debug)]
pub struct Position {
    asset: String,
    lots: i64,
    cost_basis: Decimal,
}

#[derive(Debug)]
pub struct Account {
    cash: Decimal,
    portfolio: Vec<Position>,
}

#[derive(Debug)]
pub struct Engine {
    acct: Account,
    time: DateTime<Utc>,
    prices: TS,
    index: i64,
    indicators: Vec<Box<dyn indicators::Indicator>>,
}

#[derive(Debug)]
pub struct Signal {
    asset: String,
    direction_up: bool,
    magnitude: Option<f32>,
    duration: Option<i32>,
}

impl Engine {
    pub fn step(self: &mut Engine) {
        self.update_indicators();
        let signals = self.check_signals();
    }

    pub fn check_signals(self: &Engine) -> Vec<Signal> {
        let s = Signal{asset: "SPY".to_string(), direction_up: true, magnitude: None, duration: None};
        vec![s]
    }

    pub fn register_indicator(self: &mut Engine, indicator: Box<dyn indicators::Indicator>) {
        self.indicators.push(indicator);
    }

    pub fn update_indicators(self: &mut Engine) {
        for indicator in &mut self.indicators {
            indicator.update(self)
        }
    }

}

#[derive(Debug, Deserialize)]
struct Record {
    #[serde(rename = "Date")]
    date: String,
    #[serde(rename = "Time")]
    time: String,
    #[serde(rename = "Bid")]
    bid: String,
    #[serde(rename = "Ask")]
    ask: String,
}

fn init_acct(cash: i64) -> Account {
    Account{cash: Decimal::from(cash), portfolio: vec![]}
}

fn record_to_tick(r: &Record) -> anyhow::Result<Tick> {
    let d = NaiveDate::parse_from_str(&r.date, "%Y/%m/%d")?;
    let t = NaiveTime::parse_from_str(&r.time, "%H:%M:%S%.f")?;
    let dt = NaiveDateTime::new(d, t);
    let utc_dt: DateTime<Utc> = DateTime::from_utc(dt, Utc);

    let ask: Decimal = Decimal::from_str(&r.ask)?;
    let bid: Decimal = Decimal::from_str(&r.bid)?;
    Ok(Tick{timestamp: utc_dt, ask: ask, bid: bid})
}

fn init_prices<P: AsRef<Path>>(path: &P) -> anyhow::Result<TS> {
    let mut rdr = csv::Reader::from_path(path)?;
    let mut ticks = vec![];
    for result in rdr.deserialize() {
        let record: Record = result?;
        let tick = record_to_tick(&record)?;
        ticks.push(tick);
    }

    Ok(TS{ticks})
}

pub fn init_engine<P: AsRef<Path>>(path: &P, cash: i64) -> Engine {
    let prices: TS = init_prices(path).expect("could not load prices");
    let t1 = prices.ticks[0].timestamp;
    Engine{acct: init_acct(cash), time: t1, prices: prices, index: 0, indicators: vec![]}
}

#[cfg(test)]
mod tests {
    use crate::{TS, Tick, init_engine};
    use chrono::prelude::*;
    use rust_decimal::Decimal;
    use std::path::Path;

    #[test]
    fn test_tick() {
        let t = Tick{timestamp: Utc::now(), bid: Decimal::new(202, 2), ask: Decimal::new(203, 1)};
        let twenty = Decimal::new(20, 0);
        let thirty = Decimal::new(30, 0);
        assert!(t.ask.ge(&twenty));
        assert!(t.bid.le(&thirty));
        assert!(t.timestamp <= Utc::now());
    }

    #[test]
    fn test_ts() {
        let t = Tick{timestamp: Utc::now(), bid: Decimal::new(202, 2), ask: Decimal::new(203, 1)};
        let ts = TS{ticks: vec![t]};
        assert!(ts.ticks.len() == 1);
    }

    #[test]
    fn test_engine() {
        let path = Path::new("test_resources/ticks.csv");
        let _engine = init_engine(&path, 10000);
    }

    #[test]
    fn test_init_engine() {
        let engine = init_engine(&"test_resources/ticks.csv", 10000);
        println!("{:?}", engine);
    }

    #[test]
    #[ignore]
    fn test_large_dataframe() {
        let engine = init_engine(&"test_resources/mgcticks.csv", 10000);
        println!("{:?}", engine.prices.ticks.len());
    }
}
