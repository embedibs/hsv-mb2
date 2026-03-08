# hsv-mb2

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
