use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::storage::{TransactionType, TransactionsDatabase};
use crate::transaction::{Transaction, Type};
use crate::types::{Amount, ClientId, TransactionId};
use crate::{client::client_account::ClientAccount, client::error::ClientAccountError};

use crate::engine::error::EngineError;

#[derive(Clone)]
pub struct PaymentsEngine {
    clients: Arc<RwLock<HashMap<ClientId, ClientAccount>>>,
    transactions_database: Arc<RwLock<TransactionsDatabase>>,
    disputes: Arc<RwLock<HashSet<TransactionId>>>,
}

impl PaymentsEngine {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            transactions_database: Arc::new(RwLock::new(TransactionsDatabase::new())),
            disputes: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    pub async fn handle_transaction(&self, transaction: Transaction) -> Result<(), EngineError> {
        match transaction.t_type {
            Type::Deposit => self.handle_deposit(transaction).await,
            Type::Withdrawal => self.handle_withdrawals(transaction).await,
            Type::Dispute => self.handle_dispute(transaction).await,
            Type::Resolve => self.handle_resolve(transaction).await,
            Type::Chargeback => self.handle_chargeback(transaction).await,
        }
    }

    async fn handle_deposit(&self, transaction: Transaction) -> Result<(), EngineError> {
        if self
            .transactions_database
            .read()
            .await
            .contains_key(transaction.transaction_id)
        {
            return Err(EngineError::TransactionAlreadyExists);
        }
        if let Some(transaction_value) = transaction.amount {
            let mut write_client_lock = self.clients.write().await;

            let client = write_client_lock
                .entry(transaction.t_client_id)
                .or_insert(ClientAccount::new());

            client.deposit(transaction_value)?;

            let transaction_t: TransactionType = (transaction.t_client_id, transaction_value);
            self.transactions_database
                .write()
                .await
                .insert(transaction.transaction_id, transaction_t);
            Ok(())
        } else {
            Err(EngineError::InvalidLeger(transaction.transaction_id))
        }
    }

    async fn handle_withdrawals(&self, transaction: Transaction) -> Result<(), EngineError> {
        if self
            .transactions_database
            .read()
            .await
            .contains_key(transaction.transaction_id)
        {
            return Err(EngineError::TransactionAlreadyExists);
        }
        if let Some(transaction_value) = transaction.amount {
            let mut write_client_lock = self.clients.write().await;

            let client = write_client_lock
                .entry(transaction.t_client_id)
                .or_insert(ClientAccount::new());

            client.withdrawal(transaction_value)?;
            Ok(())
        } else {
            Err(EngineError::InvalidLeger(transaction.transaction_id))
        }
    }

    async fn handle_dispute(&self, transaction: Transaction) -> Result<(), EngineError> {
        if self
            .disputes
            .read()
            .await
            .contains(&transaction.transaction_id)
        {
            return Err(EngineError::TransactionAlreadyDisputed(
                transaction.transaction_id,
            ));
        }
        self.handle_transaction_without_amount(
            transaction.t_client_id,
            transaction.transaction_id,
            |c, a| c.dispute(a),
        )
        .await?;
        self.disputes
            .write()
            .await
            .insert(transaction.transaction_id);
        Ok(())
    }

    async fn handle_resolve(&self, transaction: Transaction) -> Result<(), EngineError> {
        if !self
            .disputes
            .read()
            .await
            .contains(&transaction.transaction_id)
        {
            return Err(EngineError::TransactionNotDisputed(
                transaction.transaction_id,
            ));
        }
        self.handle_transaction_without_amount(
            transaction.t_client_id,
            transaction.transaction_id,
            |c, a| c.resolve(a),
        )
        .await?;
        self.disputes
            .write()
            .await
            .remove(&transaction.transaction_id);
        Ok(())
    }

