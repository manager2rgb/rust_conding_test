use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub enum Type {
    #[serde(rename = "deposit")]
    Deposit,
    #[serde(rename = "withdrawal")]
    Withdrawal,
    #[serde(rename = "dispute")]
    Dispute,
    #[serde(rename = "resolve")]
    Resolve,
    #[serde(rename = "chargeback")]
    Chargeback,
}

#[derive(Debug, Deserialize)]
pub struct Transaction {
    #[serde(rename = "type")]
    pub t_type: Type,
    #[serde(rename = "client")]
    pub t_client_id: u16,
    #[serde(rename = "tx")]
    pub transaction_id: u32,
    pub amount: f32,
}
