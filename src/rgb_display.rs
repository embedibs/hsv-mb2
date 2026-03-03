use embedded_hal::{delay::DelayNs, digital::OutputPin};
use microbit::hal::{gpio, pac, timer::Timer};

// The RGB PWM brightness scale is 100 steps, with each step taking 100 µs.
// 10 ms per frame; 100 frames per second.

#[cfg(feature = "log")]
use rtt_target::rprintln;

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
    // Brightness [0,1] scales the duty cycle
    brightness: f32,
}

impl RgbDisplay {
    pub fn new(
        rgb_pins: [gpio::Pin<gpio::Output<gpio::PushPull>>; 3],
        mut timer3: Timer<pac::TIMER3>,
        brightness: f32,
    ) -> Self {
        timer3.enable_interrupt();

        Self {
            rgb_pins,
            tick: 0,
            schedule: [0, 0, 0],
            next_schedule: None,
            timer3,
            brightness: brightness.clamp(0.0, 1.0),
        }
    }

    /// Set up a new schedule, to be started next frame.
    pub fn set_schedule(&mut self, c: hsv::Rgb) {
        self.next_schedule = Some([
            (c.r * 100.0 * self.brightness) as u32,
            (c.g * 100.0 * self.brightness) as u32,
            (c.b * 100.0 * self.brightness) as u32,
        ]);
    }

    /// Returns true if the next schedule is set.
    pub fn is_scheduled(&self) -> bool {
        self.next_schedule.is_some()
    }

    /// Reset schedule
    fn reset(&mut self) {
        for (i, pin) in self.rgb_pins.iter_mut().enumerate() {
            if self.schedule[i] != 0 {
                pin.set_low().unwrap();
            }
        }

        self.tick = 0;
        // Let the interrupt handler call step
        self.timer3.start(STEP_US);
    }

    /// Take the next frame update step. Called at startup
    /// and then from the timer interrupt handler.
    pub fn step(&mut self) {
        // The timer event is already reset internally
        // self.timer3.reset_event();

        if let Some(&next_tick) = self
            .schedule
            .iter()
            .chain(&[100])
            .filter(|&&duty| duty > self.tick)
            .min()
        {
            // multiple channels could have the same duty cycle.
            for (i, pin) in self.rgb_pins.iter_mut().enumerate() {
                if self.schedule[i] == self.tick {
                    pin.set_high().unwrap();
                }
            }

            let delay = (next_tick - self.tick) * STEP_US;

            // delay_us delay
            // = { apply delay_us }
            // delay_ns (delay * 1_000)
            // = { apply delay_ns }
            // delay (delay * 1_000 / 1_000)
            // = { apply delay }
            // start delay;
            // spin loop until done

            self.tick = next_tick;
            self.timer3.start(delay.max(STEP_US) /* microseconds */);
        } else {
            if let Some(schedule) = self.next_schedule.take() {
                self.schedule = schedule;
            }
            self.reset();
        }
    }
}
