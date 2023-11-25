//! Declares the types we work with and their serialization.
//! (Note also the tests to validate the serialization at the bottom.)

use serde::{
  Serialize,
  Deserialize,
};

use std::path::PathBuf;
use serde_yaml::{
  from_str,
};
use time::Date;

use super::FileIO;
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct RealYear {
  name: String,
  months: Vec<RealMonth>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Year {
  name: String,
  months: Vec<Month>,
}
impl Year {
  pub fn realize(mut self, io: &mut impl FileIO) -> RealYear {
    RealYear{
      name: self.name,
      months: self.months.drain(..).map(|m| m.realize(io)).collect(),
    }
  }
}


#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct RealMonth {
  name: String,
  transactions: Vec<Transaction>
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Month {
  /// The yaml is inlined
  Inlined(RealMonth),
  /// A path to a file containing the yaml is given
  Path(PathBuf),
}
impl Month {
  pub fn realize(self, io: &mut impl FileIO) -> RealMonth {
    match self {
      Month::Inlined(i) => i,
      Month::Path(path) => {
        let raw = io.read_path(&path);
        from_str(&raw).expect(&format!("Invalid format at {}", path.display()))
      }
    }
  }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
  name: String,
  date: Date,
  amount: i32,
  account: String,
}
#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn transaction() {
    let raw = "---
name: test
date: 2023-12-31
amount: -400
account: debts
";
    let parsed: Transaction = from_str(&raw).unwrap();
    assert_eq!(
      parsed,
      Transaction{
        name: "test".to_owned(),
        date: Date::from_calendar_date(2023, time::Month::December, 31).unwrap(),
        amount: -400,
        account: "debts".to_owned(),
      },
      "Received result (left) didn't match expected (right)."
    );
  }

  #[test]
  fn inner_month_and_inline_month() {
    let raw = "---
name: inline-month
transactions:
- name: inline-transaction
  date: 2023-12-31
  amount: -400
  account: debts
";
    let expected = RealMonth{
      name: "inline-month".to_owned(),
      transactions: vec![Transaction{
        name: "inline-transaction".to_owned(),
        date: Date::from_calendar_date(2023, time::Month::December, 31).unwrap(),
        amount: -400,
        account: "debts".to_owned(),
      }],
    };
    let parsed: RealMonth = from_str(&raw).unwrap();
    assert_eq!(
      &parsed,
      &expected,
      "Received result (left) didn't match expected (right)."
    );
    // This should also parse into a Month and .realize() give the same values
    let parsed: Month = from_str(&raw).unwrap();
    assert_eq!(
      parsed.realize(&mut crate::DummyFileIO{}), // No file IO should be needed
      expected,
      "Received result (left) didn't match expected (right)."
    );
  }

  #[test]
  fn month_path() {
    let raw = "month.yaml";
    let parsed: Month = from_str(&raw).unwrap();
    assert_eq!(
      parsed.realize(&mut crate::FakeFileIO::new()),
      RealMonth{
        name: "file-month".to_owned(),
        transactions: vec![Transaction{
          name: "file-transaction".to_owned(),
          date: Date::from_calendar_date(2023, time::Month::January, 30).unwrap(),
          amount: -300,
          account: "debts".to_owned(),
        }],
      },
      "Received result (left) didn't match expected (right)."
    );
  }

  #[test]
  fn year() {
    let raw = "---
name: test-year
months:
- month.yaml
- name: inline-month
  transactions:
  - name: inline-transaction
    date: 2023-12-31
    amount: 300
    account: money
";
    let parsed: Year = from_str(&raw).unwrap();
    assert_eq!(
      parsed.realize(&mut crate::FakeFileIO::new()),
      RealYear{
        name: "test-year".to_owned(),
        months: vec![
          // From given path
          RealMonth{
            name: "file-month".to_owned(),
            transactions: vec![Transaction{
              name: "file-transaction".to_owned(),
              date: Date::from_calendar_date(2023, time::Month::January, 30).unwrap(),
              amount: -300,
              account: "debts".to_owned(),
            }],
          },
          // From inline data
          RealMonth{
            name: "inline-month".to_owned(),
            transactions: vec![Transaction{
              name: "inline-transaction".to_owned(),
              date: Date::from_calendar_date(2023, time::Month::December, 31).unwrap(),
              amount: 300,
              account: "money".to_owned(),
            }],
          },
        ],
      },
      "Received result (left) didn't match expected (right)."
    );
  }
}
