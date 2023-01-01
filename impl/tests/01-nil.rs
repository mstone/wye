// Check that the simplest use of the #[wye] attribute proc-macro compiles.
use wye::*;

#[wye]
fn add(a: u64, b: u64) -> u64 { a + b }

#[wye(a: format!("{a:?}"), b: format!("b"))]
fn add1(a: u64, b: u64) -> u64 { a + b }

pub fn main() {
    wyre!{};
    wyre!{()};
    wyre!{(){}};
    wyre!((a: format!("{a:?}")) let a = 1; );
}