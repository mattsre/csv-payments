use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct Transaction {
    pub r#type: TransactionType,
    #[serde(rename = "client")]
    pub client_id: u16,
    #[serde(rename = "tx")]
    pub tx_id: u32,
    #[serde(deserialize_with = "csv::invalid_option")]
    pub amount: Option<Decimal>,
}
