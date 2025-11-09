use crate::types::{Amount, ClientId, TransactionId};
use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize, PartialEq)]
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

#[derive(Debug, Deserialize, PartialEq)]
pub struct Transaction {
    #[serde(rename = "type")]
    pub t_type: Type,
    #[serde(rename = "client")]
    pub t_client_id: ClientId,
    #[serde(rename = "tx")]
    pub transaction_id: TransactionId,
    #[serde(default)]
    #[serde(deserialize_with = "de_decimal_non_negative")]
    pub amount: Option<Amount>,
}

fn de_decimal_non_negative<'de, D>(deserializer: D) -> Result<Option<Amount>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<Amount>::deserialize(deserializer)?;

    if let Some(v) = opt {
        if v.is_sign_negative() {
            return Err(serde::de::Error::custom("amount must be non-negative"));
        }
        Ok(Some(v))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
pub mod tests {

    use csv::Position;
    use rust_decimal::dec;

    use super::*;

    #[test]
    fn read_desposit() {
        let csv_data = "deposit,1,1,10.50\n";
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(csv_data.as_bytes());

        // deposit,1,1,10.50
        let expected = Transaction {
            t_type: Type::Deposit,
            t_client_id: 1,
            transaction_id: 1,
            amount: Some(dec!(10.50)),
        };

        let transaction = rdr.deserialize::<Transaction>().next().unwrap();
        assert!(transaction.is_ok());
        assert_eq!(expected, transaction.unwrap());
    }

    #[test]
    fn read_desposit_4_decimal() {
        let csv_data = "deposit,1,2,10.5555\n";

        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(csv_data.as_bytes());

        // deposit,1,1,10.50
        let expected = Transaction {
            t_type: Type::Deposit,
            t_client_id: 1,
            transaction_id: 2,
            amount: Some(dec!(10.5555)),
        };

        let transaction = rdr.deserialize::<Transaction>().next().unwrap();
        assert!(transaction.is_ok());
        assert_eq!(expected, transaction.unwrap());
    }

    #[test]
    fn read_desposit_more_4_decimal() {
        let csv_data = "deposit,1,2,10.55557\n";

        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(csv_data.as_bytes());

        let expected_amount_4_decimal = dec!(10.5556);
        let transaction = rdr.deserialize::<Transaction>().next().unwrap();
        assert!(transaction.is_ok());

        let amount_4_decimal = transaction.unwrap().amount.unwrap().round_dp(4);

        assert_eq!(expected_amount_4_decimal, amount_4_decimal);
    }

    #[test]
    fn read_wrong_type_transaction() {
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader("test,1,1,1.1\n".as_bytes());

        let transaction = rdr.deserialize::<Transaction>().next().unwrap();

        assert!(transaction.is_err());
        let transaction_error = transaction.unwrap_err();
        println!(
            "Erro while deserializing transaction: {} ",
            transaction_error
        );
        let expected_error_kind_position = &Position::new();
        let postition = transaction_error.kind().position().unwrap();
        assert_eq!(postition, expected_error_kind_position);
    }

    #[test]
    fn read_wrong_deposit_transaction() {
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader("deposit,1,-1,1.1\n".as_bytes());

        let transaction = rdr.deserialize::<Transaction>().next().unwrap();

        assert!(transaction.is_err());
        let transaction_error = transaction.unwrap_err();
        println!(
            "Erro while deserializing transaction: {} ",
            transaction_error
        );
        let expected_error_kind_position = &Position::new();
        let postition = transaction_error.kind().position().unwrap();
        assert_eq!(postition, expected_error_kind_position);
    }

    #[test]
    fn read_withdrawal_transaction() {
        let csv_data = "withdrawal,1,100,10.50\n";

        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(csv_data.as_bytes());

        // deposit,1,1,10.50
        let expected = Transaction {
            t_type: Type::Withdrawal,
            t_client_id: 1,
            transaction_id: 100,
            amount: Some(dec!(10.50)),
        };

        let transaction = rdr.deserialize::<Transaction>().next().unwrap();
        assert!(transaction.is_ok());
        assert_eq!(expected, transaction.unwrap());
    }

    #[test]
    fn read_resolve_transaction() {
        let csv_data = "resolve,1,100\n";

        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(csv_data.as_bytes());

        // deposit,1,1,10.50
        let expected = Transaction {
            t_type: Type::Resolve,
            t_client_id: 1,
            transaction_id: 100,
            amount: None,
        };

        let transaction = rdr.deserialize::<Transaction>().next().unwrap();
        assert!(transaction.is_ok());
        assert_eq!(expected, transaction.unwrap());
    }

    #[test]
    fn read_chargeback_transaction() {
        let csv_data = "chargeback,1,100\n";

        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(csv_data.as_bytes());

        // deposit,1,1,10.50
        let expected = Transaction {
            t_type: Type::Chargeback,
            t_client_id: 1,
            transaction_id: 100,
            amount: None,
        };

        let transaction = rdr.deserialize::<Transaction>().next().unwrap();
        assert!(transaction.is_ok());
        assert_eq!(expected, transaction.unwrap());
    }
}
