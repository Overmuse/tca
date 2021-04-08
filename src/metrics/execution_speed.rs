use alpaca::{Order, OrderStatus};
use anyhow::{anyhow, Result};
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct ExecutionSpeed {
    pub micros: u32,
}

pub fn execution_speed(order: &Order) -> Result<ExecutionSpeed> {
    if order.status != OrderStatus::Filled {
        return Err(anyhow!(
            "Execution speed can't be calculated for unfilled order"
        ));
    }
    let speed = order.filled_at.ok_or(anyhow!("Missing filled_at value"))?
        - order
            .submitted_at
            .ok_or(anyhow!("Missing submitted_at value"))?;
    let micros = speed.num_microseconds().expect("Should work") as u32;
    Ok(ExecutionSpeed { micros })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let order_str = r#"{"id":"8caf0f25-18ec-402c-81d9-9afe473d13b2","client_order_id":"ed14a845-27b2-4061-a34a-0104f2469ef9","created_at":"2021-04-08T15:20:03.990566Z","updated_at":"2021-04-08T15:20:04.882802Z","submitted_at":"2021-04-08T15:20:03.978566Z","filled_at":"2021-04-08T15:20:04.874214Z","expired_at":null,"canceled_at":null,"failed_at":null,"replaced_at":null,"replaced_by":null,"replaces":null,"asset_id":"092efc51-b66b-4355-8132-d9c3796b9a76","symbol":"XOM","asset_class":"us_equity","notional":null,"qty":"4","filled_qty":"4","filled_avg_price":"55.8","type":"market","side":"buy","time_in_force":"gtc","status":"filled","extended_hours":false,"legs":null,"hwm":null}"#;
        let order: Order = serde_json::from_str(order_str).unwrap();
        let speed = execution_speed(&order).unwrap();
        assert_eq!(speed.micros, 895648)
    }
}
