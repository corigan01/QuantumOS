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

[bits 16]

enter_unreal:
   cli                    ; no interrupts
   push ds                ; save real mode
   push ss

   lgdt [gdtinfo]         ; load gdt register

   mov  eax, cr0          ; switch to pmode by
   or al,1                ; set pmode bit
   mov  cr0, eax
   jmp 0x8:.pmode

.pmode:
   mov  bx, 0x10          ; select descriptor 2
   mov  ds, bx            ; 10h = 10000b

   and al, 0xFE           ; back to realmode
   mov  cr0, eax          ; by toggling bit again
   jmp 0x0:.unreal

.unreal:
   pop ss
   pop ds                 ; get back old segment
   sti

   ret

gdtinfo:
   dw gdt_end - gdt - 1   ;last byte in table
   dd gdt                 ;start of table

gdt:         dd 0,0        ; entry 0 is always unused
flatcode:    db 0xff, 0xff, 0, 0, 0, 10011010b, 10001111b, 0
flatdesc:    db 0xff, 0xff, 0, 0, 0, 10010010b, 11001111b, 0
gdt_end: