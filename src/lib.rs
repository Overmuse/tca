use alpaca::{AlpacaMessage, Event};
use anyhow::Error;
use polygon::ws::{Quote, Trade};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashMap};
use stream_processor::StreamProcessor;
use uuid::Uuid;

mod metrics;
use metrics::PriceImprovement;

#[derive(Deserialize, Debug)]
pub enum TcaInput {
    Quote(Quote),
    Trade(Trade),
    // TODO: We should be able to deserialize just an OrderEvent here.
    AlpacaMessage(AlpacaMessage),
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "snake_case", tag = "metric")]
pub enum TcaMetric {
    PriceImprovement(PriceImprovement),
    ExecutionSpeed { millis: usize },
    EffectiveSpread(f64),
    QuotedSpread(f64),
    Efq(f64),
}

#[derive(Serialize, Debug)]
pub struct TcaOutput {
    metric: TcaMetric,
    client_order_id: Uuid,
}

struct TransactionCostAnalyzer {
    last_price: HashMap<String, f64>,
}

#[async_trait::async_trait]
impl StreamProcessor for TransactionCostAnalyzer {
    type Input = TcaInput;
    type Output = TcaOutput;
    type Error = Error;

    async fn handle_message(
        &self,
        input: Self::Input,
    ) -> Result<Option<Vec<Self::Output>>, Self::Error> {
        match input {
            TcaInput::Quote(quote) => todo!(),
            TcaInput::Trade(trade) => todo!(),
            TcaInput::AlpacaMessage(msg) => {
                if let AlpacaMessage::TradeUpdates(order) = msg {
                    // TODO: Maybe we can do something smart with partial fills as well
                    if let Event::Fill { .. } = order.event {
                        let trade = order.order;
                        todo!()
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
        }
    }

    fn assign_topic(&self, _output: &Self::Output) -> Cow<str> {
        "tca".into()
    }

    fn assign_key(&self, output: &Self::Output) -> Cow<str> {
        output.client_order_id.to_string().into()
    }
}
