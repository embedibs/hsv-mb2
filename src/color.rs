pub struct HsvColor {
    hsv: hsv::Hsv,
    state: State,
}

impl HsvColor {
    pub fn new() -> Self {
        Self {
            hsv: hsv::Hsv {
                h: 0.0,
                s: 0.0,
                v: 0.0,
            },
            state: State::default(),
        }
    }

    pub fn to_rgb(&self) -> hsv::Rgb {
        hsv::Rgb::from(self.hsv)
    }

    pub fn to_display(&self) -> Display {
        Display::from(self.state)
    }

    pub fn set_current(&mut self, v: f32) {
        self.with(|x| *x = v);
    }

    // If I later decide to do anything else wacky and weird with the color.
    pub fn with<F: Fn(&mut f32)>(&mut self, f: F) {
        use State::*;

        f(&mut match self.state {
            H => self.hsv.h,
            S => self.hsv.s,
            V => self.hsv.v,
        });
    }
}

/// What color channel is being manipulated.
/// Can convert to a [`Display`].
#[rustfmt::skip]
#[derive(Copy, Clone, Default)]
pub enum State { #[default] H, S, V }

impl State {
    const STATES: [Self; 3] = [Self::H, Self::S, Self::V];

    pub fn next(&self) -> Self {
        Self::STATES[(*self as usize + 1) % 3]
    }

    pub fn prev(&self) -> Self {
        Self::STATES[(*self as usize + 2) % 3]
    }
}

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
