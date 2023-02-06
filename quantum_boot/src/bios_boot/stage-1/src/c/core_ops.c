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

void memcpy(void *dest, void *src, u32 n) {
    char *csrc = (char*)src;
    char *cdest = (char*)dest;

    for (u32 i = 0; i < n; i++)
        cdest[i] = csrc[i];
}

void* memset(void *dst, char val, int n) {
    char *temp = (char*)dst;
    for(;n != 0; n--) *temp++ = val;
    return dst;
}

uint32 digit_count(uint32 num)
{
    uint32 count = 0;
    if(num == 0)
        return 1;
    while(num > 0){
        count++;
        num = num/10;
    }
    return count;
}

void itoa(u32 num, char *number) {
    u32 dgcount = digit_count(num);
    u32 index = dgcount - 1;
    char x;
    if(num == 0 && dgcount == 1){
        number[0] = '0';
        number[1] = '\0';
    }else{
        while(num != 0){
            x = num % 10;
            number[index] = x + '0';
            index--;
            num = num / 10;
        }
        number[dgcount] = '\0';
    }
}

u32 strlen(const char* str)
{
    uint32 length = 0;
    while(str[length] != '\0')
        length++;
    return length;
}