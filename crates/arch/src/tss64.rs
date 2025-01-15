/*
  ____                 __               __   _ __
 / __ \__ _____ ____  / /___ ____ _    / /  (_) /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ / _ \
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/_/_.__/
    Part of the Quantum OS Project

Copyright 2025 Gavin Kellam

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

use crate::CpuPrivilege;

#[repr(C)]
pub struct TaskStateSegment {
    reserved0: u32,
    rsp0_lo: u32,
    rsp0_hi: u32,
    rsp1_lo: u32,
    rsp1_hi: u32,
    rsp2_lo: u32,
    rsp2_hi: u32,
    reserved1: u32,
    reserved2: u32,
    ist0_lo: u32,
    ist0_hi: u32,
    ist1_lo: u32,
    ist1_hi: u32,
    ist2_lo: u32,
    ist2_hi: u32,
    ist3_lo: u32,
    ist3_hi: u32,
    ist4_lo: u32,
    ist4_hi: u32,
    ist5_lo: u32,
    ist5_hi: u32,
    ist6_lo: u32,
    ist6_hi: u32,
    ist7_lo: u32,
    ist7_hi: u32,
    reserved3: u32,
    reserved4: u32,
    reserved5: u16,
    iopb: u16,
}

impl TaskStateSegment {
    pub const fn new() -> Self {
        Self {
            reserved0: 0,
            rsp0_lo: 0,
            rsp0_hi: 0,
            rsp1_lo: 0,
            rsp1_hi: 0,
            rsp2_lo: 0,
            rsp2_hi: 0,
            reserved1: 0,
            reserved2: 0,
            ist0_lo: 0,
            ist0_hi: 0,
            ist1_lo: 0,
            ist1_hi: 0,
            ist2_lo: 0,
            ist2_hi: 0,
            ist3_lo: 0,
            ist3_hi: 0,
            ist4_lo: 0,
            ist4_hi: 0,
            ist5_lo: 0,
            ist5_hi: 0,
            ist6_lo: 0,
            ist6_hi: 0,
            ist7_lo: 0,
            ist7_hi: 0,
            reserved3: 0,
            reserved4: 0,
            reserved5: 0,
            iopb: 0,
        }
    }

    pub fn set_stack_for_priv(&mut self, rsp: *mut u8, privl: CpuPrivilege) {
        let addr_lo = (rsp.addr() & 0xFFFFFFFF) as u32;
        let addr_hi = ((rsp.addr() as u64 >> 32) & 0xFFFFFFFF) as u32;

        match privl {
            CpuPrivilege::Ring0 => {
                self.rsp0_lo = addr_lo;
                self.rsp0_hi = addr_hi;
            }
            CpuPrivilege::Ring1 => {
                self.rsp1_lo = addr_lo;
                self.rsp1_hi = addr_hi;
            }
            CpuPrivilege::Ring2 => {
                self.rsp2_lo = addr_lo;
                self.rsp2_hi = addr_hi;
            }
            CpuPrivilege::Ring3 => panic!("Ring3 RSP not supported!"),
        }
    }

    pub fn set_stack_for_ist(&mut self, rsp: *mut u8, ist_id: usize) {
        let addr_lo = (rsp.addr() & 0xFFFFFFFF) as u32;
        let addr_hi = ((rsp.addr() as u64 >> 32) & 0xFFFFFFFF) as u32;

        match ist_id {
            0 => {
                self.ist0_lo = addr_lo;
                self.ist0_hi = addr_hi;
            }
            1 => {
                self.ist1_lo = addr_lo;
                self.ist1_hi = addr_hi;
            }
            2 => {
                self.ist2_lo = addr_lo;
                self.ist2_hi = addr_hi;
            }
            3 => {
                self.ist3_lo = addr_lo;
                self.ist3_hi = addr_hi;
            }
            4 => {
                self.ist4_lo = addr_lo;
                self.ist4_hi = addr_hi;
            }
            5 => {
                self.ist5_lo = addr_lo;
                self.ist5_hi = addr_hi;
            }
            6 => {
                self.ist6_lo = addr_lo;
                self.ist6_hi = addr_hi;
            }
            7 => {
                self.ist7_lo = addr_lo;
                self.ist7_hi = addr_hi;
            }
            _ => panic!("ist id of {ist_id} is not supported!"),
        }
    }
}
