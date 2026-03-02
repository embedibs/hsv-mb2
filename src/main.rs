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

#[cfg(feature = "log")]
use rtt_target::{rprintln, rtt_init_print};

use microbit::{
    Board,
    display::nonblocking::Display,
    hal::{
        gpio::Level,
        gpiote::{self, Gpiote},
        pac::{self, interrupt},
        saadc::{Saadc, SaadcConfig},
        timer::Timer,
    },
};

use hsv_mb2::{color::*, rgb_display::*, util::*};

static HSV: LockMut<HsvColor> = LockMut::new();
static RGB_DISPLAY: LockMut<RgbDisplay> = LockMut::new();
static MB2_DISPLAY: LockMut<Display<pac::TIMER2>> = LockMut::new();

static GPIOTE_PERIPHERAL: LockMut<Gpiote> = LockMut::new();
static TIMER_DEBOUNCE_A: LockMut<Timer<pac::TIMER0>> = LockMut::new();
static TIMER_DEBOUNCE_B: LockMut<Timer<pac::TIMER1>> = LockMut::new();

const MAX_POT: i16 = 0x3FFF;

// WARN: this is big, but the blocking display was an issue
#[interrupt]
fn GPIOTE() {
    GPIOTE_PERIPHERAL.with_lock(|gpiote| {
        if gpiote.channel0().is_event_triggered() {
            debounce(&TIMER_DEBOUNCE_A, || {
                HSV.with_lock(|hsv| {
                    hsv.state = hsv.state.prev();
                    MB2_DISPLAY.with_lock(|display| {
                        display.show(&hsv.to_display());
                    });
                });
            });
        }
        if gpiote.channel1().is_event_triggered() {
            debounce(&TIMER_DEBOUNCE_B, || {
                HSV.with_lock(|hsv| {
                    hsv.state = hsv.state.next();
                    MB2_DISPLAY.with_lock(|display| {
                        display.show(&hsv.to_display());
                    });
                });
            });
        }
        gpiote.channel0().reset_events();
        gpiote.channel1().reset_events();
    });
}

#[interrupt]
fn TIMER2() {
    MB2_DISPLAY.with_lock(|display| display.handle_display_event());
}

#[interrupt]
fn TIMER3() {
    RGB_DISPLAY.with_lock(|display| display.step());
}

#[entry]
fn main() -> ! {
    #[cfg(feature = "log")]
    rtt_init_print!();

    let board = Board::take().unwrap();

    let mut mb2_display = Display::new(board.TIMER2, board.display_pins);
    let timer_pwm = Timer::new(board.TIMER3);

    // Button debounce timers
    let mut timer_debounce_a = Timer::new(board.TIMER0);
    let mut timer_debounce_b = Timer::new(board.TIMER1);

    // Potentiometer
    let mut saadc = Saadc::new(board.ADC, SaadcConfig::default());
    let mut pin_pot = board.edge.e02.into_floating_input();

    // RGB pins
    let pin_r = board.edge.e08.into_push_pull_output(Level::Low).degrade();
    let pin_g = board.edge.e09.into_push_pull_output(Level::Low).degrade();
    let pin_b = board.edge.e16.into_push_pull_output(Level::Low).degrade();

    // Button interrupts and events
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

    // HSV to RGB display
    HSV.init(HsvColor::default());
    HSV.with_lock(|hsv| mb2_display.show(&hsv.to_display()));

    RGB_DISPLAY.init(RgbDisplay::new([pin_r, pin_g, pin_b], timer_pwm));
    RGB_DISPLAY.with_lock(|display| display.step());

    // MB2 Display
    MB2_DISPLAY.init(mb2_display);

    // Initialize interrupts
    init_nvic();

    loop {
        HSV.with_lock(|hsv_color| {
            if let Ok(v) = saadc.read_channel(&mut pin_pot) {
                let v = v.clamp(0, MAX_POT) as f32 / MAX_POT as f32;
                hsv_color.set_current(v);

                #[cfg(feature = "log")]
                {
                    let hsv = hsv_color.hsv;
                    let rgb = hsv_color.to_rgb();
                    rprintln!("hsv: {} {} {}", hsv.h, hsv.s, hsv.v);
                    rprintln!("rgb: {} {} {}", rgb.r, rgb.g, rgb.b);
                }
            }

            RGB_DISPLAY.with_lock(|display| {
                if !display.is_scheduled() {
                    #[cfg(feature = "log")]
                    rprintln!("[INFO] setting next schedule");

                    display.set_schedule(hsv_color.to_rgb());
                }
            });
        });
    }
}

/// Set up the NVIC to handle interrupts.
fn init_nvic() {
    unsafe {
        pac::NVIC::unmask(pac::Interrupt::GPIOTE);
        pac::NVIC::unmask(pac::Interrupt::TIMER2);
        pac::NVIC::unmask(pac::Interrupt::TIMER3);
    };
    pac::NVIC::unpend(pac::Interrupt::GPIOTE);
    pac::NVIC::unpend(pac::Interrupt::TIMER2);
    pac::NVIC::unpend(pac::Interrupt::TIMER3);
}
