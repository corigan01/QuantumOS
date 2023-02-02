char* video_memory = (char*) 0xb8000;

void print_str() {
    for (char* i = video_memory; *i != 0; i++) {
        *i = 'X';
    }
}

void cmain() {

    asm ("nop");
    for (int i = 0; i < 100; i++) {
        *(video_memory += 2) = 'X';
    }


    while(1) {};
}
