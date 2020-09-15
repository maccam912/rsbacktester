#[cfg(test)]
mod tests {
    use crate::{indicators, indicators::Indicator, init_engine, Tick, TS, account::Account, position::Position, account::OrderState};
    use hashbrown::HashMap;
    use chrono::prelude::*;
    use rust_decimal::Decimal;
    use std::path::Path;

    #[test]
    fn test_tick() {
        let t = Tick {
            timestamp: Utc::now(),
            asset: "AAPL".to_string(),
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
            asset: "AAPL".to_string(),
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

    #[test]
    fn acct_open_position() {
        let mut acct = Account{cash: Decimal::new(10000, 0), portfolio: HashMap::new(), trades: vec![], orders: vec![]};
        let pos = Position{asset: "AAPL".to_string(), lots: 1, cost_basis: Decimal::new(1000, 0)};
        let resp = acct.position(pos);
        assert!(resp.is_ok());
        assert!(acct.portfolio.len() == 1);
        assert!(acct.cash.eq(&Decimal::new(9000, 0)));
        assert!(acct.portfolio["AAPL"].lots == 1);

        let p2 = Position{asset: "MSFT".to_string(), lots: 1, cost_basis: Decimal::new(2000, 0)};
        let resp = acct.position(p2);
        assert!(resp.is_ok());
        assert!(acct.portfolio.len() == 2);
        assert!(acct.cash.eq(&Decimal::new(7000, 0)));
        assert!(acct.portfolio["MSFT"].lots == 1);


        let p3 = Position{asset: "AAPL".to_string(), lots: 2, cost_basis: Decimal::new(250, 0)};
        let resp = acct.position(p3);
        assert!(resp.is_ok());
        assert!(acct.portfolio.len() == 2);
        assert!(acct.cash.eq(&Decimal::new(6500, 0)));
        assert!(acct.portfolio.get("AAPL").unwrap().cost_basis == Decimal::new(500, 0));
        assert!(acct.portfolio["AAPL"].lots == 3);


        let p4 = Position{asset: "AAPL".to_string(), lots: 1, cost_basis: Decimal::new(10000, 0)};
        let resp = acct.position(p4);
        assert!(resp.is_err());
        assert!(acct.cash.eq(&Decimal::new(6500, 0)));
        assert!(acct.portfolio.get("AAPL").unwrap().lots == 3);
    }

    #[test]
    fn acct_open_close() {
        let mut acct = Account{cash: Decimal::new(10000, 0), portfolio: HashMap::new(), trades: vec![], orders: vec![]};
        let pos = Position{asset: "AAPL".to_string(), lots: 1, cost_basis: Decimal::new(1000, 0)};
        let resp = acct.position(pos);
        assert!(resp.is_ok());
        assert!(acct.portfolio.len() == 1);
        assert!(acct.cash.eq(&Decimal::new(9000, 0)));

        let pos = Position{asset: "AAPL".to_string(), lots: -1, cost_basis: Decimal::new(3000, 0)};
        let resp = acct.position(pos);
        assert!(resp.is_ok());
        assert!(acct.portfolio.len() == 0);
        assert!(acct.cash.eq(&Decimal::new(12000, 0)));
    }

    #[test]
    fn check_placing_orders() {
        let mut engine = init_engine(&"test_resources/ticks.csv", 10000);
        engine.step();
        engine.step();
        engine.place_order("AAPL".to_string(), 1);
        assert!(engine.acct.orders.len() == 1);
        assert!(engine.acct.portfolio.len() == 0);
        assert!(engine.acct.orders[0].state == OrderState::Executed);
        engine.step();
        assert!(engine.acct.orders.len() == 0);
        assert!(engine.acct.portfolio.len() == 1);
        assert!(engine.acct.cash.lt(&Decimal::new(10000, 0)));
        assert!(engine.acct.portfolio["AAPL"].lots == 1);
    }

    #[test]
    fn check_several_orders() {
        let mut engine = init_engine(&"test_resources/ticks.csv", 10000);
        engine.step();
        engine.place_order("AAPL".to_string(), 1);
        engine.step();
        engine.place_order("AAPL".to_string(), -1);
        engine.step();
        assert!(engine.acct.portfolio.len() == 0);
        assert!(engine.acct.orders.len() == 0);
        assert!(engine.acct.trades.len() == 2);
        assert!(engine.acct.cash == Decimal::new(10001, 0));
    }
    
    #[test]
    fn total_equity() {
        let mut e = init_engine(&"test_resources/ticks.csv", 10000);
        assert!(e.equity() == Decimal::new(10000, 0));
        let result = e.acct.position(Position{asset: "AAPL".to_string(), lots: 1, cost_basis: Decimal::new(2, 0)});
        assert!(result.is_ok());
        e.step();
        assert!(e.equity() == Decimal::new(9998+0, 0));
        e.step();
        assert!(e.equity() == Decimal::new(9998+1, 0));
        e.step();
        assert!(e.equity() == Decimal::new(9998+2, 0));
        e.step();
        assert!(e.equity() == Decimal::new(9998+3, 0));
    }
    
    #[test]
    fn e2e_test() {
        let mut e = init_engine(&"test_resources/ticks.csv", 10000);
        let long_sma = indicators::MovingAverage::new(4, "price".to_string());
        let short_sma = indicators::MovingAverage::new(2, "price".to_string());
        e.register_indicator("long_sma".to_string(), indicators::Indicator::MovingAverage(long_sma));
        e.register_indicator("short_sma".to_string(), indicators::Indicator::MovingAverage(short_sma));
        
        // start going!
        while e.index < (e.prices.ticks.len()-1) as i64 {
            let ind_values = e.indicator_values();
            if ind_values["short_sma"].is_some() && ind_values["long_sma"].is_some() {
                if ind_values["short_sma"].unwrap() > ind_values["long_sma"].unwrap() {
                    // buy
                    let curr_aapl = e.acct.portfolio.get("AAPL");
                    if curr_aapl.is_none() || curr_aapl.unwrap().lots < 1 {
                        e.place_order("AAPL".to_string(), 1);
                    }
                } else {
                    // sell
                    let curr_aapl = e.acct.portfolio.get("AAPL");
                    if curr_aapl.is_none() || curr_aapl.unwrap().lots > -1 {
                        e.place_order("AAPL".to_string(), -1);
                    }
                }
            }
            println!("{:?}", e.acct);
            e.step();
        }
        let current_lots = e.acct.portfolio["AAPL"].lots;
        // close positions
        e.place_order("AAPL".to_string(), -1*current_lots);
        e.step();
        assert!(e.acct.portfolio.len() == 0);
        assert!(e.acct.cash.gt(&Decimal::new(10000, 0)));
    }
}
