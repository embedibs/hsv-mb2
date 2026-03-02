//! hsv-mb2
//! ethan dibble <edibble@pdx.edu>

// RGB light cycle
// total 10ms = 10_000us
// turn all rgb on
// multiply intensities by 10_000
// set timer in steps to turn off channels
// example: (0.8, 0.2, 0.5)
// interrupt at 2_000, 5_000, 8_000 seconds from cycle start
// steps of 2_000, 3_000, 3_000

#![no_main]
#![no_std]

use cortex_m_rt::entry;
use critical_section_lock_mut::LockMut;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use microbit::{
    Board,
    display::blocking::Display,
    hal::{
        gpio::Level,
        gpiote::{self, Gpiote},
        pac::{self, interrupt},
        saadc::{Saadc, SaadcConfig},
        timer::Timer,
    },
};

use hsv_mb2::{color::*, util::*};

static HSV: LockMut<HsvColor> = LockMut::new();
static GPIOTE_PERIPHERAL: LockMut<Gpiote> = LockMut::new();
static TIMER_DEBOUNCE_A: LockMut<Timer<pac::TIMER0>> = LockMut::new();
static TIMER_DEBOUNCE_B: LockMut<Timer<pac::TIMER1>> = LockMut::new();

const MAX_POT: i16 = 0x3FFF;

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

#[interrupt]
fn TIMER2() {}

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = Board::take().unwrap();

    let mut display = Display::new(board.display_pins);

    let mut timer_debounce_a = Timer::new(board.TIMER0);
    let mut timer_debounce_b = Timer::new(board.TIMER1);
    let mut timer_pwm = Timer::new(board.TIMER2);
    let mut timer_display = Timer::new(board.TIMER3);

    let mut saadc = Saadc::new(board.ADC, SaadcConfig::default());
    let mut pin_pot = board.edge.e02.into_floating_input();

    let mut _pin_r = board.edge.e08.into_push_pull_output(Level::Low).degrade();
    let mut _pin_g = board.edge.e09.into_push_pull_output(Level::Low).degrade();
    let mut _pin_b = board.edge.e16.into_push_pull_output(Level::Low).degrade();

    let button_a = board.buttons.button_a;
    let button_b = board.buttons.button_b;

    let gpiote = gpiote::Gpiote::new(board.GPIOTE);

    let _ = gpiote
        .channel0()
        .input_pin(&button_a.degrade())
        .hi_to_lo()
        .enable_interrupt();

    let _ = gpiote
        .channel1()
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

    init_nvic();

    HSV.init(HsvColor::default());

    loop {
        //asm::wfi();
        HSV.with_lock(|hsv| {
            if let Ok(v) = saadc.read_channel(&mut pin_pot) {
                let v = v.clamp(0, MAX_POT) as f32 / MAX_POT as f32;
                hsv.set_current(v);
                let hsv = hsv.hsv;
                let rgb = hsv.to_rgb();
                rprintln!("hsv: {} {} {}", hsv.h, hsv.s, hsv.v);
                rprintln!("rgb: {} {} {}", rgb.r, rgb.g, rgb.b);
            }
            display.show(&mut timer_display, hsv.to_display(), 100);
        });
    }
}

/// Set up the NVIC to handle interrupts.
fn init_nvic() {
    unsafe {
        pac::NVIC::unmask(pac::Interrupt::GPIOTE);
        pac::NVIC::unmask(pac::Interrupt::TIMER2);
    };
    pac::NVIC::unpend(pac::Interrupt::GPIOTE);
    pac::NVIC::unpend(pac::Interrupt::TIMER2);
}
