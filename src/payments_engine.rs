use std::collections::{HashMap, HashSet};
use std::io::ErrorKind;

use crate::client::Client;
use crate::transaction::{self, Transaction, Type};
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
    ) -> Result<Option<Amount>, ErrorKind> {
        let transaction = self.transactions.get(&t_client_id);
        match transaction {
            Some(transaction) => Ok(transaction.get(&transaction_id).copied()),
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

    pub async fn handle_transaction(&mut self, transaction: Transaction) {
        match transaction.t_type {
            Type::Deposit => match transaction.amount {
                Some(transaction_value) => {
                    self.handle_deposit(
                        transaction.t_client_id,
                        transaction.transaction_id,
                        transaction_value.round_dp(4),
                    );
                }
                None => {}
            },
            Type::Withdrawal => match transaction.amount {
                Some(transaction_value) => {
                    self.handle_withdrawal(
                        transaction.t_client_id,
                        transaction.transaction_id,
                        transaction_value.round_dp(4),
                    );
                }
                None => {}
            },
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

    fn handle_deposit(
        &mut self,
        t_client_id: ClientId,
        transaction_id: TransactionId,
        amount: Amount,
    ) {
        let client = self.clients.entry(t_client_id).or_insert(Client::new());

        if !client.locked() {
            client.deposit(amount);

            self.transactions_database
                .insert_transaction(t_client_id, transaction_id, amount);
        }
    }

    fn handle_withdrawal(
        &mut self,
        t_client_id: ClientId,
        transaction_id: TransactionId,
        amount: Amount,
    ) {
        let client = self.clients.entry(t_client_id).or_insert(Client::new());

        if !client.locked() {
            client.withdrawal(amount);

            self.transactions_database
                .insert_transaction(t_client_id, transaction_id, amount);
        }
    }

    fn handle_dispute(&mut self, t_client_id: ClientId, transaction_id: TransactionId) {
        match self
            .transactions_database
            .read_transaction_amount(t_client_id, transaction_id)
        {
            Ok(Some(amount)) => {
                let client = self.clients.get_mut(&t_client_id).unwrap();
                client.dispute(amount);
                self.disputes.insert(transaction_id);
            }
            Ok(None) => {}
            Err(_) => {}
        }
    }

    fn handle_resolve(&mut self, t_client_id: ClientId, transaction_id: TransactionId) {
        if self.disputes.contains(&transaction_id) {
            match self
                .transactions_database
                .read_transaction_amount(t_client_id, transaction_id)
            {
                Ok(Some(amount)) => {
                    let client = self.clients.get_mut(&t_client_id).unwrap();
                    client.resolve(amount);
                    self.disputes.remove(&transaction_id);
                }
                Ok(None) => {}
                Err(_) => {}
            }
        }
    }

    fn handle_chargeback(&mut self, t_client_id: ClientId, transaction_id: TransactionId) {
        if self.disputes.contains(&transaction_id) {
            match self
                .transactions_database
                .read_transaction_amount(t_client_id, transaction_id)
            {
                Ok(Some(amount)) => {
                    let client = self.clients.get_mut(&t_client_id).unwrap();
                    client.chargeback(amount);
                    self.disputes.remove(&transaction_id);
                }
                Ok(None) => {}
                Err(_) => {}
            }
        }
    }

    pub async fn write_state(&self) {
        println!("client,available,held,total,locked");
        for (id, client) in &self.clients {
            println!(
                "{},{},{},{},{}",
                id,
                client.available(),
                client.held(),
                client.total(),
                client.locked()
            );
        }
    }
}
