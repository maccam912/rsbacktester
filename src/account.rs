use actix::prelude::*;
use hashbrown::HashMap;
use num_rational::Ratio;

use crate::position::Position;

#[derive(Message)]
#[rtype(result = "Result<Ratio<i64>, std::io::Error>")]
pub struct AccountBalance;

pub struct Account {
    pub cash: Ratio<i64>,
    pub positions: HashMap<String, Position>,
}

impl Actor for Account {
    type Context = Context<Self>;
}

impl Handler<AccountBalance> for Account {
    type Result = Result<Ratio<i64>, std::io::Error>;

    fn handle(&mut self, _msg: AccountBalance, _ctx: &mut Context<Self>) -> Self::Result {

        Ok(self.cash)
    }
}