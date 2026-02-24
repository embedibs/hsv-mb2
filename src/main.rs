//! hsv-mb2
//! ethan dibble <edibble@pdx.edu>

#![no_main]
#![no_std]

use cortex_m::asm;
use cortex_m_rt::entry;
use critical_section_lock_mut::LockMut;
use embedded_hal::{delay::DelayNs, digital::OutputPin};
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use microbit::{
    Board,
    display::blocking::Display,
    hal::{
        self, gpiote,
        pac::{self, interrupt},
        timer::{Instance, Timer},
    },
};

mod color;
use color::*;

static HSV: LockMut<HsvColor> = LockMut::new();
static GPIOTE_PERIPHERAL: LockMut<gpiote::Gpiote> = LockMut::new();
static TIMER_DEBOUNCE_A: LockMut<hal::Timer<pac::TIMER1>> = LockMut::new();
static TIMER_DEBOUNCE_B: LockMut<hal::Timer<pac::TIMER2>> = LockMut::new();
static TIMER_DEBOUNCE_POT: LockMut<hal::Timer<pac::TIMER3>> = LockMut::new();

// 100ms at 1MHz count rate.
const DEBOUNCE_TIME: u32 = 100 * 1_000_000 / 1000;

#[interrupt]
fn GPIOTE() {
    GPIOTE_PERIPHERAL.with_lock(|gpiote| {
        if gpiote.channel0().is_event_triggered() {
            debounce(&TIMER_DEBOUNCE_A, || {
                HSV.with_lock(|hsv| hsv.state = hsv.state.prev())
            });
        }
        if gpiote.channel1().is_event_triggered() {
            debounce(&TIMER_DEBOUNCE_B, || {
                HSV.with_lock(|hsv| hsv.state = hsv.state.next())
            });
        }
        gpiote.channel0().reset_events();
        gpiote.channel1().reset_events();
    });
}

fn debounce<T, F>(timer: &LockMut<hal::Timer<T>>, f: F)
where
    T: Instance,
    F: FnOnce() -> (),
{
    timer.with_lock(|timer| {
        if timer.read() == 0 {
            f();
            timer.start(DEBOUNCE_TIME);
        }
    });
}

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = Board::take().unwrap();

    let mut timer_display = Timer::new(board.TIMER0);
    let mut timer_debounce_a = Timer::new(board.TIMER1);
    let mut timer_debounce_b = Timer::new(board.TIMER2);
    let mut timer_debounce_pot = Timer::new(board.TIMER3);
    let mut display = Display::new(board.display_pins);
    let button_a = board.buttons.button_a;
    let button_b = board.buttons.button_b;

    let gpiote = gpiote::Gpiote::new(board.GPIOTE);

    let channel0 = gpiote.channel0();
    channel0
        .input_pin(&button_a.degrade())
        .hi_to_lo()
        .enable_interrupt();

    let channel1 = gpiote.channel1();
    channel1
        .input_pin(&button_b.degrade())
        .hi_to_lo()
        .enable_interrupt();

    GPIOTE_PERIPHERAL.init(gpiote);

    timer_debounce_a.disable_interrupt();
    timer_debounce_a.reset_event();
    TIMER_DEBOUNCE_A.init(timer_debounce_a);

    timer_debounce_b.disable_interrupt();
    timer_debounce_b.reset_event();
    TIMER_DEBOUNCE_B.init(timer_debounce_b);

    timer_debounce_pot.disable_interrupt();
    timer_debounce_pot.reset_event();
    TIMER_DEBOUNCE_POT.init(timer_debounce_pot);

    HSV.init(HsvColor::default());

    // Set up the NVIC to handle interrupts.
    unsafe { pac::NVIC::unmask(pac::Interrupt::GPIOTE) };
    pac::NVIC::unpend(pac::Interrupt::GPIOTE);

    loop {
        //asm::wfi();
        HSV.with_lock(|hsv| {
            display.show(&mut timer_display, hsv.to_display(), 100);
        });
    }
}
