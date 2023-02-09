ENTRY (rust_main)

SECTIONS
{
    . = 0x7E00;

   .text : {
       *(.text .text.*)
   }
   .bss : {
       *(.bss .bss.*)
   }
   .data : {
       *(.data .data.*)
   }
   .rodata : {
       *(.rodata .rodata.*)
   }
   .eh_frame : {
       *(.eh_frame .eh_frame.*)
   }
   .eh_frame_hdr : {
       *(.eh_frame_hdr .eh_frame_hdr.*)
   }

   . = ALIGN(512);
}