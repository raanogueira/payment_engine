use std::collections::HashMap;
use std::fmt;

use crate::exchange::transaction::ClientId;
use crate::exchange::transaction::Currency;
use crate::exchange::transaction::Money;
use crate::exchange::transaction::Transaction;
use crate::exchange::transaction::TransactionId;
use crate::exchange::transaction::Type;

#[derive(Debug, PartialEq)]
pub struct ClientProfile {
    id: ClientId,
    available: Currency,
    held: Currency,
    total: Currency,
    locked: bool,
    transactions: HashMap<TransactionId, Transaction>,
}

#[derive(Debug)]
pub struct ProcessingError(pub String);

impl ClientProfile {
    pub fn new_with_defaults(id: ClientId) -> ClientProfile {
        Self::new(
            id,
            Currency::zero(),
            Currency::zero(),
            Currency::zero(),
            false,
            HashMap::new(),
        )
    }

    pub fn new(
        id: ClientId,
        available: Currency,
        held: Currency,
        total: Currency,
        locked: bool,
        transactions: HashMap<TransactionId, Transaction>,
    ) -> ClientProfile {
        ClientProfile {
            id,
            available,
            held,
            total,
            locked,
            transactions,
        }
    }

    /// It was assumed that both Deposits and Withdrawals can be disputed
    /// Malformed Deposits and Withdrawals (without an amount defined) are ignored
    /// It was also assumed that transactions can be disputed multiple times
    pub fn process_new_transaction(
        &mut self,
        transaction: Transaction,
    ) -> Result<(), ProcessingError> {
        if self.locked {
            return Err(ProcessingError(format!(
                "Client's account {} is locked. {:?} not permitted.. Rejecting transaction {}",
                self.id, transaction.tx_type, transaction
            )));
        }

        match transaction.tx_type {
            Type::Deposit => self.deposit(transaction),

            Type::Withdrawal => self.withdrawal(transaction),

            Type::Dispute => self.dispute(transaction),

            Type::Resolve => self.resolve(transaction),

            Type::Chargeback => self.chargeback(transaction),
        }
    }

    fn deposit(&mut self, transaction: Transaction) -> Result<(), ProcessingError> {
        if let Some(amount_to_deposit) = transaction.amount {
            self.transactions
                .entry(transaction.tx)
                .or_insert_with(|| transaction);
            self.available += amount_to_deposit;
            self.total += amount_to_deposit;
            Result::Ok(())
        } else {
            Result::Err(ProcessingError(format!(
                "Igoring malformed transaction {}..",
                transaction
            )))
        }
    }

    fn withdrawal(&mut self, transaction: Transaction) -> Result<(), ProcessingError> {
        if let Some(amount_to_withdraw) = transaction.amount {
            let to_debit = amount_to_withdraw;
            if self.available - to_debit >= Currency::zero() {
                self.transactions
                    .entry(transaction.tx)
                    .or_insert_with(|| transaction);

                self.available -= to_debit;
                self.total -= to_debit;
                Result::Ok(())
            } else {
                Result::Err(ProcessingError(format!(
                    "{} amount exceeds available funds {}. Igoring transaction ..",
                    to_debit, self.available
                )))
            }
        } else {
            Result::Err(ProcessingError(format!(
                "Igoring Withdrawal transaction {} with missing the amount field..",
                transaction
            )))
        }
    }

    fn dispute(&mut self, transaction: Transaction) -> Result<(), ProcessingError> {
        if let Some(open_transaction) = self.transactions.get_mut(&transaction.tx) {
            if let Some(disputed) = open_transaction.amount {
                self.held += disputed;
                self.available -= disputed;
                open_transaction.start_dispute();
            }
        }

        Result::Ok(())
    }

    fn resolve(&mut self, transaction: Transaction) -> Result<(), ProcessingError> {
        if let Some(existing_transaction) = self.transactions.get_mut(&transaction.tx) {
            if existing_transaction.under_dispute {
                if let Some(to_add) = existing_transaction.amount {
                    self.held -= to_add;
                    self.available += to_add;
                    existing_transaction.stop_dispute();
                }
            }
        }

        Result::Ok(())
    }

    fn chargeback(&mut self, transaction: Transaction) -> Result<(), ProcessingError> {
        if let Some(existing_transaction) = self.transactions.get_mut(&transaction.tx) {
            if existing_transaction.under_dispute {
                if let Some(chargeback) = existing_transaction.amount {
                    self.held -= chargeback;
                    self.total -= chargeback;
                    self.locked = true;
                    existing_transaction.stop_dispute();
                }
            }
        }

        Result::Ok(())
    }
}

