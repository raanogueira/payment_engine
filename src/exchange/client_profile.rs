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
    transactions: HashMap<TransactionId, Transaction>
}

pub struct ProcessingError(pub String);

impl ClientProfile {
    pub fn new_with_defaults(id: ClientId) -> ClientProfile {
        Self::new(
            id,
            Currency::zero(),
            Currency::zero(),
            Currency::zero(),
            false,
            HashMap::new()
        )
    }

    pub fn new(
        id: ClientId,
        available: Currency,
        held: Currency,
        total: Currency,
        locked: bool,
        transactions: HashMap<TransactionId, Transaction>
    ) -> ClientProfile {
        ClientProfile {
            id,
            available: available,
            held: held,
            total: total,
            locked: locked,
            transactions: transactions
        }
    }

    /// It was assumed that both Deposits and Withdrawals can be disputed
    /// Malformed Deposits and Withdrawals (without an amount defined) are ignored
    /// It was also assumed that transactions can be disputed multiple times
    pub fn process_new_transaction(&mut self, transaction: Transaction) -> Result<(), ProcessingError> {
        if self.locked {
            return Err(ProcessingError(format!("Client's account {} is locked. {:?} not permitted.. Rejecting transaction {}", self.id, transaction.tx_type, transaction)));
        }

        match transaction.tx_type {
            Type::Deposit => {
                self.deposit(transaction)
            }

            Type::Withdrawal => {
                 self.withdrawal(transaction)
            }

            Type::Dispute => {
                self.dispute(transaction)
            }

            Type::Resolve => {
                self.resolve(transaction)
            }

            Type::Chargeback => {
                self.chargeback(transaction)
            }
        }
    }

    fn deposit(&mut self, transaction: Transaction) -> Result<(), ProcessingError> {
        if let Some(amount_to_deposit) = transaction.amount {
            self.transactions
                .entry(transaction.tx)
                .or_insert_with(|| transaction);
            self.available += amount_to_deposit;
            self.total += amount_to_deposit;
            return Result::Ok(())
        } else {
            return Result::Err(ProcessingError(format!("Igoring malformed transaction {}..", transaction)));
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
                return Result::Ok(());
            } else {
                return Result::Err(ProcessingError(format!("{} amount exceeds available funds {}. Igoring transaction ..",to_debit, self.available)));
            }
        } else {
            return Result::Err(ProcessingError(format!("Igoring Withdrawal transaction {} with missing the amount field..", transaction)));
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
        if let Some(under_dispute) = self.transactions.get_mut(&transaction.tx) {
            if let Some(to_add) = under_dispute.amount {
                self.held -= to_add;
                self.available += to_add;
                under_dispute.stop_dispute();
            }
        }

        Result::Ok(())
    }

    fn chargeback(&mut self, transaction: Transaction) -> Result<(), ProcessingError> {
        if let Some(under_dispute) = self.transactions.get_mut(&transaction.tx) {
            if let Some(chargeback) = under_dispute.amount {
                self.held -= chargeback;
                self.total -= chargeback;
                self.locked = true;
                under_dispute.stop_dispute();
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
