use rust_decimal::dec;

use crate::types::Amount;

pub struct Client {
    available: Amount,
    held: Amount,
    total: Amount,
    locked: bool,
}

impl Client {
    pub fn new() -> Self {
        Self {
            available: dec!(0_0000),
            held: dec!(0_0000),
            total: dec!(0_0000),
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

    pub fn deposit(&mut self, amount: Amount) {
        self.available += amount;
        self.total += amount;
    }

    pub fn withdrawal(&mut self, amount: Amount) {
        if self.available >= amount {
            // meaning susfficient or equal amount of money
            self.available -= amount;
            self.total -= amount;
        }
    }

    pub fn dispute(&mut self, amount: Amount) {
        self.available -= amount; // clients available funds should decrease by the amount disputed
        self.held += amount; // their held funds should increase by the amount disputed
    }

    pub fn resolve(&mut self, amount: Amount) {
        self.held -= amount; // clients held funds should decrease by the amount no longer disputed
        self.available += amount; // available funds should increase by the amount no longer disputed
    }

    pub fn chargeback(&mut self, amount: Amount) {
        // clients held funds and total funds should decrease by the amount previously disputed.
        self.held -= amount;
        self.total -= amount;
        self.locked = true; //  If a chargeback occurs the client's account should be immediately frozen
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn client() {
        let client = Client::new();

        assert_eq!(client.available(), dec!(0_0000));
        assert_eq!(client.held(), dec!(0_0000));
        assert_eq!(client.total(), dec!(0_0000));
        assert!(!client.locked());
    }

    #[test]
    fn client_deposit() {
        let mut client = Client::new();

        client.deposit(dec!(1.5555));
        assert_eq!(client.available(), dec!(1.5555));
        assert_eq!(client.held(), dec!(0_0000));
        assert_eq!(client.total(), dec!(1.5555));
        assert!(!client.locked());
    }

    #[test]
    fn client_withdrawal() {
        let mut client = Client::new();

        client.deposit(dec!(1.5555));

        client.withdrawal(dec!(0.5555));
        assert_eq!(client.available(), dec!(1.000));
        assert_eq!(client.held(), dec!(0_0000));
        assert_eq!(client.total(), dec!(1.0000));
        assert!(!client.locked());

        client.withdrawal(dec!(0.9999));
        assert_eq!(client.available(), dec!(0.0001));
        assert_eq!(client.held(), dec!(0_0000));
        assert_eq!(client.total(), dec!(0.0001));
        assert!(!client.locked());

        client.withdrawal(dec!(0.0002)); //insufficient money
        assert_eq!(client.available(), dec!(0.0001));
        assert_eq!(client.held(), dec!(0_0000));
        assert_eq!(client.total(), dec!(0.0001));
        assert!(!client.locked());
    }

    #[test]
    fn client_dispute() {
        let mut client = Client::new();

        client.deposit(dec!(1.5555));

        client.dispute(dec!(0.5555));
        assert_eq!(client.available(), dec!(1.0000));
        assert_eq!(client.held(), dec!(0.5555));
        assert_eq!(client.total(), dec!(1.5555));
        assert!(!client.locked());

        client.withdrawal(dec!(0.9999));
        assert_eq!(client.available(), dec!(0.0001));
        assert_eq!(client.held(), dec!(0.5555));
        assert_eq!(client.total(), dec!(0.5556));
        assert!(!client.locked());
    }

    #[test]
    fn client_resolve() {
        let mut client = Client::new();

        client.deposit(dec!(1.5555));

        client.dispute(dec!(0.5555));
        assert_eq!(client.available(), dec!(1.0000));
        assert_eq!(client.held(), dec!(0.5555));
        assert_eq!(client.total(), dec!(1.5555));
        assert!(!client.locked());

        client.withdrawal(dec!(0.9999));
        assert_eq!(client.available(), dec!(0.0001));
        assert_eq!(client.held(), dec!(0.5555));
        assert_eq!(client.total(), dec!(0.5556));
        assert!(!client.locked());

        client.resolve(dec!(0.5555));
        assert_eq!(client.available(), dec!(0.5556));
        assert_eq!(client.held(), dec!(0.0000));
        assert_eq!(client.total(), dec!(0.5556));
        assert!(!client.locked());
    }

    #[test]
    fn client_chargeback() {
        let mut client = Client::new();

        client.deposit(dec!(1.5555));

        client.dispute(dec!(0.5555));
        assert_eq!(client.available(), dec!(1.0000));
        assert_eq!(client.held(), dec!(0.5555));
        assert_eq!(client.total(), dec!(1.5555));
        assert!(!client.locked());

        client.withdrawal(dec!(0.9999));
        assert_eq!(client.available(), dec!(0.0001));
        assert_eq!(client.held(), dec!(0.5555));
        assert_eq!(client.total(), dec!(0.5556));
        assert!(!client.locked());

        client.chargeback(dec!(0.5555));
        assert_eq!(client.available(), dec!(0.0001));
        assert_eq!(client.held(), dec!(0.0000));
        assert_eq!(client.total(), dec!(0.0001));
        assert!(client.locked());
    }
}
