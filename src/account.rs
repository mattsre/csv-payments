use std::fmt::Display;

use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};

use crate::transaction::{Transaction, TransactionType};

#[derive(Debug, Deserialize, Serialize)]
pub struct Account {
    #[serde(rename = "client")]
    pub client_id: u16,
    #[serde(rename = "available")]
    pub funds_available: Decimal,
    #[serde(rename = "held")]
    pub funds_held: Decimal,
    #[serde(rename = "total")]
    pub funds_total: Decimal,
    pub locked: bool,
}

impl Account {
    pub fn new(id: u16) -> Account {
        Account {
            client_id: id,
            funds_available: Decimal::new(0, 0),
            funds_held: Decimal::new(0, 0),
            funds_total: Decimal::new(0, 0),
            locked: false,
        }
    }

    pub fn settle_transaction(&mut self, tx: &Transaction, ref_tx: Option<&Transaction>) {
        match tx.r#type {
            TransactionType::Deposit => {
                if let Some(tx_amount) = tx.amount {
                    self.funds_available += tx_amount;
                    self.funds_total += tx_amount;
                }
            }
            TransactionType::Withdrawal => {
                if let Some(tx_amount) = tx.amount {
                    if self.funds_available >= tx_amount {
                        self.funds_available -= tx_amount;
                        self.funds_total -= tx_amount;
                    }
                }
            }
            TransactionType::Dispute => {
                if let Some(ref_tx) = ref_tx {
                    if let Some(tx_amount) = ref_tx.amount {
                        self.funds_available -= tx_amount;
                        self.funds_held += tx_amount;
                    }
                }
            }
            TransactionType::Resolve => {
                if let Some(ref_tx) = ref_tx {
                    if let Some(tx_amount) = ref_tx.amount {
                        self.funds_available += tx_amount;
                        self.funds_held -= tx_amount;
                    }
                }
            }
            TransactionType::Chargeback => {
                if let Some(ref_tx) = ref_tx {
                    if let Some(tx_amount) = ref_tx.amount {
                        self.funds_held -= tx_amount;
                        self.funds_total -= tx_amount;
                        self.locked = true;
                    }
                }
            }
        }
    }
}