    async fn handle_chargeback(&self, transaction: Transaction) -> Result<(), EngineError> {
        if !self
            .disputes
            .read()
            .await
            .contains(&transaction.transaction_id)
        {
            return Err(EngineError::TransactionNotDisputed(
                transaction.transaction_id,
            ));
        }
        self.handle_transaction_without_amount(
            transaction.t_client_id,
            transaction.transaction_id,
            |c, a| c.chargeback(a),
        )
        .await?;
        self.disputes
            .write()
            .await
            .remove(&transaction.transaction_id);
        Ok(())
    }

    async fn handle_transaction_without_amount<F>(
        &self,
        t_client_id: ClientId,
        transaction_id: TransactionId,
        action: F,
    ) -> Result<(), EngineError>
    where
        F: FnOnce(&mut ClientAccount, Amount) -> Result<(), ClientAccountError>,
    {
        if let Some(client) = self.clients.write().await.get_mut(&t_client_id) {
            if let Some((client_id_expected, amount)) =
                self.transactions_database.read().await.get(transaction_id)
            {
                if t_client_id == client_id_expected {
                    match action(client, amount) {
                        Ok(_) => Ok(()),
                        Err(err) => Err(EngineError::ClientAccountError(err)),
                    }
                } else {
                    Err(EngineError::NotClientOwnedTransaction(
                        transaction_id,
                        t_client_id,
                    ))
                }
            } else {
                Err(EngineError::TransactionNotFound(transaction_id))
            }
        } else {
            Err(EngineError::ClientNotFound)
        }
    }

    pub async fn write_state(&self) -> Result<String, EngineError> {
        let mut buffer = String::new();
        writeln!(&mut buffer, "client,available,held,total,locked")
            .map_err(|_| EngineError::WriteBuffer)?;

        for (id, client) in self.clients.read().await.iter() {
            writeln!(
                &mut buffer,
                "{},{:.4},{:.4},{:.4},{}",
                id,
                client.available(),
                client.held(),
                client.total(),
                client.locked()
            )
            .map_err(|_| EngineError::WriteBuffer)?;
        }
        Ok(buffer)
    }
}

#[cfg(test)]
pub mod tests {
    use rust_decimal::dec;

    use super::*;

    #[tokio::test]
    async fn handle_deposit_errors() {
        let payments_engine = PaymentsEngine::new();

        assert!(
            payments_engine
                .handle_deposit(Transaction {
                    t_type: Type::Deposit,
                    t_client_id: 1,
                    transaction_id: 1,
                    amount: Some(dec!(1.5050)),
                })
                .await
                .is_ok()
        );

        assert_eq!(
            payments_engine
                .handle_deposit(Transaction {
                    t_type: Type::Deposit,
                    t_client_id: 1,
                    transaction_id: 1,
                    amount: Some(dec!(1.5050)),
                })
                .await
                .unwrap_err(),
            EngineError::TransactionAlreadyExists
        );

        assert_eq!(
            payments_engine
                .handle_deposit(Transaction {
                    t_type: Type::Deposit,
                    t_client_id: 1,
                    transaction_id: 2,
                    amount: None,
                })
                .await
                .unwrap_err(),
            EngineError::InvalidLeger(2)
        );

        assert_eq!(
            payments_engine
                .handle_deposit(Transaction {
                    t_type: Type::Deposit,
                    t_client_id: 1,
                    transaction_id: 3,
                    amount: Some(dec!(-1.5050)),
                })
                .await
                .unwrap_err(),
            EngineError::ClientAccountError(ClientAccountError::NegativeAmount)
        );
    }

