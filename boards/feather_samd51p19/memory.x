/* Preserve the resident UF2 bootloader in the first 16 KiB of flash. */
ENTRY(Reset)
MEMORY
{
    FLASH (rx)  : ORIGIN = 0x00004000, LENGTH = 496K
    RAM   (rwx) : ORIGIN = 0x20000000, LENGTH = 192K
}
_estack = ORIGIN(RAM) + LENGTH(RAM);
SECTIONS
{
    .vectors : { . = ALIGN(256); KEEP(*(.vectors)); . = ALIGN(4); } > FLASH
    .text :
    {
        . = ALIGN(4); *(.text .text.*); *(.rodata .rodata.*);
        . = ALIGN(4); _etext = .;
    } > FLASH
    _sidata = LOADADDR(.data);
    .data :
    {
        . = ALIGN(4); _sdata = .; *(.data .data.*);
        . = ALIGN(4); _edata = .;
    } > RAM AT > FLASH
    .bss (NOLOAD) :
    {
        . = ALIGN(4); _sbss = .; *(.bss .bss.*); *(COMMON);
        . = ALIGN(4); _ebss = .;
    } > RAM
    /DISCARD/ : { *(.ARM.exidx .ARM.exidx.*); }
    ASSERT(_ebss <= ORIGIN(RAM) + LENGTH(RAM), "RAM overflow")
}
