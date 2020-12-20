/// Basado en https://os.phil-opp.com/testing/
use crate::{print, println};

pub trait Testable {
    fn run(&self);
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        print!("test {} ... \t", core::any::type_name::<T>());
        self();
        println!("\x1b[0;32mok\x1b[0m");
    }
}

#[cfg(test)]
pub fn test_runner(tests: &[&dyn Testable]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
}

#[test_case]
fn trivial() {
    assert_eq!(2 + 2, 4);
}
