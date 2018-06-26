use super::*;

#[test]
fn make_fail() {
  let f = jfail!("");
  println!("{}", f);
}
