#![allow(dead_code)]
use chrono::prelude::*;
use rust_decimal::prelude::*;
use std::path::Path;
use serde::Deserialize;
use hashbrown::HashMap;

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
    indicators: hashbrown::HashMap<String, Box<dyn indicators::Indicator>>,
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
        let _signals = self.check_signals();

        self.index += 1;
    }

    pub fn check_signals(self: &Engine) -> Vec<Signal> {
        let s = Signal{asset: "SPY".to_string(), direction_up: true, magnitude: None, duration: None};
        vec![s]
    }

    pub fn register_indicator(self: &mut Engine, name: String, indicator: Box<dyn indicators::Indicator>) {
        self.indicators.insert(name, indicator);
    }

    pub fn update_indicators(self: &mut Engine) {
        for (_, indicator) in &mut self.indicators {
            let stepvaluetick = self.prices.ticks[self.index as usize];
            let stepvaluesum = stepvaluetick.ask.checked_add(stepvaluetick.bid);
            let stepvalue = stepvaluesum.unwrap().checked_div(Decimal::new(2, 0));
            indicator.update(stepvalue.unwrap())
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
    Engine{acct: init_acct(cash), time: t1, prices: prices, index: 0, indicators: HashMap::new()}
}

#[cfg(test)]
mod tests {
    use crate::{TS, Tick, init_engine, indicators};
    use chrono::prelude::*;
    use rust_decimal::Decimal;
    use std::{collections::VecDeque, path::Path};

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
        let _ = init_engine(&"test_resources/ticks.csv", 10000);
    }

    #[test]
    #[ignore]
    fn test_large_dataframe() {
        let _ = init_engine(&"test_resources/mgcticks.csv", 10000);
    }

    #[test]
    fn test_moving_average() {
        let mut engine = init_engine(&"test_resources/ticks.csv", 10000);
        let i = indicators::MovingAverage{length: 10, prices: VecDeque::new(), input: "price".to_string()};
        engine.register_indicator("ind1".to_string(), Box::new(i));
        engine.step();
        engine.step();
        assert!(engine.indicators["ind1"].value().expect("Could not get MA value") == 1657.6);
    }

    #[test]
    fn test_e2e() {
        let mut engine = init_engine(&"test_resources/mgcticks.csv", 10000);
        println!("Engine initialized");
        let i = indicators::MovingAverage{length: 1000, prices: VecDeque::new(), input: "price".to_string()};
        engine.register_indicator("ind2".to_string(), Box::new(i));
        for _ in 1..engine.prices.ticks.len() {
            engine.step();
        }
        println!("All steps done!");
        println!("{:?}", engine.indicators["ind2"].value());
    }
}
