use crate::{client::error::ClientAccountError, types::Amount};
use rust_decimal::Decimal;

pub struct ClientAccount {
    available: Amount,
    held: Amount,
    total: Amount,
    locked: bool,
}

impl ClientAccount {
    pub fn new() -> Self {
        Self {
            available: Decimal::ZERO,
            held: Decimal::ZERO,
            total: Decimal::ZERO,
            locked: false,
        }
    }

    pub fn available(&self) -> Amount {
        self.available
    }

    pub fn held(&self) -> Amount {
        self.held
    }

    pub fn total(&self) -> Amount {
        self.total
    }

    pub fn locked(&self) -> bool {
        self.locked
    }

    pub fn deposit(&mut self, amount: Amount) -> Result<(), ClientAccountError> {
        if self.locked {
            return Err(ClientAccountError::Locked);
        }

        if amount < Decimal::ZERO {
            return Err(ClientAccountError::NegativeAmount);
        }

        self.available += amount;
        self.total += amount;
        Ok(())
    }

    pub fn withdrawal(&mut self, amount: Amount) -> Result<(), ClientAccountError> {
        if self.locked {
            return Err(ClientAccountError::Locked);
        }

        if amount < Decimal::ZERO {
            return Err(ClientAccountError::NegativeAmount);
        }

        if self.available > amount {
            // meaning susfficient or equal amount of money
            self.available -= amount;
            self.total -= amount;
        } else {
            return Err(ClientAccountError::InsufficientBalance);
        }

        Ok(())
    }

    pub fn dispute(&mut self, amount: Amount) -> Result<(), ClientAccountError> {
        if self.locked {
            return Err(ClientAccountError::Locked);
        }

        self.available -= amount; // clients available funds should decrease by the amount disputed
        self.held += amount; // their held funds should increase by the amount disputed

        Ok(())
    }

    pub fn resolve(&mut self, amount: Amount) -> Result<(), ClientAccountError> {
        if self.locked {
            return Err(ClientAccountError::Locked);
        }
        self.held -= amount; // clients held funds should decrease by the amount no longer disputed
        self.available += amount; // available funds should increase by the amount no longer disputed
        Ok(())
    }

