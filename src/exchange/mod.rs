use std::collections::HashMap;
use std::error::Error;

mod client_profile;
mod transaction;

use client_profile::ClientProfile;
use client_profile::ProcessingError;
use transaction::ClientId;
use transaction::Transaction;

pub struct Exchange {
    clients: HashMap<ClientId, ClientProfile>,
}

impl Exchange {
    pub fn new() -> Exchange {
        Exchange {
            clients: HashMap::new(),
        }
    }

    /// If the client does not exist, create a new one.
    /// ClientProfile::new() is only called when the client does not exist: or_insert_with with the default closure guarantee that a new ClientProfile is not created every time .entry() is called
    fn process_new_transaction(&mut self, transaction: Transaction) -> Result<(), ProcessingError> {
        let client = self
            .clients
            .entry(transaction.client)
            .or_insert_with(|| ClientProfile::new_with_defaults(transaction.client));
        client.process_new_transaction(transaction)
    }

    pub fn to_csv(&self) {
        println!("client,available,held,total,locked");
        self.clients.iter().for_each(|(_, client)| {
            println!("{}", client);
        });
    }
}

//read one record at the time and only deserialize the current one. This avoids loading a huge dataset into memory and also to only deserilaise the current row that is being processed
pub fn process_transactions_from_csv(
    path: &str,
    bank: &mut Exchange,
) -> Result<(), Box<dyn Error>> {
    let mut reader = csv::Reader::from_path(path)?;

    let headers = reader.headers()?.clone();

    let mut raw_record = csv::StringRecord::new();
    while reader.read_record(&mut raw_record)? {
        let t: Transaction = raw_record.deserialize(Some(&headers))?;
        if let Err(ProcessingError(error)) = bank.process_new_transaction(t) {
            eprintln!("{}", error);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    use std::rc::Rc;
    use transaction::Currency;
    use transaction::Money;
    use transaction::Type;

    #[test]
    fn it_should_handle_deposits_and_withdrawals_for_multiple_clients() {
        let mut exchange = Exchange::new();
        let tx91 = Transaction {
            tx_type: Type::Deposit,
            client: 1,
            tx: 91,
            amount: Some(Currency::str("123.0")),
        };
        let tx92 = Transaction {
            tx_type: Type::Deposit,
            client: 2,
            tx: 92,
            amount: Some(Currency::str("55.0")),
        };
        let tx93 = Transaction {
            tx_type: Type::Withdrawal,
            client: 2,
            tx: 93,
            amount: Some(Currency::str("44.0")),
        };
        let tx94 = Transaction {
            tx_type: Type::Withdrawal,
            client: 1,
            tx: 94,
            amount: Some(Currency::str("33.0")),
        };

        exchange.process_new_transaction(tx91.clone());
        exchange.process_new_transaction(tx92.clone());
        exchange.process_new_transaction(tx93.clone());
        exchange.process_new_transaction(tx94.clone());

        let client1 = ClientProfile::new(
            1,
            Currency::str("90.0"),
            Currency::str("0.0"),
            Currency::str("90.0"),
            false,
            HashMap::from([(tx91.tx, Rc::new(tx91)), (tx94.tx, Rc::new(tx94))]),
            HashMap::new(),
        );

        let client2 = ClientProfile::new(
            2,
            Currency::str("11.0"),
            Currency::str("0.0"),
            Currency::str("11.0"),
            false,
            HashMap::from([(tx92.tx, Rc::new(tx92)), (tx93.tx, Rc::new(tx93))]),
            HashMap::new(),
        );

        assert_eq!(
            HashMap::from([(1, client1), (2, client2)]),
            exchange.clients
        );
    }

    #[test]
    fn it_should_resolve_disputes() {
        let mut exchange = Exchange::new();
        let tx91 = Transaction {
            tx_type: Type::Deposit,
            client: 1,
            tx: 91,
            amount: Some(Currency::str("123.0")),
        };
        let tx92 = Transaction {
            tx_type: Type::Dispute,
            client: 1,
            tx: 91,
            amount: None,
        };

        exchange.process_new_transaction(tx91.clone());
        exchange.process_new_transaction(tx92.clone());

        let rc_tx91 = Rc::new(tx91);
        let rc_tx91_clone = Rc::clone(&rc_tx91);

        let client_with_open_dispute = ClientProfile::new(
            1,
            Currency::str("00.0"),
            Currency::str("123.0"),
            Currency::str("123.0"),
            false,
            HashMap::from([(rc_tx91.tx, rc_tx91)]),
            HashMap::from([(rc_tx91_clone.tx, rc_tx91_clone)]),
        );

        assert_eq!(
            HashMap::from([(1, client_with_open_dispute)]),
            exchange.clients
        );

        exchange.process_new_transaction(Transaction {
            tx_type: Type::Resolve,
            client: 1,
            tx: 91,
            amount: None,
        });

        let client_with_resolved_disputed = ClientProfile::new(
            1,
            Currency::str("123.0"),
            Currency::str("00.0"),
            Currency::str("123.0"),
            false,
            HashMap::new(),
            HashMap::new(),
        );

        assert_eq!(
            HashMap::from([(1, client_with_resolved_disputed)]),
            exchange.clients
        );
    }

    #[test]
    fn it_should_chargeback_disputes() {
        let mut exchange = Exchange::new();
        let tx91 = Transaction {
            tx_type: Type::Deposit,
            client: 1,
            tx: 91,
            amount: Some(Currency::str("123.0")),
        };
        let tx92 = Transaction {
            tx_type: Type::Dispute,
            client: 1,
            tx: 91,
            amount: None,
        };

        exchange.process_new_transaction(tx91.clone());
        exchange.process_new_transaction(tx92.clone());

        let rc_tx91 = Rc::new(tx91);
        let rc_tx91_clone = Rc::clone(&rc_tx91);

        let client_with_open_dispute = ClientProfile::new(
            1,
            Currency::str("00.0"),
            Currency::str("123.0"),
            Currency::str("123.0"),
            false,
            HashMap::from([(rc_tx91.tx, rc_tx91)]),
            HashMap::from([(rc_tx91_clone.tx, rc_tx91_clone)]),
        );

        assert_eq!(
            HashMap::from([(1, client_with_open_dispute)]),
            exchange.clients
        );

        exchange.process_new_transaction(Transaction {
            tx_type: Type::Chargeback,
            client: 1,
            tx: 91,
            amount: None,
        });

        let client_after_being_chargedback = ClientProfile::new(
            1,
            Currency::str("00.0"),
            Currency::str("00.0"),
            Currency::str("00.0"),
            true,
            HashMap::new()
        );

        assert_eq!(
            HashMap::from([(1, client_after_being_chargedback)]),
            exchange.clients
        );
    }

    #[test]
    fn it_should_ignore_disputes_for_non_existing_transactions() {
        let mut exchange = Exchange::new();
        let tx91 = Transaction {
            tx_type: Type::Deposit,
            client: 1,
            tx: 91,
            amount: Some(Currency::str("123.0")),
        };
        let tx92 = Transaction {
            tx_type: Type::Dispute,
            client: 1,
            tx: 555,
            amount: None,
        };

        exchange.process_new_transaction(tx91.clone());
        exchange.process_new_transaction(tx92.clone());

        let rc_tx91 = Rc::new(tx91);

        let client_with_no_disputes = ClientProfile::new(
            1,
            Currency::str("123.0"),
            Currency::str("00.0"),
            Currency::str("123.0"),
            false,
            HashMap::from([(rc_tx91.tx, rc_tx91)]),
            HashMap::new(),
        );

        assert_eq!(
            HashMap::from([(1, client_with_no_disputes)]),
            exchange.clients
        );
    }

    #[test]
    fn it_should_ignore_resolve_for_non_existing_disputes() {
        let mut exchange = Exchange::new();
        let deposit = Transaction {
            tx_type: Type::Deposit,
            client: 1,
            tx: 91,
            amount: Some(Currency::str("123.0")),
        };
        let resolve = Transaction {
            tx_type: Type::Resolve,
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

    #[test]
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
}
