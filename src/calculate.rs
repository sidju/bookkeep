use std::collections::BTreeMap;
use serde::{Serialize};

use crate::types::*;

// We use BTreeMap to give a consistent alphabetic sorting of accounts
#[derive(Debug, Serialize)]
pub struct Sums {
  accounts: BTreeMap<String, i32>,
  account_types: BTreeMap<AccountType, i32>,
}
impl Sums {
  pub fn new() -> Self {
    Self{
      accounts: BTreeMap::new(),
      account_types: BTreeMap::new(),
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
  // - summing all account types for each period,
  // - summing all accounts for all periods and
  // - summing all account types for all periods
  for grouping in &data.groupings {
    let mut local = Sums::new();
    for transaction in &grouping.transactions {
      // Track the per-transaction sum, should be 0 error otherwise
      let mut sum = 0;
      // And save the data into relevant sum locations
      for (account, amount) in &transaction.transfers {
        sum += amount;

        // per account type summing first, as it may fail if the account isn't
        // declared
        let account_type = match data.accounts.get(account) {
          None => panic!("Transaction {} used undeclared account {}, invalid.", transaction.name, account),
          Some(x) => x,
        };
        // Global
        sums.global.account_types.entry(*account_type)
          // If present, run this closure on a mut reference
          .and_modify(|x| *x += amount)
          // If absent insert this
          .or_insert(*amount)
        ;
        // Local
        local.account_types.entry(*account_type)
          .and_modify(|x| *x += amount)
          .or_insert(*amount)
        ;

        // Then the per-account summing
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
      if sum != 0 { panic!("Transaction {} didn't sum to 0, invalid.", transaction.name); }
    }
    sums.groupings.push((grouping.name.clone(), local));
  }
  sums
}
