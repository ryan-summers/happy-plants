#![no_std]
#![no_main]

// pick a panicking behavior
extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// extern crate panic_abort; // requires nightly
// extern crate panic_itm; // logs messages over ITM; requires ITM support
// extern crate panic_semihosting; // logs messages to the host stderr; requires a debugger
use cortex_m_semihosting::{hprintln};

use rtfm;

#[rtfm::app(device = nrf52840_hal::nrf52840_pac)]
const APP: () = {

    #[init]
    fn init() {
        hprintln!("Hello World!").unwrap();
    }
};
