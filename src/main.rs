#![no_main]
#![no_std]

extern crate panic_halt;

use hifive1::{
    hal::core::plic::Priority,
    hal::DeviceResources,
    hal::{core::CorePeripherals},
    hal::{e310x::interrupt::Interrupt, prelude::*},
    pin, sprintln,
};
use riscv_rt::entry;
/* use riscv::register::{mstatus, mie}; */
/* Sample function to toggle a led on interrupt handle.
For now we will just write to stdout. */
fn led_interrupt() {
    sprintln!("INTERRUPTED");
}

/* PLIC Interrupt test (Timer interrupt) */
/* We will export this as MachineExternal as mentioned by the
riscv-rt documentation. This function will serve as the external
interrupts handler written in the mtvec in direct mode.
We will use it to test RTC timer interrupts with PLIC, but could also
be done with CLINT's mtimecmp register. */
#[no_mangle]
#[export_name = "MachineExternal"]
unsafe extern "C" fn MachineExternal() {
    sprintln!("Received an interrupt");
    /* Steal the PLIC again, now to claim the interrupt */
    let mut plic = CorePeripherals::steal().plic;
    /* Claim the pending interrupt */
    let intr = plic.claim.claim().unwrap();
    /* Match the interrupt (we only have one to handle) */
    match intr {
        Interrupt::RTC => {
            led_interrupt();
            let rtc = &*hifive1::hal::e310x::RTC::ptr();
            rtc.rtccmp.modify(|r, w| w.bits(r.bits() + 65536));
        }
        _ => {
            sprintln!("We received an interrupt we don't know how to handle");
        }
    }
    /* Complete the interrupt */
    plic.claim.complete(intr);
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

    /* RTC CONFIGURATION */
    /* Get the RTC peripheral to configure it */
    /* Call the constrain() function to return a copy of the value */
    let mut rtc = peripherals.RTC.constrain();
    
    rtc.enable();
    /* Set default counting scale */
    rtc.set_scale(0);
    /* Initialize it to zero */
    rtc.set_rtc(0);
    /* Similar to initializing mtimecmp, target value to compare rtc register with */
    rtc.set_rtccmp(2000);
    /* Enable our rtc peripheral counter */
    rtc.enable();

    /* PLIC CONFIGURATION */
    unsafe {
        /* Steal the PLIC controller temporarily to modify (singleton) */
        let mut plic = CorePeripherals::steal().plic;
        /* Now working with PLIC's rtc interrupt */
        /* Set max priority to the rtc interrupt */
        plic.rtc.set_priority(Priority::P7);
        /* Enable the rtc interrupt in PLIC */
        plic.rtc.enable();
        /* Set the interrupt threshold (P0 = no threshold) */
        plic.threshold.set(Priority::P1);
        /* Enable machine external interrupts to allow RTC peripheral
        to notify us */
        plic.mext.enable();
    }
    sprintln!("Finished configuring everything");
    /* Main system loop (leave empty to test RTC) */
    loop {}
}
