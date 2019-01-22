## Support package for the [Nucleo-f401re](https://www.st.com/en/evaluation-tools/nucleo-f401re.html) board.

### How to use the examples:

1. Clone this repository

2. Start openocd and connect to the target.

 ```
 openocd -f discovery.cfg
 ```

3. And then run the examples from another terminal.

 ```
 cargo run --example gpio_hal_blinky
 ```

### Board properties

 * User led on PA5
 * User button on PC13
 * Serial port through ST-LINK on USART2, Tx: PA2 and Rx: PA3.



This repository is based on https://github.com/therealprof/stm32f407g-disc