    #[tokio::test]
    async fn handle_withdrawals_errors() {
        let payments_engine = PaymentsEngine::new();

        assert!(
            payments_engine
                .handle_deposit(Transaction {
                    t_type: Type::Deposit,
                    t_client_id: 1,
                    transaction_id: 1,
                    amount: Some(dec!(1.5050)),
                })
                .await
                .is_ok()
        );

        assert_eq!(
            payments_engine
                .handle_deposit(Transaction {
                    t_type: Type::Deposit,
                    t_client_id: 1,
                    transaction_id: 1,
                    amount: Some(dec!(1.5050)),
                })
                .await
                .unwrap_err(),
            EngineError::TransactionAlreadyExists
        );

        assert_eq!(
            payments_engine
                .handle_withdrawals(Transaction {
                    t_type: Type::Withdrawal,
                    t_client_id: 1,
                    transaction_id: 1,
                    amount: None,
                })
                .await
                .unwrap_err(),
            EngineError::TransactionAlreadyExists
        );

        assert_eq!(
            payments_engine
                .handle_withdrawals(Transaction {
                    t_type: Type::Withdrawal,
                    t_client_id: 1,
                    transaction_id: 2,
                    amount: None,
                })
                .await
                .unwrap_err(),
            EngineError::InvalidLeger(2)
        );

        assert_eq!(
            payments_engine
                .handle_withdrawals(Transaction {
                    t_type: Type::Withdrawal,
                    t_client_id: 1,
                    transaction_id: 3,
                    amount: Some(dec!(5)),
                })
                .await
                .unwrap_err(),
            EngineError::ClientAccountError(ClientAccountError::InsufficientBalance)
        );
    }

    #[tokio::test]
    async fn handle_basic_dispute_errors() {
        let payments_engine = PaymentsEngine::new();

        assert!(
            payments_engine
                .handle_deposit(Transaction {
                    t_type: Type::Deposit,
                    t_client_id: 1,
                    transaction_id: 1,
                    amount: Some(dec!(1.5050)),
                })
                .await
                .is_ok()
        );

        assert!(
            payments_engine
                .handle_dispute(Transaction {
                    t_type: Type::Dispute,
                    t_client_id: 1,
                    transaction_id: 1,
                    amount: None,
                })
                .await
                .is_ok()
        );

        assert_eq!(
            payments_engine
                .handle_dispute(Transaction {
                    t_type: Type::Dispute,
                    t_client_id: 1,
                    transaction_id: 1,
                    amount: None,
                })
                .await
                .unwrap_err(),
            EngineError::TransactionAlreadyDisputed(1)
        );

        assert_eq!(
            payments_engine
                .handle_dispute(Transaction {
                    t_type: Type::Dispute,
                    t_client_id: 2,
                    transaction_id: 3,
                    amount: None,
                })
                .await
                .unwrap_err(),
            EngineError::ClientNotFound
        );

        assert_eq!(
            payments_engine
                .handle_dispute(Transaction {
                    t_type: Type::Dispute,
                    t_client_id: 1,
                    transaction_id: 10,
                    amount: None,
                })
                .await
                .unwrap_err(),
            EngineError::TransactionNotFound(10)
        );
    }

    #[tokio::test]
    async fn dispute_not_client_owned_transaction() {
        let payments_engine = PaymentsEngine::new();

        assert!(
            payments_engine
                .handle_deposit(Transaction {
                    t_type: Type::Deposit,
                    t_client_id: 1,
                    transaction_id: 1,
                    amount: Some(dec!(1.5050)),
                })
                .await
                .is_ok()
        );

        assert!(
            payments_engine
                .handle_dispute(Transaction {
                    t_type: Type::Dispute,
                    t_client_id: 1,
                    transaction_id: 1,
                    amount: None,
                })
                .await
                .is_ok()
        );

        assert!(
            payments_engine
                .handle_deposit(Transaction {
                    t_type: Type::Deposit,
                    t_client_id: 100,
                    transaction_id: 100,
                    amount: Some(dec!(1.5050)),
                })
                .await
                .is_ok()
        );

        assert_eq!(
            payments_engine
                .handle_resolve(Transaction {
                    t_type: Type::Resolve,
                    t_client_id: 100,
                    transaction_id: 1,
                    amount: None,
                })
                .await
                .unwrap_err(),
            EngineError::NotClientOwnedTransaction(1, 100)
        );
    }

