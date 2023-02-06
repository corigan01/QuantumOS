;/*
;   ____                 __               __                __
;  / __ \__ _____ ____  / /___ ____ _    / /  ___  ___ ____/ /__ ____
; / /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ _ \/ _ `/ _  / -_) __/
; \___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/\___/\_,_/\_,_/\__/_/
;   Part of the Quantum OS Project
;
; Copyright 2023 Gavin Kellam
;
; Permission is hereby granted, free of charge, to any person obtaining a copy of this software and
; associated documentation files (the "Software"), to deal in the Software without restriction,
; including without limitation the rights to use, copy, modify, merge, publish, distribute,
; sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is
; furnished to do so, subject to the following conditions:
;
; The above copyright notice and this permission notice shall be included in all copies or substantial
; portions of the Software.
;
; THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT
; NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
; NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
; DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT
; OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
;*/


; This is where it all begins...
[bits 16]
[org 0x7c00]

; where to load the kernel to
STAGE_OFFSET equ 0x1000


; BIOS sets boot drive in 'dl'; store for later use
mov [BOOT_DRIVE], dl

; setup stack
mov bp, 0x9000
mov sp, bp

call _start

%include "bit16_print.asm"
%include "initmem.asm"
%include "disk.asm"
%include "gdt.asm"
%include "protected.asm"
%include "real.asm"

[bits 16]
_start:
    call qos_intro
    call init_a20
    call load_stage
    call reset_display
    call switch_to_32bit

    jmp $

[bits 16]
load_stage:
    mov bx, STAGE_OFFSET  ; bx -> destination to put the stage
    mov dl, [BOOT_DRIVE]  ; dl -> disk

    call disk_load

    ret

[bits 32]
BEGIN_32BIT:
    call Entry16

    push long STAGE_ADDRS
    call [STAGE_ADDRS] ; give control to the loader

    jmp $

; boot drive variable
BOOT_DRIVE db 0
STAGE_ADDRS  db 0

; padding
times 510 - ($-$$) db 0

; magic number
dw 0xaa55