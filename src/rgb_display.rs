use embedded_hal::digital::OutputPin;
use microbit::hal::{gpio, pac, timer::Timer};

use crate::util;

// The RGB PWM brightness scale is 100 steps, with each step taking 100 µs.
// 10 ms per frame; 100 frames per second.

const FRAME_US: u32 = 10_000;
const STEP_US: u32 = FRAME_US / 100;

/// The pulse width of a single RGB color channel measured in duty steps:
/// the discrete form of the duty cycle percentage.
#[derive(Default)]
struct RgbPulse {
    // Color channel
    // INFO: this option might not be necessary
    // RgbDisplay::step doesn't actually try to use the last index
    // I'll have to walk through it later to see if it's safe or not
    channel: Option<usize>,
    // Discrete duty steps in [1, 100]
    duty_steps: u8,
}

/// Iterator over the pulse widths for the current RGB frame with an empty
/// buffer at the end where all pulses are ended.
#[derive(Default)]
struct RgbPulseFrame {
    items: [RgbPulse; 4],
    index: usize,
}

impl RgbPulseFrame {
    fn new(c: hsv::Rgb) -> Self {
        #[rustfmt::skip]
        let mut items = [
            RgbPulse { channel: Some(0), duty_steps: (c.r * 100.0) as u8 },
            RgbPulse { channel: Some(1), duty_steps: (c.g * 100.0) as u8 },
            RgbPulse { channel: Some(2), duty_steps: (c.b * 100.0) as u8 },
            RgbPulse { channel: None, duty_steps: 100 },
        ];

        util::sort3_by_key(&mut items[..3], |c| c.duty_steps);

        Self { items, index: 0 }
    }

    /// Next [`RgbPulse`].
    fn next_mut(&mut self) -> Option<&mut RgbPulse> {
        match self.index {
            0..4 => {
                let item = Some(&mut self.items[self.index]);
                self.index += 1;
                item
            }
            _ => None,
        }
    }
}

/// RGB Display and scheduler.
pub struct RgbDisplay {
    // RGB pins.
    rgb_pins: [gpio::Pin<gpio::Output<gpio::PushPull>>; 3],
    // Current RGB channel.
    current_pin: Option<usize>,
    // What ticks should RGB LEDs turn off at?
    schedule: RgbPulseFrame,
    // Schedule to start at next frame.
    next_schedule: Option<RgbPulseFrame>,
    // Timer used to reach next tick.
    timer3: Timer<pac::TIMER3>,
}

impl RgbDisplay {
    pub fn new(
        rgb_pins: [gpio::Pin<gpio::Output<gpio::PushPull>>; 3],
        timer3: Timer<pac::TIMER3>,
    ) -> Self {
        Self {
            rgb_pins,
            current_pin: None,
            schedule: RgbPulseFrame::default(),
            next_schedule: None,
            timer3,
        }
    }

    /// Set up a new schedule, to be started next frame.
    pub fn set(&mut self, c: hsv::Rgb) {
        self.next_schedule = Some(RgbPulseFrame::new(c));
    }

    /// Returns true if the next schedule is set.
    pub fn is_scheduled(&self) -> bool {
        self.next_schedule.is_some()
    }

    /// Take the next frame update step. Called at startup
    /// and then from the timer interrupt handler.
    pub fn step(&mut self) {
        if let Some(RgbPulse {
            channel,
            duty_steps,
        }) = self.schedule.next_mut()
        {
            if let Some(pin_index) = self.current_pin {
                self.rgb_pins[pin_index].set_low();
            }
            // TODO: double check if the timer expects micro seconds
            self.timer3.start((*duty_steps as u32) * STEP_US);
            self.current_pin = channel.take();
        } else if let Some(schedule) = self.next_schedule.take() {
            for pin in &mut self.rgb_pins {
                pin.set_high();
            }
            self.schedule = schedule;
            self.step();
        } else {
            // no schedule, delay a little
            self.timer3.start(10);
        }
    }
}
