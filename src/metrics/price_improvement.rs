use alpaca::{Order, Side};
use polygon::ws::Quote;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct PriceImprovement {
    pub amount: f64,
    pub percentage: f64,
}

pub fn price_improvement(quote: Quote, side: Side, fill_price: f64) -> PriceImprovement {
    let bid_price = quote.bid_quote.as_ref().expect("No bid quote").price;
    let ask_price = quote.ask_quote.as_ref().expect("No ask quote").price;
    match side {
        Side::Buy => PriceImprovement {
            amount: ask_price - fill_price,
            percentage: 1.0 - fill_price / ask_price,
        },
        Side::Sell => PriceImprovement {
            amount: fill_price - bid_price,
            percentage: fill_price / bid_price - 1.0,
        },
    }
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
        let buy_pi = price_improvement(quote.clone(), Side::Buy, 95.0);
        assert_eq!(buy_pi.amount, 5.0);
        assert_eq!(buy_pi.percentage, 0.05);

        let sell_pi = price_improvement(quote, Side::Sell, 99.0);
        assert_eq!(buy_pi.amount, 5.0);
        assert_eq!(buy_pi.percentage, 0.1);
    }
}
