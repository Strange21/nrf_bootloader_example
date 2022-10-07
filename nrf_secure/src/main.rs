#![no_main]
#![no_std]
#![feature(abi_c_cmse_nonsecure_call)]


use crate::flash::memory;
pub mod flash;

#[cfg(feature = "defmt")]
use defmt_rtt as _; // global logger

use nrf9160_hal as hal;
use cortex_m::cmse::{AccessType, TestTarget};
use cortex_m::peripheral::sau::{SauRegion, SauRegionAttribute, SauError};
use cortex_m::peripheral::scb::Exception;
use cortex_m::peripheral::{Peripherals, self};
use cortex_m_rt::{entry, exception};

#[panic_handler] // panicking behavior
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}

#[entry]
fn main() -> ! {
    let mut peripherals: hal::pac::Peripherals = hal::pac::Peripherals::take().unwrap();
    let core_peripherals = hal::pac::CorePeripherals::take().unwrap();
    let mut sau = core_peripherals.SAU;
    let ns_flash_start = 0x0008_0000;

    let AIRCR_VECT_KEY_PERMIT_WRITE = 0x05FA0000;

    // disable Sau and make all reagion non secure
    unsafe {
        sau.ctrl.modify(|mut ctrl| {
            ctrl.0 = 0x00000002;
            ctrl
        });
    }
    defmt::println!("Hello from Secure World!");
    memory::config_flash(&peripherals.SPU_S);
    memory::config_ram(&peripherals.SPU_S);
    memory::config_peripherals(&peripherals.SPU_S);

    let mut control_reg = cortex_m::register::control::read();
    let npriv = cortex_m::register::control::Npriv::Privileged;
    let spsel = cortex_m::register::control::Spsel::Msp;

    let mut scb = core_peripherals.SCB;
    let mut aircr = 0x0000_0000;
    aircr |= AIRCR_VECT_KEY_PERMIT_WRITE;
    aircr |= 0x0000_6000;   // writing 1 to PRIS bit which is 14th bit of AIRCR register
    
    // enable secure fault 
    scb.enable(Exception::SecureFault);

    unsafe {
        defmt::println!("jumping to non secure code");
        cortex_m::register::psp::write(0x00000000);
        control_reg.set_spsel(spsel);
        control_reg.set_npriv(npriv);
        cortex_m::register::control::Control::set_spsel(&mut control_reg, spsel);

        /* Prioritize Secure exceptions over Non-Secure */

        /* Set non-banked exceptions to target Non-Secure */

        /* Configure if Non-Secure firmware should be allowed to issue System
        * reset. If not it could be enabled through a secure service.
        */
        scb.aircr.write(aircr);
    
        // Also set the stack pointer of nonsecure
        cortex_m::register::msp::write_ns(ns_flash_start);
        
    }
    memory::jump_ns(ns_flash_start, &scb);
    loop{}
}

#[allow(non_snake_case)]
#[exception]
fn SecureFault() {
    defmt::println!("Secure Fault!!!");
    loop {}
}
