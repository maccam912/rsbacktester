#![allow(dead_code)]
use chrono::prelude::*;
use hashbrown::HashMap;
use rust_decimal::prelude::*;
use serde::Deserialize;
use std::path::Path;

pub mod indicators;

#[derive(Debug, Clone, Copy)]
pub struct Tick {
    pub timestamp: DateTime<Utc>,
    pub bid: Decimal,
    pub ask: Decimal,
}

/// `TS` is a time series of `Tick`s.
/// ```
/// use rsbacktester::{Tick, TS};
/// use rust_decimal::Decimal;
/// use chrono::Utc;
///
/// let t = Tick{timestamp: Utc::now(), bid: Decimal::new(202, 2), ask: Decimal::new(203, 1)};
/// let ts = TS{ticks: vec![t]};
/// # assert!(ts.ticks.len() == 1);
/// ```
#[derive(Debug, Clone)]
pub struct TS {
    pub ticks: Vec<Tick>,
}

#[derive(Debug, Clone)]
pub struct Position {
    pub asset: String,
    pub lots: i64,
    pub cost_basis: Decimal,
}

#[derive(Debug, Clone)]
pub struct Account {
    pub cash: Decimal,
    pub portfolio: Vec<Position>,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Engine {
    pub acct: Account,
    pub time: DateTime<Utc>,
    pub prices: TS,
    pub index: i64,
    pub indicators: hashbrown::HashMap<String, indicators::Indicator>,
}

#[derive(Debug)]
pub struct Signal {
    pub asset: String,
    pub direction_up: bool,
    pub magnitude: Option<f32>,
    pub duration: Option<i32>,
}

impl Engine {
    pub fn step(self: &mut Engine) {
        let iv = self.indicator_values();
        self.update_indicators(iv);
        let _signals = self.check_signals();

        self.index += 1;
    }

    pub fn check_signals(self: &Engine) -> Vec<Signal> {
        let s = Signal {
            asset: "SPY".to_string(),
            direction_up: true,
            magnitude: None,
            duration: None,
        };
        vec![s]
    }

    pub fn register_indicator(
        self: &mut Engine,
        name: String,
        indicator: indicators::Indicator,
    ) {
        self.indicators.insert(name, indicator);
    }

    fn indicator_values(self: &Engine) -> HashMap<String, Option<f64>> {
        let mut values = HashMap::new();
        for (name, ind) in &self.indicators {
            let v = ind.value();
            values.insert(name.to_string(), v);
        }
        values
    }

    pub fn update_indicators(self: &mut Engine, ind_values: HashMap<String, Option<f64>>) {
        for (_, indicator) in &mut self.indicators {
            if &indicator.get_input() == "price" {
                let stepvaluetick = self.prices.ticks[self.index as usize];
                let stepvaluesum = stepvaluetick.ask.checked_add(stepvaluetick.bid);
                let stepvalue = stepvaluesum.unwrap().checked_div(Decimal::new(2, 0));
                let v: Option<f64> = stepvalue.expect("Decimal was screwy").to_f64();
                indicator.update(v);
            } else {
                let v = ind_values[&indicator.get_input()];
                indicator.update(v);
            }
        }
    }
}

#[derive(Debug, Deserialize)]
struct Record {
    #[serde(rename = "Date")]
    pub date: String,
    #[serde(rename = "Time")]
    pub time: String,
    #[serde(rename = "Bid")]
    pub bid: String,
    #[serde(rename = "Ask")]
    pub ask: String,
}

fn init_acct(cash: i64) -> Account {
    Account {
        cash: Decimal::from(cash),
        portfolio: vec![],
    }
}

fn record_to_tick(r: &Record) -> anyhow::Result<Tick> {
    let d = NaiveDate::parse_from_str(&r.date, "%Y/%m/%d")?;
    let t = NaiveTime::parse_from_str(&r.time, "%H:%M:%S%.f")?;
    let dt = NaiveDateTime::new(d, t);
    let utc_dt: DateTime<Utc> = DateTime::from_utc(dt, Utc);

    let ask: Decimal = Decimal::from_str(&r.ask)?;
    let bid: Decimal = Decimal::from_str(&r.bid)?;
    Ok(Tick {
        timestamp: utc_dt,
        ask: ask,
        bid: bid,
    })
}

fn init_prices<P: AsRef<Path>>(path: &P) -> anyhow::Result<TS> {
    let mut rdr = csv::Reader::from_path(path)?;
    let mut ticks = vec![];
    for result in rdr.deserialize() {
        let record: Record = result?;
        let tick = record_to_tick(&record)?;
        ticks.push(tick);
    }

    Ok(TS { ticks })
}

pub fn init_engine<P: AsRef<Path>>(path: &P, cash: i64) -> Engine {
    let prices: TS = init_prices(path).expect("could not load prices");
    let t1 = prices.ticks[0].timestamp;
    Engine {
        acct: init_acct(cash),
        time: t1,
        prices: prices,
        index: 0,
        indicators: HashMap::new(),
    }
}

#[cfg(test)]
mod tests {
    use crate::{indicators, indicators::Indicator, init_engine, Tick, TS};
    use chrono::prelude::*;
    use rust_decimal::Decimal;
    use std::path::Path;

    #[test]
    fn test_tick() {
        let t = Tick {
            timestamp: Utc::now(),
            bid: Decimal::new(202, 2),
            ask: Decimal::new(203, 1),
        };
        let twenty = Decimal::new(20, 0);
        let thirty = Decimal::new(30, 0);
        assert!(t.ask.ge(&twenty));
        assert!(t.bid.le(&thirty));
        assert!(t.timestamp <= Utc::now());
    }

    #[test]
    fn test_ts() {
        let t = Tick {
            timestamp: Utc::now(),
            bid: Decimal::new(202, 2),
            ask: Decimal::new(203, 1),
        };
        let ts = TS { ticks: vec![t] };
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
        let i = indicators::MovingAverage::new(10, "price".to_string());
        engine.register_indicator("ind1".to_string(), Indicator::MovingAverage(i));
        engine.step();
        engine.step();
        assert!(
            engine.indicators["ind1"]
                .value()
                .expect("Could not get MA value")
                == 0.5
        );
    }

    #[test]
    fn test_momentum() {
        let mut engine = init_engine(&"test_resources/ticks.csv", 10000);
        println!("Engine initialized");
        let i = indicators::MovingAverage::new(4, "price".to_string());
        engine.register_indicator("ind2".to_string(), Indicator::MovingAverage(i));
        let mom = indicators::Momentum::new(3, "ind2".to_string());
        engine.register_indicator("mom".to_string(), Indicator::Momentum(mom));
        for _ in 0..engine.prices.ticks.len() {
            engine.step();
        }
        assert!(engine.indicators["ind2"].value().unwrap() == 27.5);
        assert!(engine.indicators["mom"].value().unwrap() == 2.0);
    }
}
