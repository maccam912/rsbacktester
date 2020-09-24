use actix::prelude::*;
use hashbrown::HashMap;
use num_rational::Ratio;

use crate::{account::{Account, AccountBalance}, bar::Bar, indicators::Indicators};

pub struct Engine {
    account: Addr<Account>,
    indicators: HashMap<String,Addr<Indicators>>,
    bars: Vec<Bar>,
}

impl Actor for Engine {
    type Context = Context<Self>;
}

impl Engine {
    pub fn new() -> Engine {
        let acct = Account{cash: Ratio::new(10000, 1), positions: HashMap::new()};
        Engine{account: acct.start(), indicators: HashMap::new(), bars: Vec::new()}
    }
}

impl Handler<AccountBalance> for Engine {
    type Result = Result<Ratio<i64>, std::io::Error>;

    fn handle(&mut self, _msg: AccountBalance, _ctx: &mut Context<Self>) -> Self::Result {
        let result = futures::executor::block_on(self.account.send(AccountBalance));
        result.unwrap()
    }
}