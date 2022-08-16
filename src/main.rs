mod account;
mod config;
mod transaction;

use std::collections::{HashMap, VecDeque};
use std::env;
use std::error::Error;
use std::io;
use std::process;

use config::Config;
use csv::{ReaderBuilder, Trim, WriterBuilder};

use crate::account::Account;
use crate::transaction::{Transaction, TransactionType};

type AccountsDB = HashMap<u16, Account>;
type TransactionsDB = HashMap<u32, Transaction>;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args);

    match process_csv(&config) {
        Ok(txs) => {
            let finalized_accounts = process_transactions(txs);

            if let Err(e) = write_output(finalized_accounts) {
                eprintln!("CSV output error: {e}");

                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("CSV processing error: {e}");

            process::exit(1);
        }
    }
}

fn process_csv(config: &Config) -> Result<VecDeque<Transaction>, Box<dyn Error>> {
    let mut unprocessed_transactions = VecDeque::<Transaction>::new();

    let mut reader = ReaderBuilder::new()
        .trim(Trim::All)
        .from_path(config.transactions_path.clone())?;

    for result in reader.deserialize() {
        let tx: Transaction = result?;
        unprocessed_transactions.push_back(tx);
    }

    Ok(unprocessed_transactions)
}

// The account and reference transaction data stores are created inside this function for ease-of-use
// In a real-world system, connections to these external data sources would be passed in via
// parameters if needed
fn process_transactions(mut unprocessed_transactions: VecDeque<Transaction>) -> AccountsDB {
    let mut accounts = AccountsDB::new();
    let mut ref_txs = TransactionsDB::new();

    while !unprocessed_transactions.is_empty() {
        let tx = unprocessed_transactions
            .pop_front()
            .expect("transaction should exist");

        let acc = accounts
            .entry(tx.client_id)
            .or_insert_with(|| Account::new(tx.client_id));

        if tx.r#type == TransactionType::Deposit || tx.r#type == TransactionType::Withdrawal {
            acc.settle_transaction(&tx, None);
            ref_txs.insert(tx.tx_id, tx);
        } else if let Some(ref_tx) = ref_txs.get(&tx.tx_id) {
            acc.settle_transaction(&tx, Some(ref_tx));
        } else {
            unprocessed_transactions.push_back(tx);
        }
    }

    accounts
}

fn write_output(accounts: AccountsDB) -> Result<(), Box<dyn Error>> {
    let mut writer = WriterBuilder::new().from_writer(io::stdout());

    for (_, acc) in accounts {
        writer.serialize(acc)?;
    }

    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use rust_decimal::Decimal;

    use crate::{
        process_transactions,
        transaction::{Transaction, TransactionType},
    };

    #[test]
    fn process_basic_transactions() {
        let deposit1 = Transaction {
            r#type: TransactionType::Deposit,
            client_id: 1,
            tx_id: 1,
            amount: Some(Decimal::new(10, 1)),
        };

        let deposit2 = Transaction {
            r#type: TransactionType::Deposit,
            client_id: 2,
            tx_id: 2,
            amount: Some(Decimal::new(20, 1)),
        };

        let deposit3 = Transaction {
            r#type: TransactionType::Deposit,
            client_id: 1,
            tx_id: 3,
            amount: Some(Decimal::new(20, 1)),
        };

        let withdrawal1 = Transaction {
            r#type: TransactionType::Withdrawal,
            client_id: 1,
            tx_id: 4,
            amount: Some(Decimal::new(15, 1)),
        };

        let withdrawal2 = Transaction {
            r#type: TransactionType::Withdrawal,
            client_id: 2,
            tx_id: 5,
            amount: Some(Decimal::new(30, 1)),
        };

        let unprocessed_transactions =
            VecDeque::<Transaction>::from([deposit1, deposit2, deposit3, withdrawal1, withdrawal2]);

        let finalized_accounts = process_transactions(unprocessed_transactions);

        let client1 = finalized_accounts
            .get(&1)
            .expect("Client 1 should exist in finalized accounts");

        assert_eq!(client1.funds_available, Decimal::new(15, 1));
        assert_eq!(client1.funds_held, Decimal::new(0, 0));
        assert_eq!(client1.funds_total, Decimal::new(15, 1));
        assert_eq!(client1.locked, false);

        let client2 = finalized_accounts
            .get(&2)
            .expect("Client 2 should exist in finalized accounts");

        assert_eq!(client2.funds_available, Decimal::new(2, 0));
        assert_eq!(client2.funds_held, Decimal::new(0, 0));
        assert_eq!(client2.funds_total, Decimal::new(2, 0));
        assert_eq!(client2.locked, false);
    }

    #[test]
    fn process_complex_transactions() {
        let deposit1 = Transaction {
            r#type: TransactionType::Deposit,
            client_id: 1,
            tx_id: 1,
            amount: Some(Decimal::new(500_0005, 4)),
        };

        let deposit2 = Transaction {
            r#type: TransactionType::Deposit,
            client_id: 1,
            tx_id: 2,
            amount: Some(Decimal::new(1000, 0)),
        };

        let dispute1 = Transaction {
            r#type: TransactionType::Dispute,
            client_id: 1,
            tx_id: 1,
            amount: None,
        };

        let resolve1 = Transaction {
            r#type: TransactionType::Resolve,
            client_id: 1,
            tx_id: 1,
            amount: None,
        };

        let deposit3 = Transaction {
            r#type: TransactionType::Deposit,
            client_id: 1,
            tx_id: 3,
            amount: Some(Decimal::new(100, 0)),
        };

        let dispute2 = Transaction {
            r#type: TransactionType::Dispute,
            client_id: 1,
            tx_id: 3,
            amount: None,
        };

        let chargeback1 = Transaction {
            r#type: TransactionType::Chargeback,
            client_id: 1,
            tx_id: 3,
            amount: None,
        };

        let txs = VecDeque::<Transaction>::from([
            deposit1,
            deposit2,
            dispute1,
            resolve1,
            deposit3,
            dispute2,
            chargeback1,
        ]);

        let finalized_accounts = process_transactions(txs);

        let client1 = finalized_accounts
            .get(&1)
            .expect("Client 1 should exist in finalized accounts");

        assert_eq!(client1.funds_available, Decimal::new(1500_0005, 4));
        assert_eq!(client1.funds_held, Decimal::new(0, 0));
        assert_eq!(client1.funds_total, Decimal::new(1500_0005, 4));
        assert_eq!(client1.locked, true);
    }
}
