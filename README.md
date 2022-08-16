# csv-payments

A simple settlement engine which takes a CSV as input and outputs a CSV with the
status of accounts.

## Running
This Rust program takes 1 argument, a file path to the CSV input. The output can be redirected to another file for viewing, or viewed directly on stdout.

Example:
```
cargo run -- data/transactions_basic.csv >> accounts.csv
```

## Test Coverage

All supported transaction types have test coverage verifying they work as expected in the `src/account.rs` file. There are also tests verifying that processing a series of transactions outputs the expected values in `src/main.rs`.

## Performance

This engine knowingly uses additional memory to avoid time-intensive operations for processing transactions efficiently. To avoid looping over lists of transactions or accounts, we assign them locations in Hashmaps where they can be looked up using their indices.

## Next Steps

- Improve edge case handling. This program makes some naive assumptions that may not always be true, such as assuming transactions which require a ref_tx (Dispute, Resolve, Chargeback) will eventually have their ref_tx added into the system. This wouldn't work well in a system where external sources may send invalid Dispute transactions which don't have valid `tx_id`s. Additionally, if a Dispute transaction is sent multiple times, this program naively continues to process that transaction multiple times.
- Generally improve error handling throughout instead of using `expect()`
- Add debug logging which can be toggled on/off using env vars. This can help
give insight into why some edge cases were not properly handled.
- Refactor code to improve ownership/maintainability. The `src/main.rs` file contains some processing logic that should really be split out and tested elsewhere.