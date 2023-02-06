/*
  ____                 __               __                __
 / __ \__ _____ ____  / /___ ____ _    / /  ___  ___ ____/ /__ ____
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ _ \/ _ `/ _  / -_) __/
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/\___/\_,_/\_,_/\__/_/
  Part of the Quantum OS Project

Copyright 2023 Gavin Kellam

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and
associated documentation files (the "Software"), to deal in the Software without restriction,
including without limitation the rights to use, copy, modify, merge, publish, distribute,
sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial
portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT
NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT
OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

*/

#include "core_ops.h"
#include "rustcall.h"
#include "types.h"

char* vga_address = (char*)0xb8000;


void test() {
    *(vga_address++) = 'q';
}

void cmain(u32 addr) {

    char num_str[32];
    memset(num_str, 0, 32);
    itoa((u32)addr, num_str);
    for (u32 i = 0; i < strlen(num_str); i ++) {
        *(vga_address+=2) = num_str[i];
    }

    u8* our_ptr = (u8*)0x0;

    u32 loader_start = 0;
    u32 loader_end = 0;

    for (u32 i = 0;; i++) {

        if ((our_ptr[i] == 0x21 && our_ptr[i + 1] == 0x73) || (our_ptr[i] == 0x73 && our_ptr[i + 1] == 0x21)) {
            loader_start = i;

        }
        if ((our_ptr[i] == 0xbe && our_ptr[i + 1] == 0xef) || (our_ptr[i] == 0xef && our_ptr[i + 1] == 0xbe)) {
            loader_end = i;

            break;
        }



        if (i >= 0xFFFF) {
            *(vga_address+=2) = 'P';
            *(vga_address+=2) = 'o';
            *(vga_address+=2) = 'o';
            *(vga_address+=2) = 'p';
            break;
        }
    }


    *(vga_address+=2) = 'S';
    *(vga_address+=2) = ':';
    *(vga_address+=2) = ' ';

    memset(num_str, 0, 32);
    itoa((u32)loader_start, num_str);
    for (u32 i = 0; i < strlen(num_str); i ++) {
        *(vga_address+=2) = num_str[i];
    }

    *(vga_address+=2) = ' ';
    *(vga_address+=2) = ' ';

    *(vga_address+=2) = 'E';
    *(vga_address+=2) = ':';
    *(vga_address+=2) = ' ';

    memset(num_str, 0, 32);
    itoa((u32)loader_end, num_str);
    for (u32 i = 0; i < strlen(num_str); i ++) {
        *(vga_address+=2) = num_str[i];
    }

    *(vga_address+=2) = ' ';
    *(vga_address+=2) = ' ';

    u32 loader_size = loader_end - loader_start;
    u32 offset = 0x10000;

    memcpy((void*)(offset + loader_start), (void*)loader_start, loader_size);

    void (*test_ptr)() = &test;
    test_ptr += offset;

    (*test_ptr)();

    switch_to_rust(offset);



    //int32_test();
    while(1);



}

