use nrf9160_hal::pac::{SPU_S};
use cortex_m::{self, peripheral::{SCB, sau::SauRegionAttribute}};


pub fn config_flash(spu:&SPU_S){
    pub const FLASH_REGION_SIZE: u32 = 32 * 1024;
    
    // extern "C" {
    //     static mut _ns_flash_start: u32;
    //     static mut _ns_flash_end: u32;
    //     static mut _ns_ram_start: u32;
    //     static mut _ns_ram_end: u32;
    // }
    // unsafe{
    //     _ns_flash_start = 0x0008_0000;
    //     _ns_flash_end   = 0x0010_0000;
    //     _ns_ram_start   = 0x20010000;
    //     _ns_ram_end     = 0x20020000;
    // }
    
    // let ns_flash_start = unsafe { core::mem::transmute::<_, u32>(&_ns_flash_start) };
    // let ns_flash_end = unsafe { core::mem::transmute::<_, u32>(&_ns_flash_end) };
    // let ns_ram_start = unsafe { core::mem::transmute::<_, u32>(&_ns_ram_start) };
    // let ns_ram_end = unsafe { core::mem::transmute::<_, u32>(&_ns_ram_end) };

    let ns_flash_start = 0x0008_0000;
    let ns_flash_end = 0x0010_0000;
    let ns_flash = ns_flash_start..ns_flash_end;

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
                    .set_bit()
                    .read()
                    .enable()
                    .write()
                    .enable()
                    .secattr()
                    .non_secure()
                    .lock()
                    .locked()
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
                    .lock()
                    .locked()
            });
            defmt::println!("Flash {} @ {:X}..={:X} = S", index, address, address + FLASH_REGION_SIZE);
        }
    }

    // let sec_att = spu.flashregion[16].perm.read().secattr().bit();
    // let read_bit = spu.flashregion[16].perm.read().read().bit();
    // let write_bit = spu.flashregion[16].perm.read().write().bit();
    // let execute_bit = spu.flashregion[16].perm.read().execute().bit();
    // defmt::println!("secure attribute bit of falsh sector 16 is {:?}", sec_att);
    // defmt::println!("read  bit of falsh sector 16 is {:?}", read_bit);
    // defmt::println!("write  bit of falsh sector 16 is {:?}", write_bit);
    // defmt::println!("execute  bit of falsh sector 16 is {:?}", execute_bit);
    // defmt::println!("non secure flash setting done");
    let flash_access_error = spu.events_flashaccerr.read().bits();
    defmt::println!("flash access error {:?}", flash_access_error);
    defmt::println!("Flash configuration done");
    

}

pub fn config_ram(spu:&SPU_S){
    pub const RAM_REGION_SIZE: u32 = 8 * 1024;
    let ns_ram_start = 0x2002_0000;
    let ns_ram_end = 0x2004_0000;
    let ns_ram = ns_ram_start..ns_ram_end;

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
    let ram_access_error = spu.events_ramaccerr.read().bits();
    defmt::println!("ram access error {:?}", ram_access_error);
}

pub fn config_peripherals(spu:&SPU_S){
    unsafe{
        // defining GPIO as Non Secure
        spu.gpioport[0].perm.write(|w|{
            w.bits(0x00000000)
        });
    }
    
    // read the securety attribute of gpio
    let port_value = spu.gpioport[0].perm.read().bits();
    defmt::println!("Gpio Port permission register value {:?}", port_value);
}

pub fn jump_ns(ns_app_address:u32, scb:&SCB){
    unsafe{
        defmt::println!("jumping to non secure code");
        let base_img_addr = ns_app_address as u32;
        let stack_pointer = base_img_addr;
        let reset_vector  = base_img_addr + 4;
        
        // Also set the stack pointer of nonsecure
        cortex_m::register::msp::write_ns(ns_app_address);

        defmt::println!("NS Stack Pointer {}", stack_pointer);
        defmt::println!("NS reset vector {}", reset_vector);

        /* Configure core register block for Non-Secure state. */
        scb.vtor.write(base_img_addr);
        let ns_reset_vector = core::mem::transmute::<u32, extern "C-cmse-nonsecure-call" fn() -> !>(reset_vector as u32);
            
        cortex_m::register::msp::write_ns(stack_pointer);

        cortex_m::asm::dsb();
        cortex_m::asm::isb();   
        
        ns_reset_vector()
    }

    loop{}
    
}

pub fn check_memory_permission(addr:u32) -> SauRegionAttribute{
    // get the permission for non secure flash
    let value = cortex_m::asm::tt(addr as *mut u32);

    let s = value & (1 << 22) > 0;
    let nsrw = value & (1 << 21) > 0;
    
    match (s, nsrw) {
        (_, true) => SauRegionAttribute::NonSecureCallable,
        (true, false) => SauRegionAttribute::Secure,
        (false, false) => SauRegionAttribute::NonSecure,
    }
}