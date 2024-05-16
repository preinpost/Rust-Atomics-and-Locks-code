use crate::ch2::atomic::{example3, example4, fetch_add_example};
use crate::ch5::channel::safe_channel::run;
use crate::ch5::channel::safe_channel_without_arc::run3;

mod ch1;
mod ch2;
mod ch5;

fn main() {
    run3();
}