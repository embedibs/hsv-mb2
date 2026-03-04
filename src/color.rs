//! Color

use hsv::{Hsv, Rgb};
use microbit::display::nonblocking::BitImage;

pub struct HsvColor {
    /// HSV color.
    pub hsv: Hsv,
    /// What channel is being manipulated.
    pub state: State,
}

impl HsvColor {
    pub fn new() -> Self {
        Self {
            hsv: Hsv {
                h: 0.1,
                s: 0.1,
                v: 0.1,
            },
            state: State::default(),
        }
    }

    /// Convert underlying HSV color to RGB.
    pub fn to_rgb(&self) -> Rgb {
        Rgb::from(self.hsv)
    }

    /// Convert HSV state to an mb2 display.
    pub fn to_display(&self) -> BitImage {
        BitImage::from(self.state)
    }

    /// Set the current HSV color channel value.
    pub fn set_current(&mut self, v: f32) {
        self.with_current(|x| *x = v);
    }

    // If I later decide to do anything else wacky and weird with the color.
    pub fn with_current<F: Fn(&mut f32)>(&mut self, f: F) {
        use State::*;

        f(match self.state {
            H => &mut self.hsv.h,
            S => &mut self.hsv.s,
            V => &mut self.hsv.v,
        });
    }
}

impl Default for HsvColor {
    fn default() -> Self {
        Self::new()
    }
}

/// What color channel is being manipulated.
/// Can convert to a [`BitImage`].
#[rustfmt::skip]
#[derive(Debug)]
#[state_enum::state_enum]
pub enum State { H, S, V }

impl From<State> for BitImage {
    fn from(s: State) -> Self {
        use State::*;

        match s {
            H => DISPLAY_H,
            S => DISPLAY_S,
            V => DISPLAY_V,
        }
    }
}

pub const DISPLAY_H: BitImage = BitImage::new(&[
    [0, 1, 0, 1, 0],
    [0, 1, 0, 1, 0],
    [0, 1, 1, 1, 0],
    [0, 1, 0, 1, 0],
    [0, 1, 0, 1, 0],
]);

pub const DISPLAY_S: BitImage = BitImage::new(&[
    [0, 1, 1, 1, 0],
    [0, 1, 0, 0, 0],
    [0, 1, 1, 1, 0],
    [0, 0, 0, 1, 0],
    [0, 1, 1, 1, 0],
]);

pub const DISPLAY_V: BitImage = BitImage::new(&[
    [0, 1, 0, 1, 0],
    [0, 1, 0, 1, 0],
    [0, 1, 0, 1, 0],
    [0, 1, 0, 1, 0],
    [0, 0, 1, 0, 0],
]);
