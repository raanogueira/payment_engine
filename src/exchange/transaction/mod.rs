use serde::Deserialize;
use std::str::FromStr;
use std::fmt;

/// Using rust_decimal to handle fixed precision decimals with no round-off errors. rust decimal was wrapped around a small library so it can be changed easily if needed
pub type Currency = rust_decimal::Decimal;

pub type ClientId = u16;

pub type TransactionId = u32;

#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Type {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}
///Instead of a general Transaction struct with an Enum specifying its type, a possible alternative could have been top level Transaction enum
///where each value of the enum would be a different type of transaction:
/// ```
/// struct BaseTransaction {
///     client: ClientId
///     id: TransactionId
/// }
///
/// struct MoneyTransaction {
///     base: BaseTransaction,
///     amount: Money
/// }
/// enum Transaction {
///     Deposit(MoneyTransaction),
///     Withdrawal(MoneyTransaction),
///     Dispute(BaseTransaction),
///     Resolve(BaseTransaction),
///     Chargeback(BaseTransaction),
/// }
/// ```
/// BaseTransaction would have the common fields for all types of transactions (client, tx id) and MoneyTransaction would be composed by BaseTransaction and a amount field
#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct Transaction {
    #[serde(rename(deserialize = "type"))]
    pub(crate) tx_type: Type,
    pub(crate) client: ClientId,
    pub(crate) tx: TransactionId,
    pub(crate) amount: Option<Currency>,
    #[serde(skip)]
    pub(crate) on_dispute: bool
}

impl Transaction {
    pub fn new(tx_type: Type, client: ClientId, tx: TransactionId, amount: Option<Currency>, on_dispute: bool) -> Transaction {
        Transaction {
            tx_type: tx_type,
            client: client,
            tx: tx,
            amount: amount,
            on_dispute: on_dispute
        }
    }

    pub fn start_dispute(&mut self) {
        self.on_dispute = true;
    }

    pub fn stop_dispute(&mut self) {
        self.on_dispute = false;
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:?},{},{},{:?},{}",
            self.tx_type, self.client, self.tx, self.amount, self.on_dispute
        )?;
        Ok(())
    }
}


//assume that all transactions are in the same currency
pub trait Money {
    fn zero() -> Currency;
    fn str(m: &str) -> Currency;
}

impl Money for Currency {
    fn zero() -> Currency {
        Currency::new(0, 4)
    }

    fn str(m: &str) -> Currency {
        Currency::from_str(m).unwrap()
    }
}
