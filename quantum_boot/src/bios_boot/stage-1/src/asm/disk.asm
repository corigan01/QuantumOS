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


; bx -> destination to put the stage (given to us by mbr)
; dl -> disk (given to us by mbr)
; ah -> function to operate (0x02 is read)
; dh -> the number of sectors to read

perform_read:
    pusha

    push dx
    mov ah, 0x02 ; read mode
    mov al, dh   ; read dh number of sectors
    mov cl, 0x08 ; start from sector 2 (1st sector is us)
    mov ch, 0x00 ; cylinder 0
    mov dh, 0x00 ; head 0

    int 0x13      ; BIOS interrupt
    jc disk_error ; check carry bit for error

    pop dx     ; get back original number of sectors to read
    cmp al, dh ; BIOS sets 'al' to the # of sectors read, so we can compare to see if the read was successful

    jne sectors_error
    popa

    ret

disk_load:
    ; we are given the following:
    ; bx -> destination to load
    ; dl -> disk to be read


    mov dh, [SECTORS]   ; Try and read sectors (512 bytes) and see if the loader's end byte is in it
    call perform_read   ; will return if the read was successful

    jmp .look_for_beef  ; Load until we load the entire file!

    ret

.look_for_beef:
    pusha

    ; Find how many bytes we have to read
    mov  ax, [SECTORS]   ; Get the sectors read
    imul ax, 0x200       ; Multiply by 512 to get bytes
    add  ax, bx          ; add to the offset
    mov  dx, ax

    ; dx now contains the pointer where we should stop

    push bx
    .a:
        ; Check for the magic word
        mov ax, MAGIC_END
        cmp [bx], ax
        je .loaded           ; Jump if we find it

        ; Check if we are at the end of the loaded section
        cmp bx, dx
        je .c

        ; Move to the next byte and check again
        add bx, 1
        jmp .a

    .c:
        ; add one more sector
        mov ax, [SECTORS]
        inc ax
        mov dx, 10
        cmp ax, dx         ; Make sure we dont read too many sectors
        je sectors_error
        mov [SECTORS], ax

        pop bx

        ; Try loading again with (sectors + 1)
        popa
        jmp disk_load

.loaded:
    ; bx should contain the destination for our stage, so we will use that as a reference point
    pop bx

    ; Find how many bytes we have to read
    mov  ax, [SECTORS]   ; Get the sectors read
    imul ax, 0x200       ; Multiply by 512 to get bytes
    add  ax, bx          ; add to the offset
    mov  dx, ax

    .d:
        ; Check for the magic word
        mov ax, MAGIC_START
        cmp [bx], ax
        je .e           ; Jump if we find it

        ; Check if we are at the end of the loaded section
        cmp bx, dx
        je disk_error

        ; Move to the next byte and check again
        add bx, 1
        jmp .d

    .e:
        add bx, 4
        mov [STAGE_ADDRS], bx
        popa
        ret


SECTORS     db 4

MAGIC_START equ 0xdead
MAGIC_END   equ 0xbeef


disk_error:
    call print_err

sectors_error:
    call print_err
