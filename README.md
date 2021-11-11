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

The unit tests for the ClientProfilee and Exchange allowed validating the main requirements of this project.

Manual testing using different datasets (some of them available in this repo) was also performed. This manual task was very important to validate the deserialisation and serialisation logic.


# Improvements

## Real-world improvements/Sharding

In a real case scenario, the transaction processor could have been split into different components/threads/services (depending on the size of the problem) where one service would read the data from a file or receive the data/requests via a WebSocket or REST API. The service would then publish (using Rust's std::sync::mpsc, Tokio, Kafka or RabbitMQ) the transactions to separate workers. 

On a larger scale, these workers (threads or services) would be handling a smaller subset of transactions i.e. the transactions could be sharded by client id or the client's location. 

This solution, of course, brings many other challenges such as the atomicity of the transactions in a distributed environment with many services. It would be required some sort of orchestration or a pattern like SAGA (2pc in a monolith) to guarantee consistency of long-lived transactions.

## A single binary with multi-threading 

Assuming this had to be done using a single binary in a single computer.

The lack of timestamps in the file makes it hard to split the work of reading the CSV. The current solution assumes that all transactions are chronologically ordered in the file. 

If the transactions had a timestamp, it would possible to split the work to read the whole CSV into multiple smaller tasks (performed by different threads in a single binary) and speed up the whole task dramatically. This would have a big impact as the bottleneck is reading and parsing the CSV entries, 

## Datamodel

There is scope to improve the datamodel, but that would require a custom deserialiser which can be done in v1.0.1 of this project. 

A better datamodel would reduce some duplication and also make the code more robust and readable e.g deposits and withdrawals without an amount defined would be filtered out when reading and parsing the CSV entries.