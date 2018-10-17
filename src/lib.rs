// Want to use beta toolchain  #![feature(specialization)]

/* #[macro_use] */ extern crate log;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate failure_derive;

extern crate env_logger;
extern crate serde;
extern crate serde_json;
extern crate failure;

/*
Foreground:
39  default
30  black
31  red
32  green
33  yellow
34  blue
35  magenta
36  cyan
37  light gray
90  dark gray
91  light red
92  light green
93  light yellow
94  light blue
95  light magenta
96  light cyan
97  white

Background is the same, just starting from 40 and 100 respectively.

Style:
0  Reset
1  Bold/Bright
2  Dim
4  Underline
5  Blink
7  Invert
8  Hidden
21 Reset Bold, similar for the others...
*/

pub static LOGGER_INIT: std::sync::atomic::AtomicUsize = std::sync::atomic::ATOMIC_USIZE_INIT;

lazy_static! {
  pub static ref LOGGER_MX: std::sync::Mutex<u32> = Default::default();
  pub static ref LOG_LEVEL: std::sync::atomic::AtomicIsize = std::sync::atomic::AtomicIsize::new(9);
}

#[cfg(not(feature = "theme_dark"))]
pub static G_COLORS: [&'static str; 9] = ["31", "32", "33", "34", "35", "36", "31;1;107", "32;1;107", "30;1;107"];

#[cfg(feature = "theme_dark")]
pub static G_COLORS: [&'static str; 9] = ["31", "32", "33", "34", "35", "36", "31;1;100", "32;1;100", "33;1;100"];

#[allow(unused_macros)]
#[macro_export]
macro_rules! logg {
  ($level:expr, $color:expr, $me:expr, $fmt:expr, $($x:expr),*) => (
    let log_level = ::LOG_LEVEL.load(::std::sync::atomic::Ordering::SeqCst);
    if log_level >= 0 && $level <= log_level {
      jfail_fn::logger_init();
      let mut f = file!();
      if (&f).ends_with(".rs") {
        f = f.split_at(f.len()-3).0;
      }
      let f = f.split_at(f.len().max(8)-8).1;
      let t = jfail_fn::thrid();
      let t = t.split_at(t.len().max(8)-8).1;
      //let tid = ::std::thread::current().id();
      println!(concat!("[{}] {:8.8}:{:4}  ", "{:8.8} ", "\u{1b}[0;{}m", "{}", "\u{1b}[0m ", $fmt),
        $level, f, line!(), t, ::G_COLORS[$color], $me, $($x),*)
    }
  );
  ($level:expr, $color:expr, $me:expr, $fmt:expr) => (
    logg!($level, $color, $me, concat!("{}", $fmt), "")
  );
}

#[allow(unused_macros)]
#[macro_export]
macro_rules! jfail {
  ($fmt:expr, $($x:expr),*) => (
    //let t = ::thrid();
    //let t = t.split_at(t.len().max(6)-6).1;
    //let tid = ::std::thread::current().id();
    JFail {
      file: Some(file!().into()),
      line: Some(line!()),
      thread: jfail_fn::thrid(),
      why: format!(concat!($fmt), $($x),*),
      nested: JFail::make_nested(None, None),
      backtrace: JFail::create_backtrace(),
    }
  );
  ($fmt:expr) => (
    JFail {
      file: Some(file!().to_string()),
      line: Some(line!()),
      thread: jfail_fn::thrid(),
      why: $fmt.to_string(),
      nested: JFail::make_nested(None, None),
      backtrace: JFail::create_backtrace(),
    }
  );
}

#[allow(unused_macros)]
#[macro_export]
macro_rules! jfailn {
  ($nested:expr, $fmt:expr, $($x:expr),*) => (
    //let t = ::thrid();
    //let t = t.split_at(t.len().max(6)-6).1;
    //let tid = ::std::thread::current().id();
    JFail {
      file: Some(file!().to_string()),
      line: Some(line!()),
      thread: jfail_fn::thrid(),
      why: format!(concat!($fmt), $($x),*),
      nested: JFail::make_nested(Some(Box::new($nested)), None),
      backtrace: None,
    }
  );
  ($nested:expr, $fmt:expr) => (
    JFail {
      file: Some(file!().to_string()),
      line: Some(line!()),
      thread: jfail_fn::thrid(),
      why: $fmt.to_string(),
      nested: JFail::make_nested(Some(Box::new($nested)), None),
      backtrace: None,
    }
  );
}

#[allow(unused_macros)]
#[macro_export]
macro_rules! jfail_combine {
  ($e1:expr, $e2:expr, $fmt:expr) => (
    JFail {
      file: Some(file!().to_string()),
      line: Some(line!()),
      thread: jfail_fn::thrid(),
      why: $fmt.to_string(),
      nested: JFail::make_nested(Some(Box::new($e1)), Some(Box::new($e2))),
      backtrace: None,
    }
  );
}

#[macro_export]
macro_rules! jfaildb {
  ($e:expr, $fmt:expr, $($x:expr),*) => (jfail!(concat!($fmt, "  Error: {:?}"), $($x),*, $e));
  ($e:expr, $fmt:expr) => (jfail!(concat!($fmt, "  Error: {:?}"), $e));
  ($e:expr) => (jfail!("{:?}", $e));
}

fn _________logger_init() {
  use std::sync::atomic::Ordering::SeqCst;
  if LOGGER_INIT.load(SeqCst) == 0 {
    match LOGGER_MX.lock() {
      Ok(_mutex) => {
        if LOGGER_INIT.load(SeqCst) == 0 {
          env_logger::init();
          LOGGER_INIT.store(1, SeqCst);
          for x in std::env::args() {
            if x == "--nocapture" {
              LOG_LEVEL.store(9, SeqCst);
            }
          }
        }
      }
      _ => {
        panic!("mutex lock error");
      }
    }
  }
}

// Put functions into this module so that macros can reference them from whichever scope
//pub use jfail_fn_2 as jfail_fn;
pub mod jfail_fn {
use std;
pub fn thrid() -> String {
  std::thread::current().name().unwrap().into()
}
}

fn ____jsons<T: serde::Serialize>(x: &T) -> String {
  serde_json::to_string_pretty(&x)
  .unwrap_or_else(|_e| { r#"{"error": "CAN NOT FORMAT"}"#.into() })
}

#[derive(Debug, Fail, Serialize)]
pub struct Nested {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub e0: Option<Box<JFail>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub e1: Option<Box<JFail>>,
}

impl Nested {
  fn is_empty(&self) -> bool {
    self.e0.is_none() && self.e1.is_none()
  }
}

impl std::fmt::Display for Nested {
  fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(fmt, "({:?}, {:?})", self.e0, self.e1)
  }
}

#[derive(Debug, Fail, Serialize)]
//#[fail(display = "{}", why)]
pub struct JFail {
  pub file: Option<String>,
  pub line: Option<u32>,
  pub thread: String,
  pub why: String,
  #[serde(skip_serializing_if = "Nested::is_empty")]
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

impl JFail {
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
  pub fn make_nested(e0: Option<Box<JFail>>, e1: Option<Box<JFail>>) -> Nested {
    Nested { e0, e1 }
  }
}

impl std::fmt::Display for JFail {
  fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(fmt, "{}", self.to_json_string())
  }
}

impl From<JFail> for String {
  fn from(x: JFail) -> String {
    x.to_json_string()
  }
}

impl From<()> for JFail {
  fn from(_: ()) -> Self {
    jfail!("Error From<()>, no info.")
  }
}

impl From<std::io::Error> for JFail {
  fn from(x: std::io::Error) -> Self {
    jfail!("io::Error: {}", x)
  }
}

/*
Specialization only on nightly toolchain.
impl<T: std::fmt::Debug> From<T> for JFail {
  default fn from(x: T) -> Self {
    dwfail!("{:?}", x)
  }
}
*/

#[cfg(test)]
mod test;
