use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::exchange::transaction::ClientId;
use crate::exchange::transaction::Currency;
use crate::exchange::transaction::Money;
use crate::exchange::transaction::Transaction;
use crate::exchange::transaction::TransactionId;
use crate::exchange::transaction::Type;

///Deposits and Withdrawals are no longer relevant (hence removed from the open_transactions) once they get to a final state (resolve/chargeback)
///The same transaction stored in memory is shared between the open_transactions and disputes HashMap's. Rc was used to only allocate one memory space in the heap.
#[derive(Debug, PartialEq)]
pub struct ClientProfile {
    id: ClientId,
    available: Currency,
    held: Currency,
    total: Currency,
    locked: bool,
    open_transactions: HashMap<TransactionId, Rc<Transaction>>,
    //Instead of maintaing two maps, I could have used only one at the end and add "on_dispute" field to the Transaction struct, but I decided to keep it as it is to show a good usecase for std::rc::Rc 
    disputes: HashMap<TransactionId, Rc<Transaction>>,
}

impl ClientProfile {
    pub fn new_with_defaults(id: ClientId) -> ClientProfile {
        Self::new(
            id,
            Currency::zero(),
            Currency::zero(),
            Currency::zero(),
            false,
            HashMap::new(),
            HashMap::new(),
        )
    }

    pub fn new(
        id: ClientId,
        available: Currency,
        held: Currency,
        total: Currency,
        locked: bool,
        open_transactions: HashMap<TransactionId, Rc<Transaction>>,
        disputes: HashMap<TransactionId, Rc<Transaction>>,
    ) -> ClientProfile {
        ClientProfile {
            id,
            available: available,
            held: held,
            total: total,
            locked: locked,
            open_transactions: open_transactions,
            disputes: disputes,
        }
    }

    /// It was assumed that both Deposits and Withdrawals can be disputed
    /// Malformed Deposits and Withdrawals (without an amount defined) are ignored
    pub fn process_new_transaction(&mut self, transaction: Transaction) {
        match transaction.tx_type {
            Type::Deposit => {
                if let Some(amount_to_deposit) = transaction.amount {
                    self.open_transactions
                        .entry(transaction.tx)
                        .or_insert_with(|| Rc::new(transaction));
                    self.available += amount_to_deposit;
                    self.total += amount_to_deposit;
                } else {
                    eprintln!(
                        "Igoring transaction malformed transaction {:?}..",
                        transaction
                    );
                }
            }

            Type::Withdrawal => {
                if self.locked {
                    eprintln!(
                        "Client's account {} is locked. Withdrawal not permitted ",
                        self.id
                    )
                } else if let Some(amount_to_withdraw) = transaction.amount {
                    self.open_transactions
                        .entry(transaction.tx)
                        .or_insert_with(|| Rc::new(transaction));
                    let to_debit = amount_to_withdraw;

                    if self.available - to_debit >= Currency::zero() {
                        self.available -= to_debit;
                        self.total -= to_debit;
                    } else {
                        eprintln!(
                            "{} amount exceeds available funds {}. Igoring transaction..",
                            to_debit, self.available
                        )
                    }
                } else {
                    eprintln!(
                        "Igoring transaction malformed transaction {:?}..",
                        transaction
                    );
                }
            }

            Type::Dispute => {
                if let Some(existing_transaction) = self.open_transactions.get(&transaction.tx) {
                    if let Some(disputed) = existing_transaction.amount {
                        self.held += disputed;
                        self.available -= disputed;
                        self.disputes
                            .entry(transaction.tx)
                            .or_insert_with(|| Rc::clone(existing_transaction));
                    }
                }
            }

            Type::Resolve => {
                if let Some(under_dispute) = self.disputes.get(&transaction.tx) {
                    if let Some(to_add) = under_dispute.amount {
                        self.held -= to_add;
                        self.available += to_add;
                        self.disputes.remove(&transaction.tx);
                        self.open_transactions.remove(&transaction.tx);
                    }
                }
            }

            Type::Chargeback => {
                if let Some(under_dispute) = self.disputes.get(&transaction.tx) {
                    if let Some(chargeback) = under_dispute.amount {
                        self.held -= chargeback;
                        self.total -= chargeback;
                        self.locked = true;
                        self.disputes.remove(&transaction.tx);
                        self.open_transactions.remove(&transaction.tx);
                    }
                }
            }
        }
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