impl Display for Account {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "id: {}", &self.client_id).unwrap_or(());
        writeln!(f, "funds available: {}", &self.funds_available).unwrap_or(());
        writeln!(f, "funds held: {}", &self.funds_held).unwrap_or(());
        writeln!(f, "funds total: {}", &self.funds_total).unwrap_or(());
        writeln!(f, "locked: {}", &self.locked).unwrap_or(());

        write!(f, "")
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;

    use crate::{
        account::Account,
        transaction::{Transaction, TransactionType},
    };

    #[test]
    fn settle_deposit_transaction() {
        let tx = Transaction {
            r#type: TransactionType::Deposit,
            client_id: 1,
            tx_id: 1,
            amount: Some(Decimal::new(1_05, 2)),
        };

        let mut acc = Account::new(tx.client_id);

        acc.settle_transaction(&tx, None);

        assert_eq!(acc.funds_available, Decimal::new(1_05, 2));
        assert_eq!(acc.funds_total, Decimal::new(1_05, 2));
    }

    #[test]
    fn settle_withdrawal_transaction() {
        let tx = Transaction {
            r#type: TransactionType::Withdrawal,
            client_id: 1,
            tx_id: 1,
            amount: Some(Decimal::new(1_05, 2)),
        };

        let mut acc = Account {
            client_id: tx.client_id,
            funds_available: Decimal::new(3_05, 2),
            funds_held: Decimal::new(0, 0),
            funds_total: Decimal::new(3_05, 2),
            locked: false,
        };

        acc.settle_transaction(&tx, None);

        assert_eq!(acc.funds_available, Decimal::new(2_00, 2));
        assert_eq!(acc.funds_total, Decimal::new(2_00, 2));

        // Simulate having held funds instead of available
        acc.funds_available = Decimal::new(0, 0);
        acc.funds_held = Decimal::new(3_05, 2);
        acc.funds_total = Decimal::new(3_05, 2);

        acc.settle_transaction(&tx, None);

        assert_eq!(acc.funds_available, Decimal::new(0, 0));
        assert_eq!(acc.funds_total, Decimal::new(3_05, 2));
    }

    #[test]
    fn settle_dispute_transaction() {
        let deposit_tx = Transaction {
            r#type: TransactionType::Deposit,
            client_id: 1,
            tx_id: 1,
            amount: Some(Decimal::new(500, 0)),
        };

        let dispute_tx = Transaction {
            r#type: TransactionType::Dispute,
            client_id: 1,
            tx_id: 1,
            amount: None,
        };

        let mut acc = Account::new(deposit_tx.client_id);

        acc.settle_transaction(&deposit_tx, None);

        assert_eq!(acc.funds_available, Decimal::new(500, 0));
        assert_eq!(acc.funds_total, Decimal::new(500, 0));

        acc.settle_transaction(&dispute_tx, Some(&deposit_tx));

        assert_eq!(acc.funds_available, Decimal::new(0, 0));
        assert_eq!(acc.funds_held, Decimal::new(500, 0));
        assert_eq!(acc.funds_total, Decimal::new(500, 0));
    }

    #[test]
    fn settle_resolve_transaction() {
        let deposit_tx = Transaction {
            r#type: TransactionType::Deposit,
            client_id: 1,
            tx_id: 1,
            amount: Some(Decimal::new(500, 0)),
        };

        let dispute_tx = Transaction {
            r#type: TransactionType::Dispute,
            client_id: 1,
            tx_id: 1,
            amount: None,
        };

        let resolve_tx = Transaction {
            r#type: TransactionType::Resolve,
            client_id: 1,
            tx_id: 1,
            amount: None,
        };

        let mut acc = Account::new(deposit_tx.client_id);

        acc.settle_transaction(&deposit_tx, None);

        assert_eq!(acc.funds_available, Decimal::new(500, 0));
        assert_eq!(acc.funds_total, Decimal::new(500, 0));

        acc.settle_transaction(&dispute_tx, Some(&deposit_tx));

        assert_eq!(acc.funds_available, Decimal::new(0, 0));
        assert_eq!(acc.funds_held, Decimal::new(500, 0));
        assert_eq!(acc.funds_total, Decimal::new(500, 0));

        acc.settle_transaction(&resolve_tx, Some(&deposit_tx));

        assert_eq!(acc.funds_available, Decimal::new(500, 0));
        assert_eq!(acc.funds_held, Decimal::new(0, 0));
        assert_eq!(acc.funds_total, Decimal::new(500, 0));
    }

    #[test]
    fn settle_chargeback_transaction() {
        let deposit_tx = Transaction {
            r#type: TransactionType::Deposit,
            client_id: 1,
            tx_id: 1,
            amount: Some(Decimal::new(500, 0)),
        };

        let dispute_tx = Transaction {
            r#type: TransactionType::Dispute,
            client_id: 1,
            tx_id: 1,
            amount: None,
        };

        let chargeback_tx = Transaction {
            r#type: TransactionType::Chargeback,
            client_id: 1,
            tx_id: 1,
            amount: None,
        };

        let mut acc = Account::new(deposit_tx.client_id);

        acc.settle_transaction(&deposit_tx, None);

        assert_eq!(acc.funds_available, Decimal::new(500, 0));
        assert_eq!(acc.funds_total, Decimal::new(500, 0));

        acc.settle_transaction(&dispute_tx, Some(&deposit_tx));

        assert_eq!(acc.funds_available, Decimal::new(0, 0));
        assert_eq!(acc.funds_held, Decimal::new(500, 0));
        assert_eq!(acc.funds_total, Decimal::new(500, 0));

        acc.settle_transaction(&chargeback_tx, Some(&deposit_tx));

        assert_eq!(acc.funds_available, Decimal::new(0, 0));
        assert_eq!(acc.funds_held, Decimal::new(0, 0));
        assert_eq!(acc.funds_total, Decimal::new(0, 0));
        assert_eq!(acc.locked, true);
    }
}
