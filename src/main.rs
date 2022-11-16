#![no_main]
#![no_std]

extern crate panic_halt;

use hifive1::{
    hal::DeviceResources,
    hal::core::plic::Priority,
    hal::{core::CorePeripherals},
    hal::{e310x::{interrupt::Interrupt, plic::{priority, PRIORITY}, PLIC, GPIO0}, prelude::*, core::plic::{self, INTERRUPT}, gpio::gpio0},
    pin, sprintln,
};
use riscv_rt::entry;
use riscv::{register::{mstatus, mie}, interrupt};
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

/* PLIC Interrupt test (External GPIO interrupt)
   This will handle any interrupt from external GPIO */
#[no_mangle]
#[export_name = "MachineExternal"]
unsafe fn MachineExternal() {
    sprintln!("Received an interrupt!");
    let mut plic = CorePeripherals::steal().plic;
    let extintr = plic.claim.claim().unwrap();
    match extintr {
        Interrupt::GPIO12 => {
            led_interrupt();
        },
        _ => {sprintln!("Unknown external interrupt received {:?}", extintr)}
    }
    plic.claim.complete(extintr);
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

    unsafe {sprintln!("Rise_IP: {:?}", (*GPIO0::ptr()).rise_ie.read().bits());}

    /* TEST: set digital13 as output */
    // let gpio13 = pin!(gpio, dig13);
    /* Get system board LED */
    // let mut test_led = gpio13.into_output();
    let gpio12 = pin!(gpio, dig12);
    let btn = gpio12.into_pull_up_input();
    
    
    /* Get the CLINT */
    //let mut clint = resources.core_peripherals.clint;
    let mut plic = resources.core_peripherals.plic;
    /* Set the mtimecmp register to 1 second, that is 64.000.000 clock cycles    */
    //clint.mtimecmp.set_mtimecmp(clint.mtime.mtime() + 100000);
    /* CLINT CONFIGURATION */
    unsafe {
        // TEMPORARY DISABLE
        /* Activate mtime interrupt from the MIE register */
        //mie::set_mtimer();
        let rplic = &*hifive1::hal::e310x::PLIC::ptr();
        //let mut i = 0;
        for (i, p) in rplic.priority.iter().enumerate() {
            if i >= 7 && i <= 38 {
                p.write(|w| w.bits(0xffffffff));
            } else {
                p.write(|w| w.bits(0));
            }
            sprintln!("Iteracion num: {}", i);
        }
        

        
        (*GPIO0::ptr()).fall_ie.write(|w| w.bits(0xffffffff));
        (*GPIO0::ptr()).fall_ip.write(|w| w.bits(0xffffffff));
        /* Activate global interrupts (mie bit) */
        mstatus::set_mie();
        plic.threshold.set(Priority::P0);
        plic.mext.enable();
        // plic.gpio12.enable();
        // plic.gpio12.set_priority(Priority::P7);
    }
    sprintln!("Finished configuring everything");
    /* Main system loop (leave empty to test RTC) */
    loop {
        let mut delaycount = 0;
        for i in 0..100000 {
            delaycount += 1;
        }
        sprintln!("Pin 4 (dig12) is pressed?: {}", btn.is_high().unwrap());
        unsafe {
            sprintln!("Rise_IP: {:?}", (*GPIO0::ptr()).fall_ip.read().bits());
            sprintln!("Rise_IP: {:?}", (*GPIO0::ptr()).fall_ie.read().bits());
            sprintln!("Pending: {:?}", plic.mext.is_pending());
        }
    }
}
