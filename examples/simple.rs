extern crate ds;

use ds::*;

use std::thread;
use std::time::Duration;

fn main() {
    let mut ds = DriverStation::new_team(4069, Alliance::new_red(1));

    thread::sleep(Duration::from_millis(1500));
    ds.restart_code();
    loop {
        println!("Code: {}", ds.trace().is_code_started());

        thread::sleep(Duration::from_millis(20));
    }
}
