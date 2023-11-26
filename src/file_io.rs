//! Only exists so that I can hand in test versions of FileIO to some tests.
//! Aside from that we only use std::fs::read_to_string().

use std::path::Path;

pub trait FileIO {
  fn read_path(&mut self, path: &Path) -> String;
}

pub struct StdFileIO {
}
impl FileIO for StdFileIO {
  fn read_path(&mut self, path: &Path) -> String {
    std::fs::read_to_string(path)
      .expect(&format!("File not found: {}", path.display()))
  }
}

#[cfg(test)]
pub struct FakeFileIO {
  fake_fs: std::collections::HashMap<&'static Path, &'static str>,
}
#[cfg(test)]
impl FakeFileIO {
  pub fn new() -> Self {
    let mut out = std::collections::HashMap::new();
    out.insert(Path::new("grouping.yaml"), "---
  name: file-grouping
  transactions:
    - name: file-transaction
      date: 2023-01-30
      transfers:
        debts: -300
        money: 300
  "
    );
    Self{
      fake_fs: out,
    }
  }
}
#[cfg(test)]
impl FileIO for FakeFileIO {
  fn read_path(&mut self, path: &Path) -> String {
    self.fake_fs.get(path)
      .expect(&format!("File not found: {}", path.display()))
      .to_string()
  }
}
#[cfg(test)]
pub struct DummyFileIO {}
#[cfg(test)]
impl FileIO for DummyFileIO {
  fn read_path(&mut self, _path: &Path) -> String {
    unimplemented!()
  }
}
