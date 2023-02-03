char* video_memory = (char*) 0xb8000;
int bytes[4] = {0xAA, 0xBB, 0xCC, 0xDD};

void print_str() {
    for (char* i = video_memory; *i != 0; i++) {
        *i = 'X';
    }
}

void cmain() {
    asm ("int $0x10");

    for (int i = 0; i < 100; i++) {
        *(video_memory++) = 'X';
    }
}
