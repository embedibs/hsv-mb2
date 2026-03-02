#![no_main]
#![no_std]

pub mod color;
pub mod rgb_display;
pub mod thing;
pub mod util;

// rgb channel and duty cycle
type Pulse = (u8, f32);

struct RgbPulseFrame<I> {
    items: I,
    curr: Option<Pulse>,
}

impl<I> RgbPulseFrame<I>
where
    I: Iterator<Item = Pulse>,
{
    fn new(items: I) -> Self {
        Self { items, curr: None }
    }
}

pub fn new_frame(c: color::HsvColor) {
    let rgb = hsv::Rgb::from(c.hsv);

    let mut items = [(0b100u8, rgb.r), (0b010u8, rgb.g), (0b001u8, rgb.b)];
    sort_by_key(&mut items, |v| v.1);

    let frame = RgbPulseFrame::new(items.into_iter().chain([(0b000u8, 1.0)]));
}

use microbit::hal::{pac, timer::Timer};

struct RgbDisplay<I> {
    curr_frame: Option<RgbPulseFrame<I>>,
    next_frame: Option<RgbPulseFrame<I>>,
    timer2: Timer<pac::TIMER2>,
    //rgb_pins:
}

impl<I> RgbDisplay<I>
where
    I: Iterator<Item = Pulse>,
{
    pub fn step(&mut self, c: color::HsvColor) {
        let rgb = hsv::Rgb::from(c.hsv);

        let mut items = [(0b100u8, rgb.r), (0b010u8, rgb.g), (0b001u8, rgb.b)];
        sort_by_key(&mut items, |v| v.1);

        let frame = RgbPulseFrame::new(items.into_iter().chain([(0b000u8, 1.0)]));

        self.next_frame = Some(frame);
    }
}

pub fn sort_by_key<T, K, F>(list: &mut [T], mut f: F)
where
    F: FnMut(&T) -> K,
    K: PartialOrd,
{
    bubblesort(list, |a, b| f(a).lt(&f(b)))
}

pub fn bubblesort<T, G>(list: &mut [T], mut g: G)
where
    G: FnMut(&T, &T) -> bool,
{
    let n = list.len();
    for i in 0..n - 1 {
        for j in 0..n - 1 - i {
            if g(&list[j + 1], &list[j]) {
                list.swap(j, j + 1);
            }
        }
    }
}
