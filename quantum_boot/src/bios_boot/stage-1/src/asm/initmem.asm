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

init_a20:
    ; Test if the gate is supported
    mov ax, 0x2403
    int 0x15
    jb .not_supported
    cmp ah, 0
    jnz .not_supported

    ; Check the gate status of A20
    mov ax, 0x2402
    int 0x15
    jb .activation_failed
    cmp ah, 0
    jnz .activation_failed

    ; Check if already enabled
    cmp al, 1
    jz .successfully_activated

    ; Enable A20
    mov ax, 0x2401
    int 0x15
    jb .activation_failed
    cmp ah, 0
    jnz .activation_failed

.successfully_activated:
    ret

.activation_failed:
    call print_err
.not_supported:
    call print_err


