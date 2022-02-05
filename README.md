# Transactions processor 

A Simple Rust App to process withdrawals, deposits, disputes, resolve and chargebacks

# Usage

to print the output to stdout:

```
cargo run -- transactions.csv
```

to print the output to a file:

```
cargo run -- transactions.csv > accounts.csv
```

The expected input file is a CSV with three mandatory fields for all transaction: type: string, client: u16, tx: u32 and amount: f64/Decimal (mandatory for Withdrawals and Deposits):

```
type,   client, tx, amount
deposit, 1, 1, 1.0
deposit, 2, 2, 2.0
deposit, 1, 3, 2.0
withdrawal, 1, 4, 1.5
withdrawal, 2, 5, 3.0
```

Output for the input above:

```
client,available,held,total,locked
1,1.5000,0.0000,1.5000,false
2,2.0000,0.0000,2.0000,false
```

# Assumptions

* All withdrawals and deposits can be disputed.

* Withdrawals and Deposits without an amount are deemed as not valid and not taken into account

* Resolve and Chargeback transactions are only considered if there is an open dispute for the respective deposit or withdrawal 

* The same transaction can be disputed many times.

# Testing

The main requirements were verified with high-level unit tests as the one described below:

```
#[test]
fn it_should_add_funds_when_processing_deposits() {
    let mut client_profile = ClientProfile::new_with_defaults(1);

    client_profile
        .process_new_transaction(Transaction {
            tx_type: Type::Deposit,
            client: 1,
            tx: 1000,
            amount: Some(Currency::str("0.0001")),
            under_dispute: false,
        })
        .unwrap_or_default();

    assert_eq!(Currency::str("0.0001"), client_profile.available);
    assert_eq!(Currency::str("0.0001"), client_profile.total);
    assert_eq!(Currency::str("0.0000"), client_profile.held);
    assert_eq!(false, client_profile.locked);
    assert_eq!(1, client_profile.transactions.len());
}
```