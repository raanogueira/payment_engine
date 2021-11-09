use serde::Deserialize;
use std::str::FromStr;

/// Using rust_decimal to handle fixed precision decimals with no round-off errors. rust decimal was wrapped around a small library so it can be changed easily if needed 
pub type Currency = rust_decimal::Decimal;

pub type ClientId = u16;

pub type TransactionId = u32;

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
    pub tx_type: Type,
    pub client: ClientId,
    pub tx: TransactionId,
    pub amount: Option<Currency>,
}