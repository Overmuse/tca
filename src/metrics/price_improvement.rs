use alpaca::Side;
use anyhow::{anyhow, Result};
use polygon::ws::Quote;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct PriceImprovement {
    pub amount: f64,
    pub effective_spread: f64,
    pub quoted_spread: f64,
    pub efq: f64,
}

pub fn price_improvement(quote: &Quote, side: &Side, fill_price: f64) -> Result<PriceImprovement> {
    let bid_price = quote
        .bid_quote
        .as_ref()
        .ok_or_else(|| anyhow!("Missing bid quote"))?
        .price;
    let ask_price = quote
        .ask_quote
        .as_ref()
        .ok_or_else(|| anyhow!("Missing ask quote"))?
        .price;
    let mid_price = (bid_price + ask_price) / 2.0;
    let quoted_spread = ask_price - bid_price;
    let res = match side {
        Side::Buy => {
            let effective_spread = 2.0 * (fill_price - mid_price);
            PriceImprovement {
                amount: ask_price - fill_price,
                effective_spread,
                quoted_spread,
                efq: effective_spread / quoted_spread,
            }
        }
        Side::Sell => {
            let effective_spread = 2.0 * (mid_price - fill_price);
            PriceImprovement {
                amount: fill_price - bid_price,
                effective_spread,
                quoted_spread,
                efq: effective_spread / quoted_spread,
            }
        }
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
        assert!((buy_pi.effective_spread).abs() < f64::EPSILON);
        assert!((buy_pi.quoted_spread - 10.0).abs() < f64::EPSILON);
        assert!((buy_pi.efq).abs() < f64::EPSILON);

        let sell_pi = price_improvement(&quote, &Side::Sell, 94.0).unwrap();
        assert!((sell_pi.amount - 4.0).abs() < f64::EPSILON);
        assert!((sell_pi.effective_spread - 2.0).abs() < f64::EPSILON);
        assert!((sell_pi.quoted_spread - 10.0).abs() < f64::EPSILON);
        assert!((sell_pi.efq - 0.2).abs() < f64::EPSILON);
    }
}
