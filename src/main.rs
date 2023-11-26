mod types;
use types::*;
mod file_io;
use file_io::*;
mod calculate;
use calculate::*;

// Created distinct from Sums and AllSums, since it should print ordered.
// (We still want to use Sums and AllSums for simpler code when calculating)
#[derive(Debug, serde::Serialize)]
pub struct PartialSummary {
  #[serde(with = "tuple_vec_map")]
  accounts: Vec<(String, i32)>,
  #[serde(with = "tuple_vec_map")]
  account_types: Vec<(AccountType, i32)>,
}
#[derive(Debug, serde::Serialize)]
pub struct Summary {
  global: PartialSummary,
  #[serde(with = "tuple_vec_map")]
  groupings: Vec<(String, PartialSummary)>,
}

fn summarize(data: &RealBookkeeping, sums: &AllSums) -> Summary {
  todo!()
}

fn main() {
  let mut io = StdFileIO{};

  let raw = io.read_path(std::path::Path::new("bookkeeping.yaml"));
  let parsed: Bookkeeping = serde_yaml::from_str(&raw)
    .expect("Invalid format at year.yaml")
  ;
  let real = parsed.realize(&mut io);
  // Do all the calculations
  let calc = calculate(&real);
  println!("{}", serde_yaml::to_string(&calc).unwrap());
}
