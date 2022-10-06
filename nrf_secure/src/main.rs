#![no_main]
#![no_std]
#![feature(abi_c_cmse_nonsecure_call)]

use alloc::sync::Arc;
#[cfg(feature = "defmt")]
use defmt_rtt as _; // global logger

use hal::pac::Peripherals;
use nrf9160_hal as hal;
use nrf9160_hal::gpio::Level;
use embedded_hal::digital::v2::OutputPin;
use core::mem::transmute;
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

struct  control_ns{
    npriv: u32,
    spsel: u32,
}

struct tz_nonsecure_setup_conf {
    msp_ns: u32,
    psp_ns: u32,
    vtor_ns: u32,
    control: control_ns,
}

fn tz_nonsecure_state_setup(spm_ns_conf: tz_nonsecure_setup_conf){

}

fn tz_nonsecure_exception_prio_config(prio:u32){

}

fn tz_nbanked_exception_target_state_set(exception:u32){

}

fn tz_nonsecure_system_reset_req_block(enable:u32){

}

fn tz_sau_configure(enable:u32, allns:u32){

}


fn nonsecure_state_setup(spm_ns_conf: tz_nonsecure_setup_conf) {
    /* Configure core register block for Non-Secure state. */
	tz_nonsecure_state_setup(spm_ns_conf);
	/* Prioritize Secure exceptions over Non-Secure */
	tz_nonsecure_exception_prio_config(1);
	/* Set non-banked exceptions to target Non-Secure */
	tz_nbanked_exception_target_state_set(0);
	/* Configure if Non-Secure firmware should be allowed to issue System
	 * reset. If not it could be enabled through a secure service.
	 */
	tz_nonsecure_system_reset_req_block(
		CONFIG_SPM_BLOCK_NON_SECURE_RESET
	);
	/* Allow SPU to have precedence over (non-existing) ARMv8-M SAU. */
	tz_sau_configure(0, 1);
}

#[entry]
fn main() -> ! {
    defmt::println!("Hello from Secure World!");
    pub const FLASH_REGION_SIZE: u32 = 32 * 1024;
    pub const RAM_REGION_SIZE: u32 = 8 * 1024;

    let mut peripherals: hal::pac::Peripherals = hal::pac::Peripherals::take().unwrap();
    let core_peripherals = hal::pac::CorePeripherals::take().unwrap();
    let mut sau = core_peripherals.SAU;
    let mut spu = peripherals.SPU_S;
    let mut msp = 

    let ns_flash_start = 0x00050000;
    let ns_flash_end = 0x00110000;
    let ns_flash = ns_flash_start..ns_flash_end;

    let ns_ram_start = 0x20000000;
    let ns_ram_end = 0x20010000;
    let ns_ram = ns_ram_start..ns_ram_end;

    // Disable SAU as we will deal with SPU and set ALLNS bit as 1 so all memory is non secure now. 
    unsafe{ 
        sau.ctrl.modify(|mut ctrl| {
        ctrl.0 = 0x0000002;
        ctrl});

        // Also set the stack pointer of nonsecure
        cortex_m::register::msp::write_ns(ns_ram_end);

        spu.gpioport[0].perm.write(|w|{
            w.bits(0x00000000)
        });
    };


    // set secure and non secure flash area 
    for (index, address, region) in spu
        .flashregion
        .iter()
        .enumerate()
        .map(|(index, region)| (index, index as u32 * FLASH_REGION_SIZE, region))
    {
        if ns_flash.contains(&address) {
            region.perm.write(|w| {
                w.execute()
                    .enable()
                    .read()
                    .enable()
                    .write()
                    .enable()
                    .secattr()
                    .non_secure()
            });
            defmt::println!("Flash {} @ {:X}..={:X} = NS", index, address, address + FLASH_REGION_SIZE);
        } else {
            region.perm.write(|w| {
                w.execute()
                    .enable()
                    .read()
                    .enable()
                    .write()
                    .enable()
                    .secattr()
                    .secure()
            });
            defmt::println!("Flash {} @ {:X}..={:X} = S", index, address, address + FLASH_REGION_SIZE);
        }
    }
    
    
    defmt::println!("non secure flash setting done");

    for (index, address, region) in spu
        .ramregion
        .iter()
        .enumerate()
        .map(|(index, region)| (index, 0x20000000 + index as u32 * RAM_REGION_SIZE, region))
    {
        if ns_ram.contains(&address) {
            region.perm.write(|w| {
                w.execute()
                    .enable()
                    .read()
                    .enable()
                    .write()
                    .enable()
                    .secattr()
                    .non_secure()
            });
        } else {
            region.perm.write(|w| {
                w.execute()
                    .enable()
                    .read()
                    .enable()
                    .write()
                    .enable()
                    .secattr()
                    .secure()
            });
        }
    }
    
    defmt::println!("non secure RAM area setting done");
    
    let flash_access_error = spu.events_flashaccerr.read().bits();
    let ram_access_error = spu.events_ramaccerr.read().bits();
    defmt::println!("flash access error {:?}", flash_access_error);
    defmt::println!("ram access error {:?}", ram_access_error);

    // read the securety attribute of gpio
    let port_value = spu.gpioport[0].perm.read().bits();
    defmt::println!("Gpio Port permission register value {:?}", port_value);
    
    let mut scb = core_peripherals.SCB;
    
    // enable secure fault 
    
    scb.enable(Exception::SecureFault);

    unsafe {
        defmt::println!("jumping to non secure code");
        let base_img_addr = ns_flash_start as u32;
        let stack_pointer = base_img_addr;
        let reset_vector = base_img_addr + 4;
        
        nonsecure_state_setup(ns_spm_conf);
        scb.vtor.write(base_img_addr);
        cortex_m::register::psp::write(0x0000000);
        

        defmt::println!("NS Stack Pointer {}", stack_pointer);
        defmt::println!("NS reset vector {}", reset_vector);
        let ns_reset_vector: extern "C-cmse-nonsecure-call" fn() -> ! =
                                    core::mem::transmute::<u32, _>(reset_vector + 4);
        
        cortex_m::register::msp::write_ns(stack_pointer);

        cortex_m::asm::dsb();
        cortex_m::asm::isb();   
        
        ns_reset_vector()
    }
    defmt::println!("this should br printed");
    loop{
    }
}

#[allow(non_snake_case)]
#[exception]
fn SecureFault() {
    defmt::println!("Secure Fault!!!");
    loop {}
}
