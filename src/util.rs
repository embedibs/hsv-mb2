//! Utilities

use critical_section_lock_mut::LockMut;

use microbit::hal::timer::{Instance, Timer};

/// 100ms at 1MHz count rate.
pub const DEBOUNCE_TIME: u32 = 100 * 1_000_000 / 1000;

/// Debounce Helper
pub fn debounce<T, F>(timer: &LockMut<Timer<T>>, f: F)
where
    T: Instance,
    F: FnOnce(),
{
    timer.with_lock(|timer| {
        if timer.read() == 0 {
            f();
            timer.start(DEBOUNCE_TIME);
        }
    });
}

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
    // WARN: Ask Bart about asserting this
    assert_eq!(list.len(), 3);
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
