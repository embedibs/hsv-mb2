use embedded_hal::digital::OutputPin;
use microbit::hal::{gpio, pac, timer::Timer};

// The RGB PWM brightness scale is 100 steps, with each step taking 100 µs.
// 10 ms per frame; 100 frames per second.

const STEP_US: u32 = 100;

/// RGB Display and scheduler.
pub struct RgbDisplay {
    // RGB pins.
    rgb_pins: [gpio::Pin<gpio::Output<gpio::PushPull>>; 3],
    // Current RGB channel.
    tick: u32,
    // What duty steps should RGB LEDs turn off at?
    schedule: [u32; 3],
    // Schedule to start at next frame.
    next_schedule: Option<[u32; 3]>,
    // Timer used to reach next tick.
    timer3: Timer<pac::TIMER3>,
}

impl RgbDisplay {
    pub fn new(
        rgb_pins: [gpio::Pin<gpio::Output<gpio::PushPull>>; 3],
        mut timer3: Timer<pac::TIMER3>,
    ) -> Self {
        timer3.enable_interrupt();

        Self {
            rgb_pins,
            tick: 0,
            schedule: [100, 100, 100],
            next_schedule: None,
            timer3,
        }
    }

    /// Set up a new schedule, to be started next frame.
    pub fn set_schedule(&mut self, c: hsv::Rgb) {
        self.next_schedule = Some([
            (c.r * 100.0) as u32,
            (c.g * 100.0) as u32,
            (c.b * 100.0) as u32,
        ]);
    }

    /// Returns true if the next schedule is set.
    pub fn is_scheduled(&self) -> bool {
        self.next_schedule.is_some()
    }

    /// Reset self
    pub fn reset(&mut self) {
        for pin in &mut self.rgb_pins {
            pin.set_high().unwrap();
        }

        self.tick = 0;
    }

    /// Take the next frame update step. Called at startup
    /// and then from the timer interrupt handler.
    pub fn step(&mut self) {
        self.timer3.reset_event();

        if let Some(&next_tick) = self //
            .schedule
            .iter()
            .filter(|duty| **duty > self.tick)
            .min()
        {
            for (i, &tick) in self.schedule.iter().enumerate() {
                if tick == next_tick {
                    self.rgb_pins[i].set_low();
                }
            }

            let delay = (next_tick - self.tick) * STEP_US;

            self.tick = next_tick;
            self.timer3.delay(delay.max(STEP_US));
        } else if let Some(schedule) = self.next_schedule.take() {
            self.schedule = schedule;
            self.reset();
            self.timer3.start(STEP_US);
        } else {
            // no schedule, delay a little
            self.timer3.start(STEP_US);
        }
    }
}
