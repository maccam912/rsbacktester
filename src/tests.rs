#[cfg(test)]
mod tests {
    use actix::Actor;
    use num_rational::Ratio;
    use crate::{account::AccountBalance, engine::Engine};

    #[test]
    fn test_basic() {
        assert!(1 == 1);
    }
    
    async fn test_actors() {
        let engine = Engine::new();
        let engine_addr = engine.start();

        // Send Ping message.
        // send() message returns Future object, that resolves to message result
        let result = engine_addr.send(AccountBalance).await;
        assert!(result.unwrap().unwrap() == Ratio::new(10000, 1));
    }
}