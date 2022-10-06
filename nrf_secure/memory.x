/* Linker script for the nRF9160 in Non-secure mode. It assumes you have the
Nordic Secure Partition Manager installed at the bottom of flash and that
the SPM is set to boot a non-secure application from the FLASH origin below. */

MEMORY
{
    FLASH : ORIGIN = 0x00000000, LENGTH = 256K
    RAM : ORIGIN = 0x20000000, LENGTH = 64K
}


