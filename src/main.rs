#![no_main]
#![no_std]

extern crate panic_halt;

use core::borrow::BorrowMut;

use hifive1::{
    hal::DeviceResources,
    hal::{prelude::*, e310x::interrupt},
    hal::delay::Sleep,
    sprintln,
    pin,
};

/* use e310x::{
    interrupt::Interrupt,
}; */

use riscv_rt::entry;

fn LED_INT() {
    sprintln!("INTERRUPTED");
}

#[entry]
fn main() -> ! {
    /* Get the ownership of the device resources singleton */
    let mut resources = DeviceResources::take().unwrap();
    let peripherals = resources.peripherals;
        
    /* Configure system clock */
    let sysclock = hifive1::configure_clocks(peripherals.PRCI, peripherals.AONCLK, 320.mhz().into());
    /* Get the board pins */
    let gpio = resources.pins;
    let mut sltimer = Sleep::new(resources.core_peripherals.clint.mtimecmp, sysclock);

    hifive1::stdout::configure(
        peripherals.UART0,
        pin!(gpio, uart0_tx),
        pin!(gpio, uart0_rx),
        115_200.bps(),
        sysclock,
    );


    /* TEST: set digital13 as output */
    let gpio13 = pin!(gpio, dig13);
    let gpio1 = pin!(gpio, dig11);
    /* Get system board LED */
    let mut led = gpio13.into_output();
    /* Create an input for a button in GPIO1 */
    gpio1.into_pull_up_input();

    interrupt!(GPIO3, LED_INT);

    /* Main system loop */
    loop {
        led.toggle().unwrap();
        sltimer.delay_ms(1000u32);
    }
}