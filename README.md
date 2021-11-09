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

# Assumptions

* All withdrawals and deposits can be disputed.

* Withdrawals and Deposits without an amount are deemed as not valid and not taken into account

* Resolve and Chargeback transactions are only considered if there is an open dispute for the respective deposit or withdrawal 


# Architecture 


# Testing




