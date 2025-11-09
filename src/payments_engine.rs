use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::io::ErrorKind;

use crate::client::Client;
use crate::transaction::{Transaction, Type};
use crate::types::{Amount, ClientId, TransactionId};

struct TransactionsDatabase {
    transactions: HashMap<ClientId, HashMap<TransactionId, Amount>>,
}

impl TransactionsDatabase {
    pub fn new() -> Self {
        Self {
            transactions: HashMap::new(),
        }
    }

    pub fn insert_transaction(
        &mut self,
        t_client_id: ClientId,
        transaction_id: TransactionId,
        amount: Amount,
    ) {
        let transaction = self.transactions.entry(t_client_id).or_default();
        transaction.insert(transaction_id, amount);
    }

    pub fn read_transaction_amount(
        &mut self,
        t_client_id: ClientId,
        transaction_id: TransactionId,
    ) -> Result<Amount, ErrorKind> {
        let client_transactions = self.transactions.get(&t_client_id);
        match client_transactions {
            Some(transactions) => match transactions.get(&transaction_id) {
                Some(amount) => Ok(*amount),
                None => Err(ErrorKind::NotFound),
            },
            None => Err(ErrorKind::NotFound),
        }
    }
}

pub struct PaymentsEngine {
    clients: HashMap<ClientId, Client>,
    transactions_database: TransactionsDatabase,
    disputes: HashSet<TransactionId>,
}

impl PaymentsEngine {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
            transactions_database: TransactionsDatabase::new(),
            disputes: HashSet::new(),
        }
    }

    pub fn handle_transaction(&mut self, transaction: Transaction) {
        match transaction.t_type {
            Type::Deposit => {
                if let Some(transaction_value) = transaction.amount {
                    let client = self
                        .clients
                        .entry(transaction.t_client_id)
                        .or_insert(Client::new());
                    if !client.locked() {
                        client.deposit(transaction_value);
                        self.transactions_database.insert_transaction(
                            transaction.t_client_id,
                            transaction.transaction_id,
                            transaction_value,
                        );
                    }
                }
            }
            Type::Withdrawal => {
                if let Some(transaction_value) = transaction.amount {
                    let client = self
                        .clients
                        .entry(transaction.t_client_id)
                        .or_insert(Client::new());
                    if !client.locked() {
                        client.withdrawal(transaction_value);
                        //ASSUME ONLY DEPOSITS CAN BE DISPUTED
                        // self.transactions_database.insert_transaction(
                        //     transaction.t_client_id,
                        //     transaction.transaction_id,
                        //     transaction_value,
                        // );
                    }
                }
            }
            Type::Dispute => {
                self.handle_transaction_without_amount(
                    transaction.t_client_id,
                    transaction.transaction_id,
                    |c, a| c.dispute(a),
                );
                self.disputes.insert(transaction.transaction_id);
            }
            Type::Resolve => {
                if self.disputes.contains(&transaction.transaction_id) {
                    self.handle_transaction_without_amount(
                        transaction.t_client_id,
                        transaction.transaction_id,
                        |c, a| c.resolve(a),
                    );
                    self.disputes.remove(&transaction.transaction_id);
                }
            }
            Type::Chargeback => {
                if self.disputes.contains(&transaction.transaction_id) {
                    self.handle_transaction_without_amount(
                        transaction.t_client_id,
                        transaction.transaction_id,
                        |c, a| c.chargeback(a),
                    );
                    self.disputes.remove(&transaction.transaction_id);
                }
            }
        }
    }

    fn handle_transaction_without_amount<F>(
        &mut self,
        t_client_id: ClientId,
        transaction_id: TransactionId,
        action: F,
    ) where
        F: FnOnce(&mut Client, Amount),
    {
        if let Some(client) = self.clients.get_mut(&t_client_id)
            && let Ok(amount) = self
                .transactions_database
                .read_transaction_amount(t_client_id, transaction_id)
        {
            action(client, amount)
        }
    }

    pub fn write_state(&self) -> String {
        let mut buffer = String::new();
        let _ = writeln!(&mut buffer, "client,available,held,total,locked");

        for (id, client) in &self.clients {
            let _ = writeln!(
                &mut buffer,
                "{},{},{},{},{}",
                id,
                client.available(),
                client.held(),
                client.total(),
                client.locked()
            );
        }
        buffer
    }
}

unsafe impl Send for PaymentsEngine {}
unsafe impl Sync for PaymentsEngine {}

#[cfg(test)]
pub mod tests {
    use rust_decimal::dec;

    use super::*;

    //TransactionsDatabase
    #[test]
    fn transaction_database() {
        let t_client_id = 1;
        let transaction_id = 1;
        let amount = dec!(1.000);

        let mut transactions_database = TransactionsDatabase::new();

        transactions_database.insert_transaction(t_client_id, transaction_id, amount);

        let received_amout =
            transactions_database.read_transaction_amount(t_client_id, transaction_id);

        assert!(received_amout.is_ok());
        assert_eq!(received_amout.unwrap(), amount);
    }

    #[test]
    fn error_transaction_database() {
        let t_client_id = 1;
        let transaction_id = 1;
        let amount = dec!(1.000);

        let mut transactions_database = TransactionsDatabase::new();

        transactions_database.insert_transaction(t_client_id, transaction_id, amount);

        let received_amout = transactions_database.read_transaction_amount(t_client_id, 100);

        assert!(received_amout.is_err());
        assert_eq!(received_amout.unwrap_err(), ErrorKind::NotFound);
    }

    //PaymentsEngine // Something wrong with the order, commented out the test
    #[test]
    fn handle_transaction() {
        let transactions = vec![
            Transaction {
                t_type: Type::Deposit,
                t_client_id: 1,
                transaction_id: 1,
                amount: Some(dec!(1.5050)),
            },
            Transaction {
                t_type: Type::Deposit,
                t_client_id: 2,
                transaction_id: 2,
                amount: Some(dec!(2.1010)),
            },
            Transaction {
                t_type: Type::Deposit,
                t_client_id: 1,
                transaction_id: 3,
                amount: Some(dec!(1.0)),
            },
            Transaction {
                t_type: Type::Withdrawal,
                t_client_id: 1,
                transaction_id: 4,
                amount: Some(dec!(1.5)),
            },
            Transaction {
                t_type: Type::Withdrawal,
                t_client_id: 2,
                transaction_id: 5,
                amount: Some(dec!(3.0)),
            },
            Transaction {
                t_type: Type::Dispute,
                t_client_id: 1,
                transaction_id: 1,
                amount: None,
            },
            Transaction {
                t_type: Type::Resolve,
                t_client_id: 1,
                transaction_id: 1,
                amount: None,
            },
            Transaction {
                t_type: Type::Dispute,
                t_client_id: 1,
                transaction_id: 1,
                amount: None,
            },
            Transaction {
                t_type: Type::Chargeback,
                t_client_id: 1,
                transaction_id: 1,
                amount: None,
            },
        ];

        let mut payments_engine = PaymentsEngine::new();

        for transaction in transactions {
            payments_engine.handle_transaction(transaction);
        }
        let output = payments_engine.write_state();

        let mut expected_output = String::new();
        writeln!(&mut expected_output, "client,available,held,total,locked").unwrap();
        writeln!(&mut expected_output, "1,-0.5000,0.0000,-0.5000,true").unwrap();
        writeln!(&mut expected_output, "2,2.1010,0,2.1010,false").unwrap();

        assert_eq!(output, expected_output);
    }
}
