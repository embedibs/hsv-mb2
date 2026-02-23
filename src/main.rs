//! hsv-mb2
//! ethan dibble <edibble@pdx.edu>

#![no_main]
#![no_std]

use panic_rtt_target as _;
use rtt_target::rtt_init_print;

use cortex_m_rt::entry;
use embedded_hal::{delay::DelayNs, digital::OutputPin};
use microbit::{
    board::Board,
    hal::{Timer, gpio::Level},
};

use critical_section_lock_mut::LockMut;

mod color;
use color::*;

static HSV: LockMut<HsvColor> = LockMut::new();

#[entry]
fn main() -> ! {
    rtt_init_print!();

    HSV.init(HsvColor::new());

    let board = Board::take().unwrap();
    let mut timer = Timer::new(board.TIMER0);
    let pin_r = board.edge.e08.into_push_pull_output(Level::High);
    let pin_g = board.edge.e09.into_push_pull_output(Level::High);
    let pin_b = board.edge.e16.into_push_pull_output(Level::High);
    let mut pins = [pin_r.degrade(), pin_g.degrade(), pin_b.degrade()];
    let mut index = 0;
    let states = [0b100, 0b010, 0b001, 0b011, 0b101, 0b110, 0b111, 0];
    let nstates = states.len();

    loop {
        let state = states[index];
        for (i, p) in pins.iter_mut().enumerate() {
            if (state >> (2 - i)) & 1 == 1 {
                p.set_low().unwrap();
            } else {
                p.set_high().unwrap();
            }
        }
        timer.delay_ms(1000);
        index = (index + 1) % nstates;
    }
}
