#![no_main]
#![no_std]

extern crate panic_halt;

use hifive1::{
    hal::DeviceResources,
    hal::{core::CorePeripherals},
    hal::{e310x::interrupt::Interrupt, prelude::*},
    pin, sprintln,
};
use riscv_rt::entry;
use riscv::register::{mstatus, mie};
/* Sample function to toggle a led on interrupt handle.
For now we will just write to stdout. */
unsafe fn led_interrupt() {
    sprintln!("INTERRUPTED");
    let dr = DeviceResources::steal();
    let gpio = dr.pins;
    let gpio13 = pin!(gpio, dig13);
    /* Get system board LED */
    let mut test_led = gpio13.into_output();
    test_led.toggle().unwrap();
}

/* CLINT Interrupt test (Timer interrupt) */
/* We will export this as MachineTimer as mentioned by the
riscv-rt documentation. This function will serve as the external
interrupts handler written in the mtvec in direct mode.
We will use it to test mtimecmp interrupts with CLINT 
*/
#[no_mangle]
#[export_name = "MachineTimer"]
unsafe fn MachineTimer() {
    // Call our handler
    led_interrupt();
    /* Steal the CLINT */
    let mut clint = CorePeripherals::steal().clint;
    /* rewrite the mtimecmp register to clear the interrupt */
    clint.mtimecmp.set_mtimecmp(clint.mtime.mtime() + 100000);
    sprintln!("mtimecmp: {}", clint.mtimecmp.mtimecmp());
}

/* Main riscv-rt entry point */
#[entry]
fn main() -> ! {
    /* Get the ownership of the device resources singleton */
    let resources = DeviceResources::take().unwrap();
    let peripherals = resources.peripherals;

    /* Configure system clock */
    let sysclock =
        hifive1::configure_clocks(peripherals.PRCI, peripherals.AONCLK, 64.mhz().into());
    /* Get the board pins */
    let gpio = resources.pins;

    // UNUSED let mut sltimer = Sleep::new(clint.mtimecmp, sysclock);

    /* Configure stdout for debugging */
    hifive1::stdout::configure(
        peripherals.UART0,
        pin!(gpio, uart0_tx),
        pin!(gpio, uart0_rx),
        115_200.bps(),
        sysclock,
    );

    /* TEST: set digital13 as output */
    // let gpio13 = pin!(gpio, dig13);
    /* Get system board LED */
    // let mut test_led = gpio13.into_output();
    
    /* Get the CLINT */
    let mut clint = resources.core_peripherals.clint;
    /* Set the mtimecmp register to 1 second, that is 64.000.000 clock cycles    */
    clint.mtimecmp.set_mtimecmp(clint.mtime.mtime() + 100000);
    /* CLINT CONFIGURATION */
    unsafe {
        /* Activate mtime interrupt from the MIE register */
        mie::set_mtimer();
        /* Activate global interrupts (mie bit) */
        mstatus::set_mie();
    }
    sprintln!("Finished configuring everything");
    /* Main system loop (leave empty to test RTC) */
    loop {}
}
