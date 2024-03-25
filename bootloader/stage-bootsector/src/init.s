# /*
#   ____                 __               __                __
#  / __ \__ _____ ____  / /___ ____ _    / /  ___  ___ ____/ /__ ____
# / /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ _ \/ _ `/ _  / -_) __/
# \___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/\___/\_,_/\_,_/\__/_/
#   Part of the Quantum OS Project
#
# Copyright 2024 Gavin Kellam <corigan01@gmail.com>
#
# Permission is hereby granted, free of charge, to any person obtaining a copy of this software and
# associated documentation files (the "Software"), to deal in the Software without restriction,
# including without limitation the rights to use, copy, modify, merge, publish, distribute,
# sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is
# furnished to do so, subject to the following conditions:
#
# The above copyright notice and this permission notice shall be included in all copies or substantial
# portions of the Software.
#
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT
# NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
# NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
# DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT
# OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
#*/

.section .loader, "awx"
.global bios_entry
.code16

bios_entry:
        # Here we want defined state, so we clear and init registers to known values.
        # We cannot expect the bios to init this for us, because each implmentation of
        # bios is different, sometimes registers will contain 0, other times
        # they will be filled with random data.
        #
        # tldr: Zero registers
        xor ax, ax
        mov ss, ax
        mov ds, ax
        mov fs, ax
        mov gs, ax
        mov es, ax

        cld

        # Setup the stack, this will give use some room to do the rest of the setup
        mov sp, 0x7c00 # Ptr     = 0x7C00

        # Enabling A20 will panic if it fails, so no errors need to be handled here
        call enable_a20

        # Next we enter rust! Its going to be quite tight on space since we only have
        # a few more bytes to play with.
        push dx
        call main


# If we fall through, we want to just call spin, and fail
spin:
        jmp spin

enable_a20:
        # Here we enable to A20 line to access more then 1MB of memory in 16-bit mode
        # We want to load the kernel to upper memory, even when compressed it could
        # take up more then 1MB of memory. So, we need a way of copying the memory
        # out if it cannot fit.

        # Call Bios interrupt 0x15 (8042) if keyboard A20 init is supported
        mov ax, 0x2403
        int 0x15
        jb .fail
        cmp ah, 0x00
        jnz .fail

        # Check if A20 is able to be enabled
        mov ax, 0x2402
        int 0x15
        jb .fail
        cmp ah, 0
        jnz .fail

        # Check if A20 is already enabled by the bios
        cmp al, 1
        jz .return

        # Enable A20 if not already enabled
        mov ax, 0x2401
        int 0x15
        jb .fail
        cmp ah, 0
        jnz .fail
.return:
        ret

# If we fail (i.e Bios interrupt failed) then we put an 'F' and quit. Again its 
# the same story, we don't really have enough bytes to print an entire string.
.fail:
        mov al, 'F'
        call putc
        jmp spin

# Prints A Single Char to the display, useful for debugging
# 
# al: Contains the Char to print
putc:
    mov ah, 0x0e
    int 0x10
    ret
