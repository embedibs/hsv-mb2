//! Utilities

use critical_section_lock_mut::LockMut;

use microbit::hal::timer::{self, Timer};

/// 100ms at 1MHz count rate.
const DEBOUNCE_TIME: u32 = 100 * 1_000_000 / 1000;

/// Debounce Helper
pub fn debounce<I, F>(timer: &LockMut<Timer<I>>, f: F)
where
    I: timer::Instance,
    F: FnOnce(),
{
    timer.with_lock(|timer| {
        if timer.read() == 0 {
            f();
            timer.start(DEBOUNCE_TIME);
        }
    });
}

/// Debounced button events
pub struct Button<I, F> {
    timer: Timer<I>,
    on_press: F,
}

impl<I, F> Button<I, F>
where
    I: timer::Instance,
    F: Fn(),
{
    pub fn new(timer: Timer<I>, on_press: F) -> Self {
        Self { timer, on_press }
    }

    pub fn handle_event(&mut self) {
        if self.timer.read() == 0 {
            (self.on_press)();
            self.timer.start(DEBOUNCE_TIME);
        }
    }
}

// based on slice::sort_by_key
/// Sorts exactly three elements by key.
pub fn sort3_by_key<T, K, F>(list: &mut [T], mut f: F)
where
    F: FnMut(&T) -> K,
    K: PartialOrd,
{
    sort3(list, |a, b| f(a).lt(&f(b)))
}

// https://www.geeksforgeeks.org/computer-science-fundamentals/sort-3-numbers/
/// Sorts exactly three elements.
pub fn sort3<T, G>(list: &mut [T], mut g: G)
where
    G: FnMut(&T, &T) -> bool,
{
    assert_eq!(list.len(), 3, "length {} does not equal three.", list.len());
    if g(&list[1], &list[0]) {
        list.swap(1, 0);
    }
    if g(&list[2], &list[1]) {
        list.swap(2, 1);
        if g(&list[1], &list[0]) {
            list.swap(1, 0);
        }
    }
}
