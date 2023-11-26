//! Declares the types we work with and their serialization.
//! (Note also the tests to validate the serialization at the bottom.)

use serde::{
  Serialize,
  Deserialize,
};

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
  InitialValue,
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct RealBookkeeping {
  // A recognizeable name. Basically just a comment
  pub name: String,
  // Declare all accounts and their type
  pub accounts: std::collections::HashMap<String, AccountType>,
  // Contains all the transaction data
  pub groupings: Vec<RealGrouping>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Bookkeeping {
  pub name: String,
  pub accounts: std::collections::HashMap<String, AccountType>,
  pub groupings: Vec<Grouping>,
}
impl Bookkeeping {
  pub fn realize(mut self, io: &mut impl FileIO) -> RealBookkeeping {
    let real = RealBookkeeping{
      name: self.name,
      accounts: self.accounts,
      groupings: self.groupings.drain(..).map(|m| m.realize(io)).collect(),
    };
    real.groupings.iter().fold(std::collections::HashSet::new(), |mut s, m|{
      if !s.insert(&m.name) { panic!("Duplicate grouping {}", m.name); }
      s
    });
    real
  }
}


#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct RealGrouping {
  pub name: String,
  pub transactions: Vec<Transaction>
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Grouping {
  /// The yaml is inlined
  Inlined(RealGrouping),
  /// A path to a file containing the yaml is given
  Path(PathBuf),
}
impl Grouping {
  pub fn realize(self, io: &mut impl FileIO) -> RealGrouping {
    match self {
      Grouping::Inlined(i) => i,
      Grouping::Path(path) => {
        let raw = io.read_path(&path);
        from_str(&raw).expect(&format!("Invalid format at {}", path.display()))
      }
    }
  }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
  pub name: String,
  pub date: Date,
  #[serde(with = "tuple_vec_map")]
  pub transfers: Vec<(String, i32)>,
  // To keep paths to receipts/bills/descriptions...
  #[serde(flatten)]
  pub comments: std::collections::HashMap<String, String>,
}

#[cfg(test)]
mod test {
  use super::*;
  use std::collections::HashMap;

  #[test]
  fn transaction() {
    let raw = "---
name: test
date: 2023-12-31
transfers:
  debts: -400
  money: 400
";
    let parsed: Transaction = from_str(&raw).unwrap();
    assert_eq!(
      parsed,
      Transaction{
        name: "test".to_owned(),
        date: Date::from_calendar_date(2023, time::Month::December, 31).unwrap(),
        transfers: vec![
          ("debts".to_owned(), -400),
          ("money".to_owned(), 400),
        ],
        comments: HashMap::new(),
      },
      "Received result (left) didn't match expected (right)."
    );
  }

  #[test]
  fn inner_grouping_and_inline_grouping() {
    let raw = "---
name: inline-grouping
transactions:
- name: inline-transaction
  date: 2023-12-31
  transfers:
    debts: -400
    money: 400
";
    let expected = RealGrouping{
      name: "inline-grouping".to_owned(),
      transactions: vec![Transaction{
        name: "inline-transaction".to_owned(),
        date: Date::from_calendar_date(2023, time::Month::December, 31).unwrap(),
        transfers: vec![("debts".to_owned(), -400),("money".to_owned(), 400)],
        comments: HashMap::new(),
      }],
    };
    let parsed: RealGrouping = from_str(&raw).unwrap();
    assert_eq!(
      &parsed,
      &expected,
      "Received result (left) didn't match expected (right)."
    );
    // This should also parse into a Grouping and .realize() give the same values
    let parsed: Grouping = from_str(&raw).unwrap();
    assert_eq!(
      parsed.realize(&mut crate::DummyFileIO{}), // No file IO should be needed
      expected,
      "Received result (left) didn't match expected (right)."
    );
  }

  #[test]
  fn grouping_path() {
    let raw = "grouping.yaml";
    let parsed: Grouping = from_str(&raw).unwrap();
    assert_eq!(
      parsed.realize(&mut crate::FakeFileIO::new()),
      RealGrouping{
        name: "file-grouping".to_owned(),
        transactions: vec![Transaction{
          name: "file-transaction".to_owned(),
          date: Date::from_calendar_date(2023, time::Month::January, 30).unwrap(),
          transfers: vec![("debts".to_owned(), -300),("money".to_owned(), 300)],
          comments: HashMap::new(),
        }],
      },
      "Received result (left) didn't match expected (right)."
    );
  }

  #[test]
  fn bookkeeping() {
    let raw = "---
name: test-bookkeeping
accounts:
  starting_money: initial_value
  money: asset
  groceries: expense
  salary: income
groupings:
- name: Start of year
  transactions:
  - name: Money from last year
    date: 2023-01-01
    transfers:
      starting_money: -45000
      money: 45000
- grouping.yaml
- name: inline-grouping
  transactions:
  - name: inline-transaction
    date: 2023-12-31
    transfers:
      money: -300
      groceries: 300
    receipt: ./receipts/groceries-2023-12-31.jpeg
";
    let parsed: Bookkeeping = from_str(&raw).unwrap();
    assert_eq!(
      parsed.realize(&mut crate::FakeFileIO::new()),
      RealBookkeeping{
        name: "test-bookkeeping".to_owned(),
        accounts: [
          ("starting_money".to_owned(), AccountType::InitialValue),
          ("money".to_owned(), AccountType::Asset),
          ("groceries".to_owned(), AccountType::Expense),
          ("salary".to_owned(), AccountType::Income),
        ].into(),
        groupings: vec![
          RealGrouping{
            name: "Start of year".to_owned(),
            transactions: vec![Transaction{
              name: "Money from last year".to_owned(),
              date: Date::from_calendar_date(2023, time::Month::January, 1).unwrap(),
              transfers: vec![("starting_money".to_owned(), -45000),("money".to_owned(), 45000)],
              comments: HashMap::new(),
            }],
          },
          // From given path
          RealGrouping{
            name: "file-grouping".to_owned(),
            transactions: vec![Transaction{
              name: "file-transaction".to_owned(),
              date: Date::from_calendar_date(2023, time::Month::January, 30).unwrap(),
              transfers: vec![("debts".to_owned(), -300),("money".to_owned(), 300)],
              comments: HashMap::new(),
            }],
          },
          // From inline data
          RealGrouping{
            name: "inline-grouping".to_owned(),
            transactions: vec![Transaction{
              name: "inline-transaction".to_owned(),
              date: Date::from_calendar_date(2023, time::Month::December, 31).unwrap(),
              transfers: vec![("money".to_owned(), -300),("groceries".to_owned(), 300)],
              comments: [
                ("receipt".to_owned(), "./receipts/groceries-2023-12-31.jpeg".to_owned()),
              ].into(),
            }],
          },
        ],
      },
      "Received result (left) didn't match expected (right)."
    );
  }

}
