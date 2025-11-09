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
            Type::Deposit => match transaction.amount {
                Some(transaction_value) => {
                    self.handle_transaction_with_amount(
                        transaction.t_client_id,
                        transaction.transaction_id,
                        transaction_value.round_dp(4),
                        |c, a| c.deposit(a),
                    );
                }
                None => {}
            },
            Type::Withdrawal => match transaction.amount {
                Some(transaction_value) => {
                    self.handle_transaction_with_amount(
                        transaction.t_client_id,
                        transaction.transaction_id,
                        transaction_value.round_dp(4),
                        |c, a| c.withdrawal(a),
                    );
                }
                None => {}
            },
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

    fn handle_transaction_with_amount<F>(
        &mut self,
        t_client_id: ClientId,
        transaction_id: TransactionId,
        amount: Amount,
        action: F,
    ) where
        F: FnOnce(&mut Client, Amount),
    {
        let client = self.clients.entry(t_client_id).or_insert(Client::new());
        if !client.locked() {
            action(client, amount);
            self.transactions_database
                .insert_transaction(t_client_id, transaction_id, amount);
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
        match self
            .transactions_database
            .read_transaction_amount(t_client_id, transaction_id)
        {
            Ok(amount) => {
                let client = self.clients.get_mut(&t_client_id).unwrap();
                action(client, amount);
            }
            Err(_) => {}
        }
    }

    pub fn write_state(&self) -> String {
        let mut buffer = String::new();
        writeln!(&mut buffer, "client,available,held,total,locked").unwrap();

        for (id, client) in &self.clients {
            writeln!(
                &mut buffer,
                "{},{},{},{},{}",
                id,
                client.available(),
                client.held(),
                client.total(),
                client.locked()
            )
            .unwrap();
        }
        buffer
    }
}

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

    //PaymentsEngine
    #[test]
    fn handle_transaction() {
        let csv_data = "\
                                deposit,1,1,1.0\n\
                                deposit,2,2,2.0\n\
                                deposit,1,3,2.0\n\
                                withdrawal,1,4,1.5\n\
                                withdrawal,2,5,3.0";
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(csv_data.as_bytes()); // error deserializing order

        let iter = rdr.deserialize::<Transaction>();

        let mut payments_engine = PaymentsEngine::new();

        for transaction_result in iter {
            let transaction: Transaction = transaction_result.unwrap();
            payments_engine.handle_transaction(transaction);
        }
        let output = payments_engine.write_state();

        let mut expected_output = String::new();
        writeln!(&mut expected_output, "client,available,held,total,locked").unwrap();
        writeln!(&mut expected_output, "1,1.5,0,1.5,false").unwrap();
        writeln!(&mut expected_output, "2,2,0,2,false").unwrap();

        assert_eq!(output, expected_output);
    }
}
