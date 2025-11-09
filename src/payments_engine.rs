use std::collections::HashMap;
use std::collections::hash_map::Entry;

use crate::client::Client;
use crate::transaction::{self, Transaction, Type};

type client_id = u16;
type transaction_id = u64;

type amount = f32;

struct TransactionsDatabase {
    transactions: HashMap<client_id, HashMap<transaction_id, amount>>,
    //disputes: HashMap<u16, u32>, makes this sense here????
}

impl TransactionsDatabase {
    pub fn new() -> Self {
        Self {
            transactions: HashMap::new(),
            //disputes: HashMap::new(),
        }
    }

    pub fn insert_transaction(&mut self, t_client_id: u16, transaction_id: u64, amount: f32) {
        let transaction = self
            .transactions
            .entry(t_client_id)
            .or_insert(HashMap::new());
        transaction.insert(transaction_id, amount);
    }

    pub fn read_transaction_amount(&mut self, t_client_id: u16, transaction_id: u64) -> f32 {
        self.transactions
            .get(&t_client_id)
            .unwrap()
            .get(&transaction_id)
            .unwrap()
            .clone()
    }
}

pub struct PaymentsEngine {
    clients: HashMap<client_id, Client>,
    transactions_database: TransactionsDatabase,
}

impl PaymentsEngine {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
            transactions_database: TransactionsDatabase::new(),
        }
    }

    pub async fn handle_transaction(&mut self, transaction: Transaction) {
        match transaction.t_type {
            Type::Deposit => {
                self.handle_deposit(
                    transaction.t_client_id,
                    transaction.transaction_id,
                    transaction.amount,
                );
            }
            Type::Withdrawal => {
                self.handle_withdrawal(
                    transaction.t_client_id,
                    transaction.transaction_id,
                    transaction.amount,
                );
            }
            Type::Dispute => {
                self.handle_dispute(transaction.t_client_id, transaction.transaction_id);
            }
            Type::Resolve => {
                self.handle_resolve(transaction.t_client_id, transaction.transaction_id);
            }
            Type::Chargeback => {
                self.handle_chargeback(transaction.t_client_id, transaction.transaction_id);
            }
        }
    }

    fn handle_deposit(&mut self, t_client_id: u16, transaction_id: u64, amount: f32) {
        let client = self.clients.entry(t_client_id).or_insert(Client::new());

        client.deposit(amount);

        self.transactions_database
            .insert_transaction(t_client_id, transaction_id, amount);
    }

    fn handle_withdrawal(&mut self, t_client_id: u16, transaction_id: u64, amount: f32) {
        let client = self.clients.entry(t_client_id).or_insert(Client::new());

        client.withdrawal(amount);

        self.transactions_database
            .insert_transaction(t_client_id, transaction_id, amount);
    }

    fn handle_dispute(&mut self, t_client_id: u16, transaction_id: u64) {
        let amount = self
            .transactions_database
            .read_transaction_amount(t_client_id, transaction_id);
        self.clients.get_mut(&t_client_id).unwrap().dispute(amount);
    }

    fn handle_resolve(&mut self, t_client_id: u16, transaction_id: u64) {
        let amount = self
            .transactions_database
            .read_transaction_amount(t_client_id, transaction_id);
        self.clients.get_mut(&t_client_id).unwrap().resolve(amount);
    }

    fn handle_chargeback(&mut self, t_client_id: u16, transaction_id: u64) {
        let amount = self
            .transactions_database
            .read_transaction_amount(t_client_id, transaction_id);
        self.clients
            .get_mut(&t_client_id)
            .unwrap()
            .chargeback(amount);
    }

    pub async fn write_state(&self) {
        println!("client,available,held,total,locked");
        for (id, client) in &self.clients {
            println!(
                "{},{},{},{},{}",
                id, client.available, client.held, client.total, client.locked
            );
        }
    }
}
