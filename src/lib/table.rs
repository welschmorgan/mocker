use std::io::Write;

/// Represent a terminal table, drawn aligned.
#[derive(Debug, Clone)]
pub struct Table<const N: usize> {
  header: Option<[String; N]>,
  line_prefix: Option<String>,
  separator: Option<String>,
  rows: Vec<[String; N]>,
  widths: [usize; N],
  dirty: bool,
}

impl<const N: usize> Table<N> {
  const C_STR: String = String::new();

  pub fn new() -> Self {
    Self {
      header: None,
      line_prefix: None,
      separator: Some(String::from(" ")),
      rows: Default::default(),
      dirty: false,
      widths: [Default::default(); N],
    }
  }

  pub fn from_iter<I: IntoIterator<Item = [String; N]>>(iter: I) -> Self {
    let mut ret = Self::new();
    for row in iter.into_iter() {
      ret.push(row);
    }
    ret
  }

  pub fn with_header(mut self, v: [String; N]) -> Self {
    self.header = Some(v);
    self
  }

  pub fn with_line_prefix<S: AsRef<str>>(mut self, v: S) -> Self {
    self.line_prefix = Some(v.as_ref().to_string());
    self
  }

  pub fn with_row<V: AsRef<str>>(mut self, v: [V; N]) -> Self {
    self.push(v);
    self
  }

  pub fn with_separator<V: AsRef<str>>(mut self, v: V) -> Self {
    self.separator = Some(v.as_ref().to_string());
    self
  }

  pub fn with_rows<V: AsRef<str>, I: IntoIterator<Item = [V; N]>>(mut self, v: I) -> Self {
    let mut row = [Self::C_STR; N];
    for in_row in v.into_iter() {
      for (i, cell) in in_row.iter().enumerate() {
        row[i] = cell.as_ref().to_string();
      }
      self.push(row);
      row = [Self::C_STR; N];
    }
    self
  }

  pub fn with_widths(mut self, v: [usize; N]) -> Self {
    self.widths = v;
    self
  }

  pub fn rows(&self) -> &Vec<[String; N]> {
    &self.rows
  }

  pub fn width(&self, n: usize) -> Option<&usize> {
    self.widths.get(n)
  }

  pub fn is_dirty(&self) -> bool {
    self.dirty
  }

  pub fn widths(&self) -> &[usize; N] {
    &self.widths
  }

  pub fn clear(&mut self) {
    self.dirty = false;
    self.rows.clear();
    self.widths = [Default::default(); N];
  }

  pub fn push<T: AsRef<str>>(&mut self, row: [T; N]) {
    self.dirty = true;
    let mut strs = [Self::C_STR; N];
    for (i, cell) in row.iter().enumerate() {
      let v = cell.as_ref().to_string();
      self.widths[i] = self.widths[i].max(v.len());
      strs[i] = v;
    }
    self.rows.push(strs);
  }

  pub fn aligned(&self) -> Self {
    let mut ret = self.clone();
    if !self.dirty {
      return ret;
    }
    if let Some(header) = &ret.header {
      for (i, cell) in header.iter().enumerate() {
        ret.widths[i] = ret.widths[i].max(cell.len());
      }
    }
    for row in &mut ret.rows {
      let mut aligned_row = [Self::C_STR; N];
      for (i, cell) in row.iter().enumerate() {
        aligned_row[i] = format!("{:width$}", cell, width = self.widths[i]);
      }
      *row = aligned_row;
    }
    ret.dirty = false;
    ret
  }

  pub fn write<W: Write>(&self, mut w: W) -> crate::Result<()> {
    let aligned = self.aligned();
    let mut first_row = true;
    for row in &aligned.rows {
      if !first_row {
        writeln!(w)?;
      }
      if let Some(prefix) = self.line_prefix.as_ref() {
        write!(w, "{}", prefix)?;
      }
      let mut first_cell = true;
      for (i, cell) in row.iter().enumerate() {
        if let Some(sep) = self.separator.as_ref() {
          if !first_cell {
            write!(w, "{}", sep)?;
          }
        }
        write!(w, "{:width$}", cell, width = self.widths[i])?;
        first_cell = false;
      }
      first_row = false;
    }
    w.flush()?;
    Ok(())
  }
}
