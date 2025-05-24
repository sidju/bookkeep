use cursive::backends::crossterm::crossterm::tty::IsTty;


mod types;
use types::*;
mod file_io;
use file_io::*;
mod calculate;
use calculate::*;
mod tui;
use tui::*;


fn main() {
  let mut io = StdFileIO{};

  let raw = io.read_path(std::path::Path::new("bookkeeping.yaml"));
  let parsed: Bookkeeping = serde_yaml::from_str(&raw)
    .expect("Invalid format at year.yaml")
  ;
  let real = parsed.realize(&mut io);
  // Do all the calculations
  let calc = calculate(real);

  if std::io::stdout().is_tty() {
    run_tui(calc);
  }
  else {
    println!("{}", serde_yaml::to_string(&calc).unwrap());
  }
}
