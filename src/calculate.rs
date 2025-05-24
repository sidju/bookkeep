use std::collections::{
  BTreeMap,
  BTreeSet,
};
use serde::{Serialize};
use rust_decimal::Decimal;

use crate::types::*;

// Here we should do two things:
// - calculated sums for every relevant level
// - split/duplicated all the transactions to wherever relevant
// - drop the data we are done with? (accounts BTreeSet and groupings)
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Clone)]
pub struct Transfer {
  // BEWARE, ordering of members defines their ordering priority!
  pub date: time::Date,
  pub name: String,
  pub amount: Decimal,
  pub unique_id: String,
  // Other transfers in the same Transaction
  // (Their sum is asserted to be -1 * Transfer.amount)
  pub related_transfers: Vec<(String, Decimal)>,
}
#[derive(Debug, Serialize, Clone)]
pub struct SummedAccount {
  pub name: String,
  pub sum: Decimal,
  // We use a set to order the transfers, otherwise they come in the order
  // they are read from their groupings and are chunked per grouping.
  pub transfers: BTreeSet<Transfer>,
}
#[derive(Debug, Serialize)]
pub struct SummedGrouping {
  pub account_types: Vec<(AccountType, Decimal, Vec<SummedAccount>)>,
  pub account_sums: Vec<(String, Decimal, Vec<SummedAccount>)>,
}
#[derive(Debug, Serialize)]
pub struct SummedBookkeeping {
  pub name: String,
  pub total: SummedGrouping,
  #[serde(with = "tuple_vec_map")]
  pub groupings: Vec<(String, SummedGrouping)>,
}

pub fn calculate(data: RealBookkeeping) -> SummedBookkeeping {
  // We need somewhere to put the sums from the groupings
  let mut summed_periods = Vec::new();
  // Each level (total and per grouping) needs to aggregate accounts with all their transactions
  let mut total_accounts = BTreeMap::<String, SummedAccount>::new();
  // We iterate over the groupings:
  // - for each transaction, sum it to its accounts both in the grouping and the total
  // - for each account type, sum it from its accounts in the grouping
  // - for each accoust sum, sum it from its accounts in the grouping
  // At end of each grouping save its data into vec of SummedGrouping
  // At end of everything:
  // - for each account type, sum it from its accounts
  // - for each account sum, sum it from its accounts
  // Save that with the SummedGroupings into a Summed Bookkeeping
  for grouping in data.groupings {
    let mut grouping_accounts = BTreeMap::<String, SummedAccount>::new();

    for transaction in &grouping.transactions {
      // Track the per-transaction sum, should be 0 error otherwise
      let mut sum = Decimal::ZERO;
      // And save the data into relevant sum locations
      for (i, (account, amount)) in transaction.transfers.iter().enumerate() {
        sum += amount;

        // ensure that the account is declared
        if !data.accounts.contains(account) {
          panic!("Transaction {} used undeclared account {}, invalid.", transaction.name, account)
        };

        // Per-account summing
        let transfer = Transfer {
          date: transaction.date.clone(),
          name: transaction.name.clone(),
          amount: *amount,
          unique_id: format!("{}[{}][{}]", grouping.name, transaction.index, i),
          // Includes self, but who cares
          related_transfers: transaction.transfers.clone(),
        };
        // Global
        total_accounts.entry(account.to_owned())
          .and_modify(|mut x| {
            x.sum += amount;
            if ! x.transfers.insert(
              transfer.clone()
            ) { panic!("Identical transactions matching: {:?}", transaction) }
          })
          .or_insert(SummedAccount{
            name: account.to_owned(),
            sum: *amount,
            transfers: [transfer.clone()].into(),
          })
        ;
        // Local
        grouping_accounts.entry(account.to_owned())
          .and_modify(|mut x| {
            x.sum += amount;
            if !x.transfers.insert(
              transfer.clone()
            ) { panic!("Identical transactions matching: {:?}", transaction) }
          })
          .or_insert(SummedAccount{
            name: account.to_owned(),
            sum: *amount,
            transfers: [transfer.clone()].into(),
          })
        ;
      }
      if sum != Decimal::ZERO {
        panic!("Transaction {} didn't sum to 0, invalid. (sum: {})", transaction.name, sum);
      }
    }

    // After summing all transactions, use the account sums to sum account categories
    let mut account_sums = Vec::new();
    for (sum_name, accounts) in data.account_sums.iter() {
      let mut sum = Decimal::ZERO;
      let mut summed_accounts = Vec::new();
      for account in accounts {
        if let Some(acc) = grouping_accounts.get(account) {
          sum += acc.sum;
          summed_accounts.push(acc.clone());
        }
      }
      account_sums.push((sum_name.to_owned(), sum, summed_accounts));
    }

    // And the same for account types
    let mut account_types = Vec::new();
    for (type_name, accounts) in data.account_types.iter() {
      let mut sum = Decimal::ZERO;
      let mut summed_accounts = Vec::new();
      for account in accounts {
        if let Some(acc) = grouping_accounts.get(account) {
          sum += acc.sum;
          summed_accounts.push(acc.clone());
        }
      }
      account_types.push((*type_name, sum, summed_accounts));
    }

    // Whereafter we can add the summed grouping
    summed_periods.push((grouping.name, SummedGrouping{account_types, account_sums}));
  }

  // Finally do the same summing of account_sums and account_types as within
  // each grouping, this time using the total_accounts
  let mut account_sums = Vec::new();
  for (sum_name, accounts) in data.account_sums.iter() {
    let mut sum = Decimal::ZERO;
    let mut summed_accounts = Vec::new();
    for account in accounts {
      if let Some(acc) = total_accounts.get(account) {
        sum += acc.sum;
        summed_accounts.push(acc.clone());
      }
    }
    account_sums.push((sum_name.to_owned(), sum, summed_accounts));
  }

  let mut account_types = Vec::new();
  for (type_name, accounts) in data.account_types.iter() {
    let mut sum = Decimal::ZERO;
    let mut summed_accounts = Vec::new();
    for account in accounts {
      if let Some(acc) = total_accounts.get(account) {
        sum += acc.sum;
        summed_accounts.push(acc.clone());
      }
    }
    account_types.push((*type_name, sum, summed_accounts));
  }

  // Whereafter we can add the summed grouping
  SummedBookkeeping{
    name: data.name,
    total: SummedGrouping{
      account_types,
      account_sums,
    },
    groupings: summed_periods,
  }
}
