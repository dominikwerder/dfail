//#![feature(specialization)]

use failure_derive::Fail;

#[allow(unused_macros)]
#[macro_export]
macro_rules! fail {
  ($fmt:expr, $($x:expr),*) => (
    $crate::Fail {
      file: Some(file!().into()),
      line: Some(line!()),
      thread: $crate::__thrid(),
      why: format!(concat!($fmt), $($x),*),
      nested: $crate::Fail::make_nested(None, None),
      backtrace: $crate::Fail::create_backtrace(),
    }
  );
  ($fmt:expr) => (
    $crate::Fail {
      file: Some(file!().to_string()),
      line: Some(line!()),
      thread: $crate::__thrid(),
      why: $fmt.to_string(),
      nested: $crate::Fail::make_nested(None, None),
      backtrace: $crate::Fail::create_backtrace(),
    }
  );
}

#[allow(unused_macros)]
#[macro_export]
macro_rules! failn {
  ($nested:expr, $fmt:expr, $($x:expr),*) => (
    $crate::Fail {
      file: Some(file!().to_string()),
      line: Some(line!()),
      thread: $crate::__thrid(),
      why: format!(concat!($fmt), $($x),*),
      nested: $crate::Fail::make_nested(Some(Box::new($nested)), None),
      backtrace: None,
    }
  );
  ($nested:expr, $fmt:expr) => (
    $crate::Fail {
      file: Some(file!().to_string()),
      line: Some(line!()),
      thread: $crate::__thrid(),
      why: $fmt.to_string(),
      nested: $crate::Fail::make_nested(Some(Box::new($nested)), None),
      backtrace: None,
    }
  );
}

#[allow(unused_macros)]
#[macro_export]
macro_rules! fail_combine {
  ($e1:expr, $e2:expr, $fmt:expr) => (
    Fail {
      file: Some(file!().to_string()),
      line: Some(line!()),
      thread: $crate::__thrid(),
      why: $fmt.to_string(),
      nested: $crate::Fail::make_nested(Some(Box::new($e1)), Some(Box::new($e2))),
      backtrace: None,
    }
  );
}

#[macro_export]
macro_rules! faildb {
  ($e:expr, $fmt:expr, $($x:expr),*) => ($crate::fail!(concat!($fmt, "  Error: {:?}"), $($x),*, $e));
  ($e:expr, $fmt:expr) => ($crate::fail!(concat!($fmt, "  Error: {:?}"), $e));
  ($e:expr) => ($crate::fail!("{:?}", $e));
}

pub fn __thrid() -> String {
  std::thread::current().name().unwrap().into()
}

/*
fn __jsons<T: serde::Serialize>(x: &T) -> String {
  serde_json::to_string_pretty(&x)
  .unwrap_or_else(|_e| { r#"{"error": "CAN NOT FORMAT"}"#.into() })
}
*/

#[derive(Debug, Fail)]
#[derive(serde_derive::Serialize)]
pub struct Nested {
  //#[serde(skip_serializing_if = "Option::is_none")]
  pub e0: Option<Box<Fail>>,
  //#[serde(skip_serializing_if = "Option::is_none")]
  pub e1: Option<Box<Fail>>,
}

impl Nested {
  #[allow(unused)]
  fn is_empty(&self) -> bool {
    self.e0.is_none() && self.e1.is_none()
  }
}

impl std::fmt::Display for Nested {
  fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(fmt, "({:?}, {:?})", self.e0, self.e1)
  }
}

#[derive(Debug, Fail)]
#[derive(serde_derive::Serialize)]
//#[fail(display = "{}", why)]
pub struct Fail {
  pub file: Option<String>,
  pub line: Option<u32>,
  pub thread: String,
  pub why: String,
  //#[serde(skip_serializing_if = "Nested::is_empty")]
  pub nested: Nested,
  #[serde(skip_deserializing, skip_serializing_if = "Option::is_none", serialize_with="serialize_backtrace")]
  pub backtrace: Option<failure::Backtrace>,
}

fn serialize_backtrace<S>(t: &Option<failure::Backtrace>, s: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
  match t {
    None => s.serialize_none(),
    Some(x) => s.collect_seq(format!("{}", x).split("\n"))
  }
}

impl Fail {
  pub fn into_fail_box(self) -> Box<failure::Fail> {
    Box::new(self) as Box<failure::Fail>
  }
  pub fn why(&self) -> &String {
    &self.why
  }
  pub fn to_json_string(&self) -> String {
    serde_json::to_string_pretty(self)
    .unwrap_or_else(|e| {
      format!("can't format the error: {}", e)
    })
  }
  pub fn create_backtrace() -> Option<failure::Backtrace> {
    Some(failure::Backtrace::new())
  }
  pub fn make_nested(e0: Option<Box<Fail>>, e1: Option<Box<Fail>>) -> Nested {
    Nested { e0, e1 }
  }
}

impl std::fmt::Display for Fail {
  fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(fmt, "{:?}", self)
  }
}

impl From<Fail> for String {
  fn from(x: Fail) -> String {
    format!("{:?}", x)
  }
}

impl From<()> for Fail {
  fn from(_: ()) -> Self {
    fail!("Error From<()>, no info.")
  }
}

impl From<std::io::Error> for Fail {
  fn from(x: std::io::Error) -> Self {
    fail!("io::Error: {}", x)
  }
}

/*
Specialization only on nightly toolchain.
impl<T: std::fmt::Debug> From<T> for Fail {
  default fn from(x: T) -> Self {
    fail!("{:?}", x)
  }
}
*/
