# hsv-mb2

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

## Video

https://github.com/user-attachments/assets/d2757e51-06db-42ee-9e17-5beaef508c68

## License

This project is licensed under the [MIT License][License].

[License]: ./LICENSE
[Video]: ./VIDEO.mp4
