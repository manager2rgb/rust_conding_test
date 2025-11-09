## Payments engine

## 1 Assumptions:
### 1.1 Only deposits can be disputed.
The exercise describes disputes in the context of reversing deposits: “a malicious actor may try to deposit fiat… and then reverse their fiat deposit”.  
Because of this, I treat only deposit transactions as dispute.

### 1.2 Amounts can not be negative
A negative withdrawal or deposit does not make sense in this use case.  
Negative amounts are rejected during deserialization.

### 1.3 About efficiency
The coding test mentions a scenario where the engine could be receiving streams from thousands of concurrent TCP connections.  
For this exercise, I used a `LazyLock` holding a `PaymentsEngine` wrapped in `Arc<Mutex<…>>`.  
In a real system, I would consider per-client locking, or lock-free data structures, but due to time constraints and because this is a coding test, I kept the concurrency model simple and safe.


## 2 Design

### 2.1 Read file without loading fully in memory.
A `BufReader` is used so the file is streamed line by line rather than loaded entirely into memory.

### 2.2 Transactions
Transactions are deserialized using `serde`.  
I implemented a custom deserializer to reject invalid values up front:
-   Negative amounts
-   Amounts with more than four decimal places
This enforces correctness early and keeps the rest of the code simpler.

### 2.3 Types
Instead of using `u16` or `u32` throughout the code, I define specific type aliases for client IDs, transaction IDs, and amounts.  
This makes the domain model easier to understand and reduces mistakes.

### 2.4 Decimal handling
Since the exercise requires four decimal places, I used the `rust_decimal` crate.  
I had not worked with 4-decimal financial values in Rust before, so using a library avoided potential floating-point issues.

### 2.5 Data structures
My initial structure for transactions was:

`HashMap<ClientId, HashMap<TransactionId, Amount>>` 

I later realized that only the transaction ID is needed when resolving or disputing, since the engine already knows the client.

I decided to use a flatter structure mapping:

`transaction_id → (client_id, amount)` 

This avoids one level of hashing and simplifies lookups.

### 2.6 Errors
I used `thiserror` to define clear error types for client and engine failures.  
This keeps the main logic cleaner and easier to test.

### 2.7 Unit tests
Most components are covered by unit tests.  
One unit-test is still commented out while getting problems with ordering and formatting.

## AI prompts
I did **not** use AI to generate the entire solution or to write code I could not explain. I can explain every line of code and every prompt I used if asked.
I work on open source projects which use most of the libraries used for this projcet and I can demostrate exactly where and how I haved used code as reference for this implementation,


> how to reject serde value to be non negative?
```Rust
use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer};

fn de_decimal_non_negative<'de, D>(deserializer: D) -> Result<Option<Decimal>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<Decimal>::deserialize(deserializer)?;

    if let Some(v) = opt {
        if v.is_sign_negative() {
            return Err(serde::de::Error::custom("amount must be non-negative"));
        }
        Ok(Some(v))
    } else {
        Ok(None)
    }
}
```



> how to use F: FnOnce?
```Rust
fn call_once_with_arg<F>(f: F, x: i32)
where
    F: FnOnce(i32) -> i32,
{
    let result = f(x);
    println!("Result: {}", result);
}

fn main() {
    let double = |n| n * 2;
    call_once_with_arg(double, 5); // prints 10
}
```


> error wtih csv if a row contain less fields (dispuse, resolve, ect)

IA suggested `.flexible(true)` option


## Test
The CLI `payment_engine` takes one arguments to run: the input CSV file path.
```sh
cargo run -- transactions.csv
```
Output will be written to std out.

```sh
cargo run -- transactions_large.csv
```

To execute unit test run:
```sh
cargo test
```