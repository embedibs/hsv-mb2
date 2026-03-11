# hsv-mb2

Ethan Dibble 2026  

HSV and software PWM demo using and RGB LED and a potentiometer.  

## Pins

|MB2|Edge|LED|POT|
|-|-|-|-|
|p0_04|P02||P02|
|p0_10|P08|P01||
|p0_09|P09|P03||
|p1_02|P16|P04||
||+3.3V|P02|P03|
||GND||P01|

## Dependencies

```
rustup target add thumbv7em-none-eabihf
rustup component add llvm-tools
cargo install cargo-binutils
cargo install --locked probe-rs-tools
```

## Build and Run

```
cargo embed --release
```

## Features

- log: enables rtt logging

## How the demo works

The Micro:Bit A and B buttons cycle forward and back through Hue, Saturation,
Value. Turning the knob will adjust the current HSV state as shown on the mb2
display and effect the RGB LED.

## Notes

Only the PWM required interrupts, but I wanted to also explore interrupts with
button debouncing and the nonblocking display.  

I didn't need to store the button in the `Button` struct to know that the button is pressed
because the gpiote event is triggered on `hi_to_lo`; to respond to press *and* release,
I would store the button and react on toggle, passing the result of `button.is_low()` to
the event handler. I also used one timer per button since the board has five available.  

I had difficulty in this project trying to abstract too much of the PWM. I scrapped
the effort and built off a skeleton `RgbDisplay` provided by Bart Massey. The result
interrupts at most 4 times for each color channel and the end buffer, and as little as
once for just the end buffer if all color channels are zero.  

An issue was also discovered with the HAL whereby Delaying the timer zero cycles blocks
until the timer overflows because the `COMPARE` event is not immediately raised.
At 1MHz timer frequency and 32-bits of accuracy, setting the delay to zero should
actually delay 1.19 hours. It's safest to always delay a minimum of one second.  

I also made a small attribute macro for [state enums][state-enum].  

## Video

https://github.com/user-attachments/assets/d2757e51-06db-42ee-9e17-5beaef508c68

## Acknowledgements

Bart Massey for [no_std HSV to sRGB][HSV to RGB] and
[critical-section-lock-mut][LockMut].

## License

This project is licensed under the [MIT License][License].

[License]: ./LICENSE
[Video]: ./VIDEO.mp4
[HSV to RGB]: https://github.com/pdx-cs-rust-embedded/hsv
[LockMut]: https://crates.io/crates/critical-section-lock-mut
[state-enum]: https://github.com/edibblepdx/state-enum
