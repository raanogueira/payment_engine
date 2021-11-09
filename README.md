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

# Testing

The main requirements were verified with high level unit tests as the one described below:

```
fn it_should_ignore_chargeback_for_non_existing_disputes() {
        let mut exchange = Exchange::new();
        let deposit = Transaction {
            tx_type: Type::Deposit,
            client: 1,
            tx: 91,
            amount: Some(Currency::str("123.0")),
        };
        let resolve = Transaction {
            tx_type: Type::Chargeback,
            client: 1,
            tx: 91,
            amount: None,
        };

        exchange.process_new_transaction(deposit.clone());
        exchange.process_new_transaction(resolve);

        let rc_deposit = Rc::new(deposit);

        let client_with_no_disputes = ClientProfile::new(
            1,
            Currency::str("123.0"),
            Currency::str("00.0"),
            Currency::str("123.0"),
            false,
            HashMap::from([(rc_deposit.tx, rc_deposit)]),
            HashMap::new(),
        );

        assert_eq!(
            HashMap::from([(1, client_with_no_disputes)]),
            exchange.clients
        );
    }
```

Manual testing using different datasets (some of the available in this repo) was also performed

# Improvements

## Real world improvements/Sharding

In a real case scenario, the transaction processor could have been split into different components/threads/services (depending on the size of the problem) where one service would read the data from a file or receive the requests via a WebSocket/REST API and publish (using Rust's std::sync::mpsc, Tokio, Kafka or RabbitMQ) the transactions to separate workers. 

On a larger scale, these workers (threads or services) would be handling a smaller subset of transactions i.e. the transactions could be sharded by client id or the client's location. 

This solution, of course, brings many other challenges like the atomicity of the transactions in a distributed environment with many services. It would be required some sort of orchestration or a pattern like SAGA (2pc in a monolith) to guarantee consistency of long-lived transactions.

## A single binary with multi-threading 

Assuming this had to be done using a single binary in a single computer.

The lack of timestamps in the file makes it hard to split the work of reading the CSV because it is assumed that all transactions are ordered in the file. 
If transactions had a timestamp, it would possible to split the work to read the whole CSV into multiple smaller tasks (performed by different threads in a single binary) and speed up the whole task dramatically. The bottleneck is the task of reading and parsing the CSV entries.