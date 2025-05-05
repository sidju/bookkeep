use std::collections::BTreeMap;
use serde::{Serialize};
use rust_decimal::Decimal;

use crate::types::*;

// We use BTreeMap to give a consistent alphabetic sorting of accounts
#[derive(Debug, Serialize)]
pub struct Sums {
  accounts: BTreeMap<String, Decimal>,
  account_sums: BTreeMap<String, Decimal>,
}
impl Sums {
  pub fn new() -> Self {
    Self{
      accounts: BTreeMap::new(),
      account_sums: BTreeMap::new(),
    }
  }
}
#[derive(Debug, Serialize)]
pub struct AllSums {
  global: Sums,
  // Use Vec of tuples, so that the order the groupings were given in is preserved
  #[serde(with = "tuple_vec_map")]
  groupings: Vec<(String, Sums)>,
}

pub fn calculate(data: &RealBookkeeping) -> AllSums {
  let mut sums = AllSums{
    global: Sums::new(),
    groupings: Vec::new(),
  };
  // We iterate over the periods, both
  // - summing all accounts for each period,
  // - summing the account categories for each period
  // - summing all accounts for all periods and
  // - summing the account categories for each period
  for grouping in &data.groupings {
    let mut local = Sums::new();
    for transaction in &grouping.transactions {
      // Track the per-transaction sum, should be 0 error otherwise
      let mut sum = Decimal::ZERO;
      // And save the data into relevant sum locations
      for (account, amount) in &transaction.transfers {
        sum += amount;

        // ensure that the account is declared
        match data.accounts.get(account) {
          None => panic!("Transaction {} used undeclared account {}, invalid.", transaction.name, account),
          Some(x) => (),
        };

        // Per-account summing
        // Global
        sums.global.accounts.entry(account.to_owned())
          .and_modify(|x| *x += amount)
          .or_insert(*amount)
        ;
        // Local
        local.accounts.entry(account.to_owned())
          .and_modify(|x| *x += amount)
          .or_insert(*amount)
        ;
      }
      if sum != Decimal::ZERO {
        panic!("Transaction {} didn't sum to 0, invalid.", transaction.name);
      }
    }

    // After summing all transactions, use the account sums to sum account categories
    for (sum_name, accounts) in data.account_sums.iter() {
      let mut sum = Decimal::ZERO;
      for account in accounts {
        sum += local.accounts.get(account).unwrap_or(&Decimal::ZERO);
      }
      local.account_sums.insert(sum_name.to_owned(), sum);

      // We might as well add this to the global category sum as well here
      sums.global.account_sums.entry(sum_name.to_owned())
        .and_modify(|x| *x += sum)
        .or_insert(sum)
      ;
    }

    sums.groupings.push((grouping.name.clone(), local));
  }
  sums
}
