use alpaca::Side;
use anyhow::{anyhow, Result};
use polygon::ws::Quote;
use rust_decimal::prelude::*;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct PriceImprovement {
    pub amount: Decimal,
    pub effective_spread: Decimal,
    pub quoted_spread: Decimal,
    pub efq: Decimal,
}

pub fn price_improvement(
    quote: &Quote,
    side: &Side,
    fill_price: Decimal,
) -> Result<PriceImprovement> {
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
    let mid_price = (bid_price + ask_price) / Decimal::TWO;
    let quoted_spread = ask_price - bid_price;
    let res = match side {
        Side::Buy => {
            let effective_spread = Decimal::TWO * (fill_price - mid_price);
            PriceImprovement {
                amount: ask_price - fill_price,
                effective_spread,
                quoted_spread,
                efq: effective_spread / quoted_spread,
            }
        }
        Side::Sell => {
            let effective_spread = Decimal::TWO * (mid_price - fill_price);
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
    use chrono::prelude::*;
    use polygon::ws::{AskQuote, BidQuote};

    #[test]
    fn test() {
        let bid_quote = BidQuote {
            exchange_id: 0,
            price: Decimal::new(90, 0),
            size: 100,
        };
        let ask_quote = AskQuote {
            exchange_id: 0,
            price: Decimal::new(100, 0),
            size: 100,
        };
        let quote = Quote {
            symbol: "AAPL".into(),
            bid_quote: Some(bid_quote),
            ask_quote: Some(ask_quote),
            condition: None,
            timestamp: Utc::now(),
        };
        let buy_pi = price_improvement(&quote, &Side::Buy, Decimal::new(95, 0)).unwrap();
        assert_eq!(buy_pi.amount, Decimal::new(5, 0));
        assert_eq!(buy_pi.effective_spread, Decimal::ZERO);
        assert_eq!(buy_pi.quoted_spread, Decimal::new(10, 0));
        assert_eq!(buy_pi.efq, Decimal::ZERO);

        let sell_pi = price_improvement(&quote, &Side::Sell, Decimal::new(94, 0)).unwrap();
        assert_eq!(sell_pi.amount, Decimal::new(4, 0));
        assert_eq!(sell_pi.effective_spread, Decimal::new(2, 0));
        assert_eq!(sell_pi.quoted_spread, Decimal::new(10, 0));
        assert_eq!(sell_pi.efq, Decimal::new(2, 1));
    }
}