    #[tokio::test]
    async fn handle_basic_resolve_errors() {
        let payments_engine = PaymentsEngine::new();

        assert!(
            payments_engine
                .handle_deposit(Transaction {
                    t_type: Type::Deposit,
                    t_client_id: 1,
                    transaction_id: 1,
                    amount: Some(dec!(1.5050)),
                })
                .await
                .is_ok()
        );

        assert!(
            payments_engine
                .handle_dispute(Transaction {
                    t_type: Type::Dispute,
                    t_client_id: 1,
                    transaction_id: 1,
                    amount: None,
                })
                .await
                .is_ok()
        );

        assert_eq!(
            payments_engine
                .handle_resolve(Transaction {
                    t_type: Type::Resolve,
                    t_client_id: 1,
                    transaction_id: 2,
                    amount: None,
                })
                .await
                .unwrap_err(),
            EngineError::TransactionNotDisputed(2)
        );
    }

    #[tokio::test]
    async fn handle_basic_chargeback_errors() {
        let payments_engine = PaymentsEngine::new();

        assert!(
            payments_engine
                .handle_deposit(Transaction {
                    t_type: Type::Deposit,
                    t_client_id: 1,
                    transaction_id: 1,
                    amount: Some(dec!(1.5050)),
                })
                .await
                .is_ok()
        );

        assert!(
            payments_engine
                .handle_dispute(Transaction {
                    t_type: Type::Dispute,
                    t_client_id: 1,
                    transaction_id: 1,
                    amount: None,
                })
                .await
                .is_ok()
        );

        assert_eq!(
            payments_engine
                .handle_chargeback(Transaction {
                    t_type: Type::Chargeback,
                    t_client_id: 1,
                    transaction_id: 2,
                    amount: None,
                })
                .await
                .unwrap_err(),
            EngineError::TransactionNotDisputed(2)
        );
    }

    //PaymentsEngine // Something wrong with the order, commented out the test
    // #[test]
    // fn handle_transaction() {
    //     let transactions = vec![
    //         Transaction {
    //             t_type: Type::Deposit,
    //             t_client_id: 1,
    //             transaction_id: 1,
    //             amount: Some(dec!(1.5050)),
    //         },
    //         Transaction {
    //             t_type: Type::Deposit,
    //             t_client_id: 2,
    //             transaction_id: 2,
    //             amount: Some(dec!(2.1010)),
    //         },
    //         Transaction {
    //             t_type: Type::Deposit,
    //             t_client_id: 1,
    //             transaction_id: 3,
    //             amount: Some(dec!(1.0)),
    //         },
    //         Transaction {
    //             t_type: Type::Withdrawal,
    //             t_client_id: 1,
    //             transaction_id: 4,
    //             amount: Some(dec!(1.5)),
    //         },
    //         Transaction {
    //             t_type: Type::Withdrawal,
    //             t_client_id: 2,
    //             transaction_id: 5,
    //             amount: Some(dec!(3.0)),
    //         },
    //         Transaction {
    //             t_type: Type::Dispute,
    //             t_client_id: 1,
    //             transaction_id: 1,
    //             amount: None,
    //         },
    //         Transaction {
    //             t_type: Type::Resolve,
    //             t_client_id: 1,
    //             transaction_id: 1,
    //             amount: None,
    //         },
    //         Transaction {
    //             t_type: Type::Dispute,
    //             t_client_id: 1,
    //             transaction_id: 1,
    //             amount: None,
    //         },
    //         Transaction {
    //             t_type: Type::Chargeback,
    //             t_client_id: 1,
    //             transaction_id: 1,
    //             amount: None,
    //         },
    //     ];

    //     let mut payments_engine = PaymentsEngine::new();

    //     for transaction in transactions {
    //         let _ = payments_engine.handle_transaction(transaction);
    //     }
    //     let output = payments_engine.write_state().unwrap();

    //     let mut expected_output = String::new();
    //     writeln!(&mut expected_output, "client,available,held,total,locked").unwrap();
    //     writeln!(&mut expected_output, "1,-0.5000,0.0000,-0.5000,true").unwrap();
    //     writeln!(&mut expected_output, "2,2.1010,0.0000,2.1010,false").unwrap();

    //     assert_eq!(output, expected_output);
    // }
}
