# si7021

A Rust I²C driver for the [Si7021] hygrometer and thermometer.

## Example

```rust
extern crate i2cdev;
extern crate si7021;
extern crate i2csensors;

use i2cdev::linux::LinuxI2CDevice;
use si7021::{Si7021, SI7021_I2C_ADDRESS};
use i2csensors::{Hygrometer, Thermometer};

fn main() {
    let device = LinuxI2CDevice::new("/dev/i2c-1", SI7021_I2C_ADDRESS).unwrap();
    let mut si7021 = Si7021::new(device);

    println!("humidity:    {:6.2} %", si7021.relative_humidity().unwrap());
    println!("temperature: {:6.2} °C", si7021.temperature_celsius().unwrap());
}
```

## License

Licensed under either of

 * Apache License, Version 2.0, (http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license (http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.

[Si7021]: https://www.silabs.com/documents/public/data-sheets/Si7021-A20.pdf
