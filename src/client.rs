pub struct Client {
    pub available: f32,
    pub held: f32,
    pub total: f32,
    pub locked: bool,
}

impl Client {
    pub fn new() -> Self {
        Self {
            available: 0.0,
            held: 0.0,
            total: 0.0,
            locked: false,
        }
    }

    pub fn deposit(&mut self, amount: f32) {
        self.available += amount;
        self.total += amount;
    }

    pub fn withdrawal(&mut self, amount: f32) {
        if self.available > amount {
            self.available -= amount;
            self.total -= amount;
        }
    }

    pub fn dispute(&mut self, amount: f32) {
        self.available -= amount; // clients available funds should decrease by the amount disputed
        self.held += amount; // their held funds should increase by the amount disputed
    }

    pub fn resolve(&mut self, amount: f32) {
        self.held -= amount; // clients held funds should decrease by the amount no longer disputed
        self.available += amount; // available funds should increase by the amount no longer disputed
    }

    pub fn chargeback(&mut self, amount: f32) {
        // clients held funds and total funds should decrease by the amount previously disputed.
        self.held -= amount;
        self.total -= amount;
        self.locked = true; //  If a chargeback occurs the client's account should be immediately frozen
    }
}