impl fmt::Display for ClientProfile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{},{:.4},{:.4},{:.4},{}",
            self.id, self.available, self.held, self.total, self.locked
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

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

    #[test]
    fn it_should_subtract_funds_when_processing_withdrawals() {
        let mut client_profile = ClientProfile::new(
            1,
            Currency::str("0.0002"),
            Currency::str("0.0"),
            Currency::str("0.0002"),
            false,
            HashMap::new(),
        );

        client_profile
            .process_new_transaction(Transaction {
                tx_type: Type::Withdrawal,
                client: 1,
                tx: 1000,
                amount: Some(Currency::str("0.0002")),
                under_dispute: false,
            })
            .unwrap_or_default();

        assert_eq!(Currency::str("0.0000"), client_profile.available);
        assert_eq!(Currency::str("0.0000"), client_profile.total);
        assert_eq!(Currency::str("0.0000"), client_profile.held);
        assert_eq!(false, client_profile.locked);
        assert_eq!(1, client_profile.transactions.len());
    }

    #[test]
    fn it_should_ignore_withdrawal_when_account_does_not_enough_funds() {
        let mut client_profile = ClientProfile::new(
            1,
            Currency::str("0.0002"),
            Currency::str("0.1000"),
            Currency::str("0.1002"),
            false,
            HashMap::new(),
        );

        let result = client_profile
            .process_new_transaction(Transaction {
                tx_type: Type::Withdrawal,
                client: 1,
                tx: 1000,
                amount: Some(Currency::str("0.0003")),
                under_dispute: false,
            })
            .err();

        assert_eq!(true, result.is_some());
        assert_eq!(Currency::str("0.0002"), client_profile.available);
        assert_eq!(Currency::str("0.1002"), client_profile.total);
        assert_eq!(Currency::str("0.1000"), client_profile.held);
        assert_eq!(false, client_profile.locked);
        assert_eq!(0, client_profile.transactions.len());
    }

    #[test]
    fn it_should_ignore_disputes_for_non_existing_transactions() {
        let mut client_profile = ClientProfile::new(
            1,
            Currency::str("0.0002"),
            Currency::str("0.0"),
            Currency::str("0.0002"),
            false,
            HashMap::from([(
                1000,
                Transaction {
                    tx_type: Type::Deposit,
                    client: 1,
                    tx: 1000,
                    amount: Some(Currency::str("0.0002")),
                    under_dispute: false,
                },
            )]),
        );

        //dispute referencing an non existing transaction
        client_profile
            .process_new_transaction(Transaction {
                tx_type: Type::Dispute,
                client: 1,
                tx: 1001,
                amount: None,
                under_dispute: false,
            })
            .unwrap_or_default();

        assert_eq!(Currency::str("0.0002"), client_profile.available);
        assert_eq!(Currency::str("0.0002"), client_profile.total);
        assert_eq!(Currency::str("0.0000"), client_profile.held);
        assert_eq!(false, client_profile.locked);
        assert_eq!(1, client_profile.transactions.len());
        assert_eq!(
            false,
            client_profile
                .transactions
                .get(&1000)
                .unwrap()
                .under_dispute
        );
    }

    #[test]
    fn it_should_dispute_existing_transactions() {
        let mut client_profile = ClientProfile::new(
            1,
            Currency::str("0.0002"),
            Currency::str("0.00"),
            Currency::str("0.0002"),
            false,
            HashMap::from([(
                1000,
                Transaction {
                    tx_type: Type::Deposit,
                    client: 1,
                    tx: 1000,
                    amount: Some(Currency::str("0.0002")),
                    under_dispute: false,
                },
            )]),
        );

        client_profile
            .process_new_transaction(Transaction {
                tx_type: Type::Dispute,
                client: 1,
                tx: 1000,
                amount: None,
                under_dispute: false,
            })
            .unwrap_or_default();

        assert_eq!(Currency::str("0.0000"), client_profile.available);
        assert_eq!(Currency::str("0.0002"), client_profile.total);
        assert_eq!(Currency::str("0.0002"), client_profile.held);
        assert_eq!(false, client_profile.locked);
        assert_eq!(1, client_profile.transactions.len());
        assert_eq!(
            true,
            client_profile
                .transactions
                .get(&1000)
                .unwrap()
                .under_dispute
        );
    }

    #[test]
    fn it_should_resolve_existing_dispute() {
        let mut client_profile = ClientProfile::new(
            1,
            Currency::str("0.0000"),
            Currency::str("0.0002"),
            Currency::str("0.0002"),
            false,
            HashMap::from([(
                1000,
                Transaction {
                    tx_type: Type::Deposit,
                    client: 1,
                    tx: 1000,
                    amount: Some(Currency::str("0.0002")),
                    under_dispute: true,
                },
            )]),
        );

        client_profile
            .process_new_transaction(Transaction {
                tx_type: Type::Resolve,
                client: 1,
                tx: 1000,
                amount: None,
                under_dispute: false,
            })
            .unwrap_or_default();

        assert_eq!(Currency::str("0.0002"), client_profile.available);
        assert_eq!(Currency::str("0.0002"), client_profile.total);
        assert_eq!(Currency::str("0.0000"), client_profile.held);
        assert_eq!(false, client_profile.locked);
        assert_eq!(1, client_profile.transactions.len());
        assert_eq!(
            false,
            client_profile
                .transactions
                .get(&1000)
                .unwrap()
                .under_dispute
        );
    }

    #[test]
    fn it_should_chargeback_existing_dispute() {
        let mut client_profile = ClientProfile::new(
            1,
            Currency::str("0.0000"),
            Currency::str("0.0002"),
            Currency::str("0.0002"),
            false,
            HashMap::from([(
                1000,
                Transaction {
                    tx_type: Type::Deposit,
                    client: 1,
                    tx: 1000,
                    amount: Some(Currency::str("0.0002")),
                    under_dispute: true,
                },
            )]),
        );

        client_profile
            .process_new_transaction(Transaction {
                tx_type: Type::Chargeback,
                client: 1,
                tx: 1000,
                amount: None,
                under_dispute: false,
            })
            .unwrap_or_default();

        assert_eq!(Currency::str("0.0000"), client_profile.available);
        assert_eq!(Currency::str("0.0000"), client_profile.total);
        assert_eq!(Currency::str("0.0000"), client_profile.held);
        assert_eq!(true, client_profile.locked);
        assert_eq!(1, client_profile.transactions.len());
        assert_eq!(
            false,
            client_profile
                .transactions
                .get(&1000)
                .unwrap()
                .under_dispute
        );
    }

    #[test]
    fn it_should_be_able_to_dispute_multiple_transactions() {
        let mut client_profile = ClientProfile::new(
            1,
            Currency::str("1.0011"),
            Currency::str("0.00"),
            Currency::str("1.0011"),
            false,
            HashMap::from([
                (
                    333,
                    Transaction {
                        tx_type: Type::Deposit,
                        client: 1,
                        tx: 333,
                        amount: Some(Currency::str("0.0002")),
                        under_dispute: false,
                    },
                ),
                (
                    2222,
                    Transaction {
                        tx_type: Type::Deposit,
                        client: 1,
                        tx: 2222,
                        amount: Some(Currency::str("1.0009")),
                        under_dispute: false,
                    },
                ),
            ]),
        );

        client_profile
            .process_new_transaction(Transaction {
                tx_type: Type::Dispute,
                client: 1,
                tx: 333,
                amount: None,
                under_dispute: false,
            })
            .unwrap_or_default();

        client_profile
            .process_new_transaction(Transaction {
                tx_type: Type::Dispute,
                client: 1,
                tx: 2222,
                amount: None,
                under_dispute: false,
            })
            .unwrap_or_default();

        assert_eq!(Currency::str("0.0000"), client_profile.available);
        assert_eq!(Currency::str("1.0011"), client_profile.total);
        assert_eq!(Currency::str("1.0011"), client_profile.held);
        assert_eq!(false, client_profile.locked);
        assert_eq!(2, client_profile.transactions.len());
        assert_eq!(
            true,
            client_profile.transactions.get(&333).unwrap().under_dispute
        );
        assert_eq!(
            true,
            client_profile
                .transactions
                .get(&2222)
                .unwrap()
                .under_dispute
        );
    }

    #[test]
    fn it_should_ignore_transactions_without_an_amount() {
        let mut client_profile = ClientProfile::new_with_defaults(1);

        client_profile
            .process_new_transaction(Transaction {
                tx_type: Type::Deposit,
                client: 1,
                tx: 1000,
                amount: None,
                under_dispute: false,
            })
            .unwrap_or_default();

        client_profile
            .process_new_transaction(Transaction {
                tx_type: Type::Withdrawal,
                client: 1,
                tx: 1001,
                amount: None,
                under_dispute: false,
            })
            .unwrap_or_default();

        assert_eq!(Currency::str("0.0000"), client_profile.available);
        assert_eq!(Currency::str("0.0000"), client_profile.total);
        assert_eq!(Currency::str("0.0000"), client_profile.held);
        assert_eq!(false, client_profile.locked);
        assert_eq!(0, client_profile.transactions.len());
    }
}