    pub fn chargeback(&mut self, amount: Amount) -> Result<(), ClientAccountError> {
        // clients held funds and total funds should decrease by the amount previously disputed.
        self.held -= amount;
        self.total -= amount;
        self.locked = true; //  If a chargeback occurs the client's account should be immediately frozen
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use rust_decimal::dec;

    #[test]
    fn client() {
        let client = ClientAccount::new();

        assert_eq!(client.available(), dec!(0_0000));
        assert_eq!(client.held(), dec!(0_0000));
        assert_eq!(client.total(), dec!(0_0000));
        assert!(!client.locked());
    }

    #[test]
    fn client_deposit() {
        let mut client = ClientAccount::new();

        assert!(client.deposit(dec!(1.5555)).is_ok());
        assert_eq!(client.available(), dec!(1.5555));
        assert_eq!(client.held(), dec!(0_0000));
        assert_eq!(client.total(), dec!(1.5555));
        assert!(!client.locked());
    }

    #[test]
    fn client_deposit_error() {
        let mut client = ClientAccount {
            available: Decimal::ZERO,
            held: Decimal::ZERO,
            total: Decimal::ZERO,
            locked: true,
        };

        assert_eq!(
            client.deposit(dec!(1.5555)).unwrap_err(),
            ClientAccountError::Locked
        );
        client.locked = false;
        assert_eq!(
            client.deposit(dec!(-1)).unwrap_err(),
            ClientAccountError::NegativeAmount
        );
    }

    #[test]
    fn client_withdrawal() {
        let mut client = ClientAccount::new();

        assert!(client.deposit(dec!(1.5555)).is_ok());

        assert!(client.withdrawal(dec!(0.5555)).is_ok());
        assert_eq!(client.available(), dec!(1.000));
        assert_eq!(client.held(), dec!(0_0000));
        assert_eq!(client.total(), dec!(1.0000));
        assert!(!client.locked());

        assert!(client.withdrawal(dec!(0.9999)).is_ok());
        assert_eq!(client.available(), dec!(0.0001));
        assert_eq!(client.held(), dec!(0_0000));
        assert_eq!(client.total(), dec!(0.0001));
        assert!(!client.locked());

        assert!(client.withdrawal(dec!(0.0002)).is_err()); //insufficient money error
        assert_eq!(client.available(), dec!(0.0001));
        assert_eq!(client.held(), dec!(0_0000));
        assert_eq!(client.total(), dec!(0.0001));
        assert!(!client.locked());
    }

    #[test]
    fn client_withdrawal_error() {
        let mut client = ClientAccount {
            available: dec!(1.0000),
            held: Decimal::ZERO,
            total: Decimal::ZERO,
            locked: true,
        };

        assert_eq!(
            client.withdrawal(dec!(1.5555)).unwrap_err(),
            ClientAccountError::Locked
        );
        client.locked = false;
        assert_eq!(
            client.withdrawal(dec!(-1)).unwrap_err(),
            ClientAccountError::NegativeAmount
        );
        assert_eq!(
            client.withdrawal(Decimal::MAX).unwrap_err(),
            ClientAccountError::InsufficientBalance
        );
    }

    #[test]
    fn client_dispute() {
        let mut client = ClientAccount::new();

        assert!(client.deposit(dec!(1.5555)).is_ok());

        assert!(client.dispute(dec!(0.5555)).is_ok());
        assert_eq!(client.available(), dec!(1.0000));
        assert_eq!(client.held(), dec!(0.5555));
        assert_eq!(client.total(), dec!(1.5555));
        assert!(!client.locked());

        assert!(client.withdrawal(dec!(0.9999)).is_ok());
        assert_eq!(client.available(), dec!(0.0001));
        assert_eq!(client.held(), dec!(0.5555));
        assert_eq!(client.total(), dec!(0.5556));
        assert!(!client.locked());
    }

    #[test]
    fn client_dispute_error() {
        let mut client = ClientAccount {
            available: Decimal::ZERO,
            held: Decimal::ZERO,
            total: Decimal::ZERO,
            locked: true,
        };

        assert_eq!(
            client.dispute(dec!(1.5555)).unwrap_err(),
            ClientAccountError::Locked
        );
    }

    #[test]
    fn client_resolve() {
        let mut client = ClientAccount::new();

        assert!(client.deposit(dec!(1.5555)).is_ok());

        assert!(client.dispute(dec!(0.5555)).is_ok());
        assert_eq!(client.available(), dec!(1.0000));
        assert_eq!(client.held(), dec!(0.5555));
        assert_eq!(client.total(), dec!(1.5555));
        assert!(!client.locked());

        assert!(client.withdrawal(dec!(0.9999)).is_ok());
        assert_eq!(client.available(), dec!(0.0001));
        assert_eq!(client.held(), dec!(0.5555));
        assert_eq!(client.total(), dec!(0.5556));
        assert!(!client.locked());

        assert!(client.resolve(dec!(0.5555)).is_ok());
        assert_eq!(client.available(), dec!(0.5556));
        assert_eq!(client.held(), dec!(0.0000));
        assert_eq!(client.total(), dec!(0.5556));
        assert!(!client.locked());
    }

    #[test]
    fn client_resolve_error() {
        let mut client = ClientAccount {
            available: Decimal::ZERO,
            held: Decimal::ZERO,
            total: Decimal::ZERO,
            locked: true,
        };

        assert_eq!(
            client.resolve(dec!(1.5555)).unwrap_err(),
            ClientAccountError::Locked
        );
    }

    #[test]
    fn client_chargeback() {
        let mut client = ClientAccount::new();

        assert!(client.deposit(dec!(1.5555)).is_ok());

        assert!(client.dispute(dec!(0.5555)).is_ok());
        assert_eq!(client.available(), dec!(1.0000));
        assert_eq!(client.held(), dec!(0.5555));
        assert_eq!(client.total(), dec!(1.5555));
        assert!(!client.locked());

        assert!(client.withdrawal(dec!(0.9999)).is_ok());
        assert_eq!(client.available(), dec!(0.0001));
        assert_eq!(client.held(), dec!(0.5555));
        assert_eq!(client.total(), dec!(0.5556));
        assert!(!client.locked());

        assert!(client.chargeback(dec!(0.5555)).is_ok());
        assert_eq!(client.available(), dec!(0.0001));
        assert_eq!(client.held(), dec!(0.0000));
        assert_eq!(client.total(), dec!(0.0001));
        assert!(client.locked());
    }
}
