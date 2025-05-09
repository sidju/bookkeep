use crate::*;
// Created distinct from Sums and AllSums, since it should print ordered.
// (We still want to use Sums and AllSums for simpler code when calculating)

// We wish to group accounts by their type, just as in the input file
#[derive(Debug, serde::Serialize)]
pub struct AccountGroupSummary {
  pub total: Decimal,
  #[serde(flatten, with = "tuple_vec_map")]
  pub accounts: Vec<(String, Decimal)>,
}
#[derive(Debug, serde::Serialize)]
pub struct GroupingSummary {
  #[serde(with = "tuple_vec_map")]
  pub account_sums: Vec<(String, AccountGroupSummary)>,
  #[serde(with = "tuple_vec_map")]
  pub account_types: Vec<(AccountType, AccountGroupSummary)>,
}
impl GroupingSummary {
  pub fn create(
    calculations: &Sums,
    bookkeeping: &Bookkeeping,
    total: bool,
  ) -> Self {
    let mut types = Vec::new();
    // First get the account types
    for (account_type, accounts) in bookkeeping.accounts.iter() {
      // This is only relevant to show in the yearly summary
      if *account_type == AccountType::YearlyResult && !total { continue; }
      let mut local = AccountGroupSummary{
        total: *calculations.account_types.get(account_type).unwrap_or(&Decimal::ZERO),
        accounts: Vec::new(),
      };
      for account in accounts {
        local.accounts.push((
          account.clone(),
          *calculations.accounts.get(account).unwrap_or(&Decimal::ZERO),
        ));
      }
      types.push((*account_type, local));
    }
    // Then get the account sums
    let mut sums = Vec::new();
    for (sum_name, accounts) in bookkeeping.account_sums.iter() {
      let mut local = AccountGroupSummary{
        total: *calculations.account_sums.get(sum_name).unwrap_or(&Decimal::ZERO),
        accounts: Vec::new(),
      };
      for account in accounts {
        local.accounts.push((
          account.clone(),
          *calculations.accounts.get(account).unwrap_or(&Decimal::ZERO),
        ));
      }
      sums.push((sum_name.clone(), local));
    }
    Self{
      account_sums: sums,
      account_types: types,
    }
  }
}
#[derive(Debug, serde::Serialize)]
pub struct Summary {
  pub total: GroupingSummary,
  #[serde(with = "tuple_vec_map")]
  pub groupings: Vec<(String, GroupingSummary)>
}
impl Summary {
  pub fn create(
    calculations: &AllSums,
    bookkeeping: &Bookkeeping,
  ) -> Self {
    Self {
      total: GroupingSummary::create(
        &calculations.global,
        bookkeeping,
        true,
      ),
      groupings: calculations.groupings.iter()
        .map(|(name, sums)| {(
          name.clone(),
          GroupingSummary::create(
            sums,
            bookkeeping,
            false,
          ),
        )})
        .collect(),
    }
  }
}
