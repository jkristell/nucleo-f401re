## Support package for the [Nucleo-f401re](https://www.st.com/en/evaluation-tools/nucleo-f401re.html) board.

### How to use the examples:

1. Clone this repository

#### Flash using Probe.rs

```cargo flash --chip stm32f401re --example button-interrupt```

Or with cargo embed

```cargo embed --release --example button-rtfm```

### Board properties

 * User led on PA5
 * User button on PC13
 * Serial port through ST-LINK on USART2, Tx: PA2 and Rx: PA3.

This repository is based on https://github.com/therealprof/stm32f407g-disc
