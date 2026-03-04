//! hsv-mb2
//! ethan dibble <edibble@pdx.edu>

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

use hsv_mb2::common::*;

static HSV_COLOR: LockMut<HsvColor> = LockMut::new();
static RGB_DISPLAY: LockMut<RgbDisplay> = LockMut::new();
static MB2_DISPLAY: LockMut<Display<pac::TIMER2>> = LockMut::new();

static BUTTON_A: LockMut<Button<pac::TIMER0, fn()>> = LockMut::new();
static BUTTON_B: LockMut<Button<pac::TIMER1, fn()>> = LockMut::new();

static GPIOTE_PERIPHERAL: LockMut<Gpiote> = LockMut::new();

const MAX_POT: i16 = 0x3FFF;

#[interrupt]
fn GPIOTE() {
    GPIOTE_PERIPHERAL.with_lock(|gpiote| {
        if gpiote.channel0().is_event_triggered() {
            BUTTON_A.with_lock(|btn| btn.handle_event());
        }
        if gpiote.channel1().is_event_triggered() {
            BUTTON_B.with_lock(|btn| btn.handle_event());
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

    // Potentiometer
    let mut saadc = Saadc::new(board.ADC, SaadcConfig::default());
    let mut pin_pot = board.edge.e02;

    // RGB pins
    let pin_r = board.edge.e08.into_push_pull_output(Level::Low).degrade();
    let pin_g = board.edge.e09.into_push_pull_output(Level::Low).degrade();
    let pin_b = board.edge.e16.into_push_pull_output(Level::Low).degrade();

    HSV_COLOR.init(HsvColor::default());
    HSV_COLOR.with_lock(|hsv| mb2_display.show(&hsv.to_display()));

    RGB_DISPLAY.init(RgbDisplay::new([pin_r, pin_g, pin_b], timer_pwm, 0.2));
    RGB_DISPLAY.with_lock(|display| display.step());

    MB2_DISPLAY.init(mb2_display);

    init_buttons(
        board.TIMER0,
        board.TIMER1,
        board.GPIOTE,
        board.buttons.button_a.degrade(),
        board.buttons.button_b.degrade(),
    );

    init_nvic(board.NVIC);

    loop {
        if let Ok(v) = saadc.read_channel(&mut pin_pot) {
            let v = v.clamp(0, MAX_POT) as f32 / MAX_POT as f32;

            HSV_COLOR.with_lock(|hsv_color| {
                hsv_color.set_current(v);

                #[cfg(feature = "log")]
                {
                    let rgb = hsv_color.to_rgb();
                    rprintln!("rgb: {:.2} {:.2} {:.2}", rgb.r, rgb.g, rgb.b);
                }

                RGB_DISPLAY.with_lock(|display| {
                    if !display.is_scheduled() {
                        #[cfg(feature = "log")]
                        rprintln!("setting next schedule");

                        display.set_schedule(hsv_color.to_rgb());
                    }
                });
            });
        }
    }
}

/// Set up the NVIC to handle interrupts.
fn init_nvic(mut nvic: pac::NVIC) {
    unsafe {
        // buttons (low priority).
        pac::NVIC::unmask(pac::Interrupt::GPIOTE);
        nvic.set_priority(pac::Interrupt::GPIOTE, 32);

        // mb2 display (low priority).
        pac::NVIC::unmask(pac::Interrupt::TIMER2);
        nvic.set_priority(pac::Interrupt::TIMER2, 32);

        // led display (high priority).
        pac::NVIC::unmask(pac::Interrupt::TIMER3);
        nvic.set_priority(pac::Interrupt::TIMER3, 16);
    };
    pac::NVIC::unpend(pac::Interrupt::GPIOTE);
    pac::NVIC::unpend(pac::Interrupt::TIMER2);
    pac::NVIC::unpend(pac::Interrupt::TIMER3);
}

/// Set up microbit buttons.
fn init_buttons(
    timer0: pac::TIMER0,
    timer1: pac::TIMER1,
    gpiote: pac::GPIOTE,
    button_a: impl gpiote::GpioteInputPin,
    button_b: impl gpiote::GpioteInputPin,
) {
    let mut timer_debounce_a = Timer::new(timer0);
    let mut timer_debounce_b = Timer::new(timer1);

    let gpiote = gpiote::Gpiote::new(gpiote);

    let _ = gpiote
        .channel0()
        .input_pin(&button_a)
        .hi_to_lo()
        .enable_interrupt();

    let _ = gpiote
        .channel1()
        .input_pin(&button_b)
        .hi_to_lo()
        .enable_interrupt();

    GPIOTE_PERIPHERAL.init(gpiote);

    timer_debounce_a.disable_interrupt();
    timer_debounce_a.reset_event();

    BUTTON_A.init(Button::new(timer_debounce_a, || {
        HSV_COLOR.with_lock(|hsv| {
            hsv.state = hsv.state.pred();
            MB2_DISPLAY.with_lock(|display| {
                display.show(&hsv.to_display());
            });
        });
    }));

    timer_debounce_b.disable_interrupt();
    timer_debounce_b.reset_event();

    BUTTON_B.init(Button::new(timer_debounce_b, || {
        HSV_COLOR.with_lock(|hsv| {
            hsv.state = hsv.state.succ();
            MB2_DISPLAY.with_lock(|display| {
                display.show(&hsv.to_display());
            });
        });
    }));
}
