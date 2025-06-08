use super::*;

use cursive::{
  Cursive,
  CursiveExt,
  view::{
    Nameable,
  },
  views::{
    LinearLayout,
    TextView,
    ScrollView,
  },
};
use cursive_table_view::{
  TableView,
  TableViewItem,
};
use cursive_tree_view::{
  Placement,
  TreeView,
};

// The intended approximate layout is:
// ---------------------------------------------------------------------------
//  (Probably use the cursive| total: <sum of assed, creditor, debtor> ^
//  table views, so we can   |   accounts_by_sum: ^
//  set click events)        |     car: 3063.50 ^
//  +--------------+------+  |       car_insurance: 2303
//  | Assets       | 7898 |  |       car_loan_interest: 760.50
//  +--------------+------+  |     home: >
//  | in_account   | 3008 |  |     dog: >
//  | card_account | 4890 |  |   accounts_by_type: >
//  +--------------+------+  | May: (monetary change for the month)
//                           |   accounts_by_sum: ^
//  +-----------+-------+    |     car: 1063.50 ^
//  | Creditors | 49723 |    |       car_insurance: 718.50
//  +-----------+-------+    |       car_loan_interest: 345
//  | car_loan  | 49723 |    | June: >
//  +-----------+-------+    |
//                           | (Probably use cursive tree view for this,
//  +---------+              | since that offers showing the same thing
//  | Debtors |              | from multiple periods and total at once)
//  +---------+              |
// ---------------------------------------------------------------------------

// An additional layout should be vertical tables of account balance per day of
// the bookkeeping, with functional reload from file.
// That would really help verifying that you are inputting correctly by checking
// the end-of-day account balances against the bank statement.
//
// (This handles that you often input data from different accounts separately,
// which really complicates things when you transfer money between them and wish
// to check their balances as you go on filling them in. It is a constant offset
// and you can comment out the relevant transfers, but it is not too hard a
// feature to build.)
//
// +---------------+------------+--------------+
// | at end of day | in_account | card_account |
// +---------------+------------+--------------+
// | 2025-04-24    | 5000.85    | 2085.94      |
// | 2025-04-25    | ...
// ...

fn grouping_summary_to_tree_entries(
  tree: &mut TreeView<String>,
  gs: &SummedGrouping,
  row: usize,
) {
  let r = tree.insert_item(
    format!("Account types"),
    Placement::LastChild,
    row,
  ).expect("The row on which grouping_summary_to_tree_entries is called on must not be collapsed");
  for (t, sum, accounts) in &gs.account_types {
    let inner_r = tree.insert_item(
      format!("{:?}: ({})", t, sum),
      Placement::LastChild,
      r,
    ).unwrap();
    for account in accounts {
      let innermost_r = tree.insert_item(
        format!("{}: ({})", account.name, account.sum),
        Placement::LastChild,
        inner_r,
      ).unwrap();
      for transfer in &account.transfers {
        tree.insert_item(
          format!("{}, {}: ({} -> {})", transfer.name, transfer.date, transfer.amount, transfer.resulting_balance),
          Placement::LastChild,
          innermost_r,
        );
      }
      tree.set_collapsed(innermost_r, true);
    }
    tree.set_collapsed(inner_r, true);
  }
  tree.set_collapsed(r, true);

  let r = tree.insert_item(
    format!("Account sums"),
    Placement::After,
    r,
  ).unwrap();
  for (name, sum, accounts) in &gs.account_sums {
    let inner_r = tree.insert_item(
      format!("{}: ({})", name, sum),
      Placement::LastChild,
      r,
    ).unwrap();
    for account in accounts {
      let innermost_r = tree.insert_item(
        format!("{}: ({})", account.name, account.sum),
        Placement::LastChild,
        inner_r,
      ).unwrap();
      for transfer in &account.transfers {
        tree.insert_item(
          format!("{}, {}: ({} -> {})", transfer.name, transfer.date, transfer.amount, transfer.resulting_balance),
          Placement::LastChild,
          innermost_r,
        );
      }
      tree.set_collapsed(innermost_r, true);
    }
    tree.set_collapsed(inner_r, true);
  }
  tree.set_collapsed(r, true);
}

pub fn run_tui(
  summary: SummedBookkeeping,
) {
  let mut siv = Cursive::new();
  siv.add_global_callback('q', |s| s.quit());

  // Create the main view
  let mut detail_tree = TreeView::<String>::new()
  ;
  // First insert totals in one container
  let mut row = detail_tree.insert_item(
    format!("Totals"),
    Placement::After,
    0,
  ).unwrap();
  grouping_summary_to_tree_entries(
    &mut detail_tree,
    &summary.total,
    row,
  );
  detail_tree.set_collapsed(0, true);

  // Then one container for each grouping
  for (name, gs) in &summary.groupings {
    row = detail_tree.insert_item(
      format!("{}", name),
      Placement::After,
      row,
    ).unwrap();
    grouping_summary_to_tree_entries(
      &mut detail_tree,
      gs,
      row
    );
    detail_tree.set_collapsed(row, true);
  }
  //let main = LinearView::vertical()
  //  .child(
  //  )
  //  .child(
  //  )
  //;
  siv.add_layer(
    ScrollView::new(detail_tree).with_name("detail_tree")
  );

  siv.run();
}
