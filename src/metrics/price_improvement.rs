use alpaca::Side;
use anyhow::{anyhow, Result};
use polygon::ws::Quote;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct PriceImprovement {
    pub amount: f64,
    pub percentage: f64,
}

pub fn price_improvement(quote: &Quote, side: &Side, fill_price: f64) -> Result<PriceImprovement> {
    let bid_price = quote
        .bid_quote
        .as_ref()
        .ok_or(anyhow!("Missing bid quote"))?
        .price;
    let ask_price = quote
        .ask_quote
        .as_ref()
        .ok_or(anyhow!("Missing ask quote"))?
        .price;
    let res = match side {
        Side::Buy => PriceImprovement {
            amount: ask_price - fill_price,
            percentage: 1.0 - fill_price / ask_price,
        },
        Side::Sell => PriceImprovement {
            amount: fill_price - bid_price,
            percentage: fill_price / bid_price - 1.0,
        },
    };
    Ok(res)
}

#[cfg(test)]
mod test {
    use super::*;
    use polygon::ws::{AskQuote, BidQuote};

    #[test]
    fn test() {
        let bid_quote = BidQuote {
            exchange_id: 0,
            price: 90.0,
            size: 100,
        };
        let ask_quote = AskQuote {
            exchange_id: 0,
            price: 100.0,
            size: 100,
        };
        let quote = Quote {
            symbol: "AAPL".into(),
            bid_quote: Some(bid_quote),
            ask_quote: Some(ask_quote),
            condition: None,
            timestamp: 0,
        };
        let buy_pi = price_improvement(&quote, &Side::Buy, 95.0).unwrap();
        assert!((buy_pi.amount - 5.0).abs() < f64::EPSILON);
        assert!((buy_pi.percentage - 0.05).abs() < f64::EPSILON);

        let sell_pi = price_improvement(&quote, &Side::Sell, 99.0).unwrap();
        assert!((sell_pi.amount - 9.0).abs() < f64::EPSILON);
        assert!((sell_pi.percentage - 0.1).abs() < f64::EPSILON);
    }
}
