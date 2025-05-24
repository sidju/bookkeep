//! Declares the types we work with and their serialization.
//! (Note also the tests to validate the serialization at the bottom.)

use serde::{
  Serialize,
  Deserialize,
};
use rust_decimal::Decimal;

use std::path::PathBuf;
use serde_yaml::from_str;
use time::Date;

use super::FileIO;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum AccountType {
  // Source of money
  Income,
  // Owes money to you
  Debtor,
  // Value you have right now
  Asset,
  // You owe money to
  Creditor,
  // Sink of money
  Expense,
  // A way to signify that an account is just used to set initial balance for
  // another account. (When you have multiple periods of bookkeeping you can
  // validate the initial value against the calculated result of the previous
  // period.)
  YearlyResult,
}
#[derive(Debug, PartialEq, Serialize)]
pub struct RealBookkeeping {
  // A recognizeable name. Basically just a comment
  pub name: String,
  // Easy way to check if account has been declared
  // (Bonus, iterate in alphabetical order)
  pub accounts: std::collections::BTreeSet<String>,
  // All accounts and their type with order preserved
  #[serde(with = "tuple_vec_map")]
  pub account_types: Vec<(AccountType, Vec<String>)>,
  // Secondary sums of these are created from the account sums
  #[serde(with = "tuple_vec_map")]
  pub account_sums: Vec<(String, Vec<String>)>,
  // Contains all the transaction data
  pub groupings: Vec<RealGrouping>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Bookkeeping {
  pub name: String,
  #[serde(with = "tuple_vec_map")]
  pub accounts: Vec<(AccountType, Vec<String>)>,
  #[serde(with = "tuple_vec_map")]
  pub account_sums: Vec<(String, Vec<String>)>,
  pub groupings: Vec<Grouping>,
}
impl Bookkeeping {
  pub fn realize(mut self, io: &mut impl FileIO) -> RealBookkeeping {
    let real = RealBookkeeping{
      name: self.name,
      accounts: self.accounts.iter()
        .fold(std::collections::BTreeSet::new(), |mut m, (_, accounts)| {
          for account in accounts {
            m.insert(account.to_owned());
          }
          m
        }),
      account_types: self.accounts,
      account_sums: self.account_sums,
      groupings: self.groupings.drain(..).map(|m| m.realize(io)).collect(),
    };
    real.groupings.iter().fold(std::collections::HashSet::new(), |mut s, m|{
      if !s.insert(&m.name) { panic!("Duplicate grouping {}", m.name); }
      s
    });
    real
  }
}


#[derive(Debug, PartialEq, Serialize)]
pub struct RealGrouping {
  pub name: String,
  pub transactions: Vec<RealTransaction>,
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Grouping {
  pub name: String,
  pub transactions: Transactions
}
impl Grouping {
  pub fn realize(self, io: &mut impl FileIO) -> RealGrouping {
    RealGrouping{
      name: self.name,
      transactions: self.transactions.realize(io)
    }
  }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum Transactions {
  /// The yaml is inlined
  Inlined(Vec<Transaction>),
  /// A path to a file containing the yaml is given
  Paths(Vec<PathBuf>),
}
impl Transactions {
  fn read(self, io: &mut impl FileIO) -> Vec<Transaction> {
    match self {
      Transactions::Inlined(i) => i,
      Transactions::Paths(paths) => {
        let mut transactions = Vec::new();
        for path in paths {
          let raw = io.read_path(&path);
          transactions.append(&mut from_str(&raw).expect(&format!("Invalid format at {}", path.display())))
        }
        transactions
      }
    }
  }
  pub fn realize(self, io: &mut impl FileIO) -> Vec<RealTransaction> {
    self.read(io).drain(..).enumerate().map(|(i,x)| RealTransaction{
      name: x.name,
      date: x.date,
      index: i,
      transfers: x.transfers,
      comments: x.comments,
    }).collect()
  }
}

#[derive(Debug, PartialEq, Serialize, Clone)]
pub struct RealTransaction {
  pub name: String,
  pub date: Date,
  pub index: usize,
  #[serde(with = "tuple_vec_map")]
  pub transfers: Vec<(String, Decimal)>,
  pub comments: std::collections::HashMap<String, String>,
}
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Transaction {
  pub name: String,
  pub date: Date,
  #[serde(with = "tuple_vec_map")]
  pub transfers: Vec<(String, Decimal)>,
  // To keep paths to receipts/bills/descriptions...
  #[serde(flatten)]
  pub comments: std::collections::HashMap<String, String>,
}
// 
// #[cfg(test)]
// mod test {
//   use super::*;
//   use std::collections::HashMap;
// 
//   #[test]
//   fn transaction() {
//     let raw = "---
// name: test
// date: 2023-12-31
// transfers:
//   debts: -400.00
//   money: 400.00
// ";
//     let parsed: Transaction = from_str(&raw).unwrap();
//     assert_eq!(
//       parsed,
//       Transaction{
//         name: "test".to_owned(),
//         date: Date::from_calendar_date(2023, time::Month::December, 31).unwrap(),
//         transfers: vec![
//           ("debts".to_owned(), Decimal::from(-400)),
//           ("money".to_owned(), Decimal::from(400)),
//         ],
//         comments: HashMap::new(),
//       },
//       "Received result (left) didn't match expected (right)."
//     );
//   }
// 
//   #[test]
//   fn inner_grouping_and_inline_grouping() {
//     let raw = "---
// !Inlined
// name: inline-grouping
// transactions:
// - name: inline-transaction
//   date: 2023-12-31
//   transfers:
//     debts: -400.00
//     money: 400.00
// ";
//     let expected = RealGrouping{
//       name: "inline-grouping".to_owned(),
//       transactions: vec![Transaction{
//         name: "inline-transaction".to_owned(),
//         date: Date::from_calendar_date(2023, time::Month::December, 31).unwrap(),
//         transfers: vec![("debts".to_owned(), (-400).into()),("money".to_owned(), 400.into())],
//         comments: HashMap::new(),
//       }],
//     };
//     let parsed: RealGrouping = from_str(&raw).unwrap();
//     assert_eq!(
//       &parsed,
//       &expected,
//       "Received result (left) didn't match expected (right)."
//     );
//     //  This should also parse into a Grouping and .realize() give the same values
//     let parsed: Grouping = from_str(&raw).unwrap();
//     assert_eq!(
//       parsed.realize(&mut crate::DummyFileIO{}), //  No file IO should be needed
//       expected,
//       "Received result (left) didn't match expected (right)."
//     );
//   }
// 
//   #[test]
//   fn grouping_path() {
//     let raw = "!Path grouping.yaml";
//     let parsed: Grouping = from_str(&raw).unwrap();
//     assert_eq!(
//       parsed.realize(&mut crate::FakeFileIO::new()),
//       RealGrouping{
//         name: "file-grouping".to_owned(),
//         transactions: vec![Transaction{
//           name: "file-transaction".to_owned(),
//           date: Date::from_calendar_date(2023, time::Month::January, 30).unwrap(),
//           transfers: vec![("debts".to_owned(), (-300).into()),("money".to_owned(), 300.into())],
//           comments: HashMap::new(),
//         }],
//       },
//       "Received result (left) didn't match expected (right)."
//     );
//   }
// 
//   #[test]
//   fn bookkeeping() {
//     let raw = "---
// name: test-bookkeeping
// accounts:
//   starting_money: initial_value
//   money: asset
//   groceries: expense
//   salary: income
// groupings:
// - !Inlined
//   name: Start of year
//   transactions:
//   - name: Money from last year
//     date: 2023-01-01
//     transfers:
//       starting_money: -45_000
//       money: 45_000
// - !Path grouping.yaml
// - !Inlined
//   name: inline-grouping
//   transactions:
//   - name: inline-transaction
//     date: 2023-12-31
//     transfers:
//       money: -300
//       groceries: 300
//     receipt: ./receipts/groceries-2023-12-31.jpeg
// ";
//     let parsed: Bookkeeping = from_str(&raw).unwrap();
//     assert_eq!(
//       parsed.realize(&mut crate::FakeFileIO::new()),
//       RealBookkeeping{
//         name: "test-bookkeeping".to_owned(),
//         accounts: [
//           ("starting_money".to_owned(), AccountType::YearlyResult),
//           ("money".to_owned(), AccountType::Asset),
//           ("groceries".to_owned(), AccountType::Expense),
//           ("salary".to_owned(), AccountType::Income),
//         ].into(),
//         groupings: vec![
//           RealGrouping{
//             name: "Start of year".to_owned(),
//             transactions: vec![Transaction{
//               name: "Money from last year".to_owned(),
//               date: Date::from_calendar_date(2023, time::Month::January, 1).unwrap(),
//               transfers: vec![
//                 ("starting_money".to_owned(), (-45000).into()),
//                 ("money".to_owned(), 45000.into()),
//               ],
//               comments: HashMap::new(),
//             }],
//           },
//           //  From given path
//           RealGrouping{
//             name: "file-grouping".to_owned(),
//             transactions: vec![Transaction{
//               name: "file-transaction".to_owned(),
//               date: Date::from_calendar_date(2023, time::Month::January, 30).unwrap(),
//               transfers: vec![("debts".to_owned(), (-300).into()),("money".to_owned(), 300.into())],
//               comments: HashMap::new(),
//             }],
//           },
//           //  From inline data
//           RealGrouping{
//             name: "inline-grouping".to_owned(),
//             transactions: vec![Transaction{
//               name: "inline-transaction".to_owned(),
//               date: Date::from_calendar_date(2023, time::Month::December, 31).unwrap(),
//               transfers: vec![("money".to_owned(), (-300).into()),("groceries".to_owned(), 300.into())],
//               comments: [
//                 ("receipt".to_owned(), "./receipts/groceries-2023-12-31.jpeg".to_owned()),
//               ].into(),
//             }],
//           },
//         ],
//       },
//       "Received result (left) didn't match expected (right)."
//     );
//   }
// 
// }
// 
