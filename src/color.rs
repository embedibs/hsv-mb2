//! Color

#![allow(unused)]

use hsv::{Hsv, Rgb};

pub struct HsvColor {
    pub hsv: Hsv,
    pub state: State,
}

impl HsvColor {
    pub fn new() -> Self {
        Self {
            hsv: Hsv {
                h: 0.0,
                s: 0.0,
                v: 0.0,
            },
            state: State::default(),
        }
    }

    pub fn to_rgb(&self) -> Rgb {
        Rgb::from(self.hsv)
    }

    pub fn to_display(&self) -> Display {
        Display::from(self.state)
    }

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
/// Can convert to a [`Display`].
#[rustfmt::skip]
#[derive(Debug)]
#[state_enum::state_enum]
pub enum State { H, S, V }

impl From<State> for Display {
    fn from(s: State) -> Self {
        use State::*;

        match s {
            H => DISPLAY_H,
            S => DISPLAY_S,
            V => DISPLAY_V,
        }
    }
}

/// 5x5 display buffer for the BBC Micro:Bit V2.
pub type Display = [[u8; 5]; 5];

#[rustfmt::skip]
pub const DISPLAY_H: Display = [
    [0,1,0,1,0],
    [0,1,0,1,0],
    [0,1,1,1,0],
    [0,1,0,1,0],
    [0,1,0,1,0],
];

#[rustfmt::skip]
pub const DISPLAY_S: Display = [
    [0,1,1,1,0],
    [0,1,0,0,0],
    [0,1,1,1,0],
    [0,0,0,1,0],
    [0,1,1,1,0],
];

#[rustfmt::skip]
pub const DISPLAY_V: Display = [
    [0,1,0,1,0],
    [0,1,0,1,0],
    [0,1,0,1,0],
    [0,1,0,1,0],
    [0,0,1,0,0],
];
