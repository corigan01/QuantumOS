use arch::registers;
use core::{arch::asm, mem::size_of};

type GDEntry = u64;

#[repr(C)]
struct GlobalDT {
    entries: [GDEntry; 4],
}

impl GlobalDT {
    const fn zeroed() -> Self {
        Self { entries: [0; 4] }
    }

    const fn unreal() -> Self {
        let mut unreal = Self::zeroed();

        // FIXME: Make this easier to understand
        // segment 0x08
        unreal.entries[1] = 0xcf9a000000ffff;
        // segment 0x10
        unreal.entries[2] = 0xcf92000000ffff;

        unreal
    }

    fn package(&'static self) -> GdtPointer {
        GdtPointer {
            size: size_of::<Self>() as u16 - 1,
            ptr: self as *const GlobalDT,
        }
    }
}

#[repr(C, packed(2))]
pub struct GdtPointer {
    size: u16,
    ptr: *const GlobalDT,
}

impl GdtPointer {
    unsafe fn load(self) {
        asm!("
                cli
                lgdt [{ptr}]
            ",
            ptr = in(reg) &self
        );
    }
}

#[link_section = ".GDT"]
static GLOBAL_DESCRIPTOR_TABLE: GlobalDT = GlobalDT::unreal();

pub unsafe fn enter_unreal() {
    GLOBAL_DESCRIPTOR_TABLE.package().load();

    // Set protected mode
    let mut cr0: u32;
    asm!("mov {0:e}, cr0", out(reg) cr0);
    cr0 |= 1;
    asm!("mov cr0, {0:e}", in(reg) cr0);

    // set protected segments
    asm!("
            mov ds, {0:x}
            mov ss, {0:x}
        ",
        in(reg) 0x10
    );

    // unset protected mode
    cr0 &= !1;
    asm!("mov cr0, {0:e}", in(reg) cr0);

    // restore default segments
    asm!("
            mov ds, {0:x}
            mov ss, {0:x}
            sti
        ",
        in(reg) 0x0
    );
}

pub unsafe fn enter_stage2(entry_point: *const u8) {
    arch::interrupts::disable_interrupts();
    cr0::set_protected_mode(true);

    todo!()
}
