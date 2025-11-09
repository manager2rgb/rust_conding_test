use crate::types::{Amount, ClientId, TransactionId};
use std::collections::HashMap;

pub type TransactionType = (ClientId, Amount);

pub struct TransactionsDatabase {
    transactions: HashMap<TransactionId, TransactionType>,
}

impl TransactionsDatabase {
    pub fn new() -> Self {
        Self {
            transactions: HashMap::new(),
        }
    }

    pub fn insert(&mut self, transaction_id: TransactionId, transaction: TransactionType) {
        self.transactions.insert(transaction_id, transaction);
    }

    pub fn get(&self, transaction_id: TransactionId) -> Option<TransactionType> {
        self.transactions.get(&transaction_id).copied()
    }

    pub fn contains_key(&self, transaction_id: TransactionId) -> bool {
        self.transactions.contains_key(&transaction_id)
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

        let transaction: TransactionType = (t_client_id, amount);

        transactions_database.insert(transaction_id, transaction);

        let received_amout = transactions_database.get(transaction_id);

        assert!(received_amout.is_some());
        assert_eq!(received_amout.unwrap(), transaction);
    }

    #[test]
    fn error_transaction_database() {
        let t_client_id = 1;
        let transaction_id = 1;
        let amount = dec!(1.000);

        let mut transactions_database = TransactionsDatabase::new();

        let transaction: TransactionType = (t_client_id, amount);

        transactions_database.insert(transaction_id, transaction);

        let received_amout = transactions_database.get(100);

        assert!(received_amout.is_none());
    }
}
