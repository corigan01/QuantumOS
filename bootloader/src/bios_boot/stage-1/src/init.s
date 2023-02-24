.section .boot, "awx"
.global begin
.code16

# This is the pre-init for the pre-init stage, this is where the bios will first call our bootloader!
# At this point we know nothing about the hardware other then our pre-compiled definitions, we will
# need to find out everything we can about the system before we boot the kernel, and even the next
# stage.
#
#
# This loader section is going to load kinda weird as it will be cutoff from the rust segments as
# the bios will only load the first sector into memory, so its our job to load the next few sectors
# into memory to make sure that rust is able to be called. Its kinda like a zombie that is cut in
# half, all it knows how to do is load a few extra sectors into memory :)

begin:
    # Clear segment registers, and set them all to 0, they should be 0 when we begin. It is considered
    # good practice to zero these registers in bootloaders because you never know, and it could cause
    # undefined behavior on some motherboards when we enter rust.
    xor ax, ax
    mov ss, ax
    mov ds, ax
    mov fs, ax
    mov gs, ax
    mov es, ax

    call init_a20

    # Make sure we are going forward in memory for things like the stack
    cld

    # Point the stack here at 0x7c00, this is right before the bootloader is loaded into memory
    # This gives us memory regions between 0x7c00 and 0x0500 for our stack. This will be plenty
    # for our simple 16 bit rust stage-1.
    mov sp, 0x7c00

    # This will load rust into memory, this way we can keep the bootloader section small and just load
    # it into memory. This allows us to do way more in one binary then we would otherwise be able to.
    call load_legs

    # Finally call rust! :)
    push dx
    call bit16_entry

spin:
    jmp spin

init_a20:
    # Test if the gate is supported
    mov ax, 0x2403
    int 0x15
    jb .not_supported
    cmp ah, 0
    jnz .not_supported

    # Check the gate status of A20
    mov ax, 0x2402
    int 0x15
    jb .activation_failed
    cmp ah, 0
    jnz .activation_failed

    # Check if already enabled
    cmp al, 1
    jz .successfully_activated

    # Enable A20
    mov ax, 0x2401
    int 0x15
    jb .activation_failed
    cmp ah, 0
    jnz .activation_failed

.successfully_activated:
    ret

# FIXME: Tell the user this is unsupported!
.activation_failed:
    mov al, 0x25
    call print_err
    jmp spin
.not_supported:
    mov al, 0x27
    call print_err
    jmp spin

load_legs:
    # We are unsure if the 13h extension is enabled here
    # FIXME: Check if DAP is supported (for now this works fine)
    pusha

    mov si, 0x7c80
    mov ah, 0x42

    int 0x13                            # BIOS interrupt
    jc disk_error                       # check carry bit for error

    popa

    ret


# we couldn't read the rest of the bootloader
# TODO: Add a message so the user can see if this fails
disk_error:
    mov al, 0x23
    call print_err
    jmp spin

print_err:
    mov ah, 0x0e
    int 0x10

    ret

# This is how many sectors we need to read, as before I computed the number of sectors by continuously
# reading until I reached the stop byte, however this isn't a great solution as pre-compiling is much
# simpler and works just as well.
.align 16
DATAPACKET:
    .byte    0x10                           # Size of packet
    .byte    0x00                           # Always 0
    .byte    _stage_1_end_sectors           # Sectors to read
    .byte    0x00                           # Always 0
    .2byte   0x0000                         # Load address
    .2byte   0x07e0                         # Load segment
    .4byte   0x0001                         # Starting LBA
    .4byte   0x0000
    .4byte   0x0000