use alpaca::{AlpacaMessage, Event};
use anyhow::{anyhow, Error, Result};
use polygon::ws::Quote;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::{borrow::Cow, collections::HashMap};
use stream_processor::{StreamProcessor, StreamRunner};

mod metrics;
mod settings;
use metrics::{execution_speed, price_improvement, ExecutionSpeed, PriceImprovement};
pub use settings::Settings;

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum TcaInput {
    Quote(Quote),
    // TODO: We should be able to deserialize just an OrderEvent here.
    AlpacaMessage(AlpacaMessage),
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "snake_case", tag = "metric")]
pub enum TcaMetric {
    PriceImprovement(PriceImprovement),
    ExecutionSpeed(ExecutionSpeed),
    EffectiveSpread(f64),
    QuotedSpread(f64),
    Efq(f64),
}

#[derive(Serialize, Debug)]
pub struct TcaOutput {
    metric: TcaMetric,
    client_order_id: String,
}

struct TransactionCostAnalyzer {
    // TODO: I think using a Mutex is fine here cause we don't actually call any .await in the
    // critical section
    last_quote: Mutex<HashMap<String, Quote>>,
}

impl TransactionCostAnalyzer {
    fn new() -> Self {
        Self {
            last_quote: Mutex::new(HashMap::new()),
        }
    }
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
            TcaInput::Quote(quote) => {
                self.last_quote
                    .lock()
                    .expect("Lock poisoned")
                    .insert(quote.symbol.clone(), quote);
                Ok(None)
            }
            TcaInput::AlpacaMessage(msg) => {
                if let AlpacaMessage::TradeUpdates(order) = msg {
                    // TODO: Maybe we can do something smart with partial fills as well
                    if let Event::Fill { price, .. } = order.event {
                        let trade = order.order;
                        let client_order_id = trade
                            .client_order_id
                            .as_ref()
                            .ok_or(anyhow!("Missing client_order_id"))?;
                        let execution_speed = TcaOutput {
                            metric: TcaMetric::ExecutionSpeed(execution_speed(&trade)?),
                            client_order_id: client_order_id.to_string(),
                        };
                        let guard = self.last_quote.lock().expect("Lock poisoned");
                        let quote = guard.get(&trade.symbol).ok_or(anyhow!("No last quote"))?;
                        let price_improvement = TcaOutput {
                            metric: TcaMetric::PriceImprovement(price_improvement(
                                quote,
                                &trade.side,
                                price,
                            )?),
                            client_order_id: client_order_id.to_string(),
                        };
                        Ok(Some(vec![execution_speed, price_improvement]))
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

    fn assign_key(&self, output: &TcaOutput) -> Cow<str> {
        Cow::Owned(output.client_order_id.clone())
    }
}

pub async fn run(settings: Settings) -> Result<()> {
    let runner = StreamRunner::new(TransactionCostAnalyzer::new(), settings.kafka);
    runner.run().await.map_err(From::from)
}
