[![crates.io version](https://img.shields.io/crates/v/nucleo-f401re.svg)](https://crates.io/crates/nucleo-f401re)
[![docs.rs](https://docs.rs/nucleo-f401re/badge.svg)](https://docs.rs/nucle-f401re)

## Support package for the [Nucleo-f401re](https://www.st.com/en/evaluation-tools/nucleo-f401re.html) board.

### Running the examples

1. Clone this repository

#### Run

```
cargo run --example button-interrupt
```

If probe fails to flash your board you probably need to update the firmware on the onboard programmer.
The updater can be found at: https://www.st.com/en/development-tools/stsw-link007.html

### Board properties

 * User led on PA5
 * User button on PC13
 * Serial port through ST-LINK on USART2, Tx: PA2 and Rx: PA3.

This repository is based on https://github.com/therealprof/stm32f407g-disc
