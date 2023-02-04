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

quantum_err db 'Quantum Boot Error', 0, 0
quantum_inr db 'Quantum Booting...', 0, 0

qos_intro:
    mov bl, 0x09 ; set color to blue

    call screen_conf

    mov si, quantum_inr
    call print_string

    ret

print_err:
    mov bl, 0x0C ; set color to red
    call screen_conf

    mov si, quantum_err
    call print_string
    jmp $

screen_conf:
    call set_video_mode40

    ; Move cursor to middle of screen
    mov ah, 0x02
    mov dh, 10    ; ROW
    mov dl, 10    ; COL
    mov bh, 0x00
    int 10h

    ; Draw a bunch of spaces that user defined color
    ; This will make the message a different color!
    mov ah, 09
    mov al, 0x20
    mov cx, 18
    int 10h

    ret

reset_display:
    ; Reset the color back to normal
    mov ah, 09
    mov al, 0x20
    mov bl, 0x00
    mov cx, 18
    int 10h

    call set_video_mode80

    ; Move Cursor to top of screen
    mov ah, 0x02
    mov dh, 0     ; ROW
    mov dl, 0     ; COL
    mov bh, 0x00
    int 10h

    ret


set_video_mode40:
    ; Set video mode to text 40x25
    mov ah, 0x00
    mov al, 0x01
    int 0x10

    ret

set_video_mode80:
   ; Set video mode to text 80x25
   mov ah, 0x00
   mov al, 0x03
   int 0x10

   ret

print_string:
    lodsb

    or al, al
    jz .done

    mov ah, 0x0E
    int 0x10

    jmp print_string

 .done:
    ret