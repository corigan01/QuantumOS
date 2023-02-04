/*
  ____                 __               __ __                 __
 / __ \__ _____ ____  / /___ ____ _    / //_/__ _______  ___ / /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / ,< / -_) __/ _ \/ -_) /
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /_/|_|\__/_/ /_//_/\__/_/
  Part of the Quantum OS Kernel

Copyright 2022 Gavin Kellam

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

type CpuidVendorId = u16;

#[derive(Copy, Clone, PartialEq, Debug)]
enum CpuidVendor {
    // Real CPUs
    AMD,
    AMDOld,
    Intel,
    Via,
    Transmeta,
    TransmetaOld,
    Cyrix,
    Centaur,
    Nextgen,
    UMC,
    SIS,
    NSC,
    RISE,
    Vortex,
    A0486,
    A0486Old,
    Zhaoxin,
    Hygon,
    Elbrus,

    // Virtual CPUs
    Qemu,
    Kvm,
    VMWare,
    VirtualBox,
    XEN,
    HyperV,
    Parallels,
    ParallelsAlt,
    Bhyve,
    QNX,

    // Unknown CPU
    Unknown
}

impl CpuidVendor {
    const STRING_WITH_VENDOR: [(&'static str, CpuidVendor); 29] = [
        // Real
        ("AuthenticAMD", CpuidVendor::AMD),
        ("AMDisbetter!", CpuidVendor::AMDOld),
        ("GenuineIntel", CpuidVendor::Intel),
        ("VIA VIA VIA ", CpuidVendor::Via),
        ("GenuineTMx86", CpuidVendor::Transmeta),
        ("TransmetaCPU", CpuidVendor::TransmetaOld),
        ("CyrixInstead", CpuidVendor::Cyrix),
        ("CentaurHauls", CpuidVendor::Centaur),
        ("NexGenDriven", CpuidVendor::Nextgen),
        ("UMC UMC UMC ", CpuidVendor::UMC),
        ("SiS SiS SiS ", CpuidVendor::SIS),
        ("Geode by NSC", CpuidVendor::NSC),
        ("RiseRiseRise", CpuidVendor::RISE),
        ("Vortex86 SoC", CpuidVendor::Vortex),
        ("MiSTer AO486", CpuidVendor::A0486),
        ("GenuineAO486", CpuidVendor::A0486Old),
        ("  Shanghai  ", CpuidVendor::Zhaoxin),
        ("HygonGenuine", CpuidVendor::Hygon),
        ("E2K MACHINE ", CpuidVendor::Elbrus),

        // Virtual
        ("TCGTCGTCGTCG", CpuidVendor::Qemu),
        (" KVMKVMKVM  ", CpuidVendor::Kvm),
        ("VMwareVMware", CpuidVendor::VMWare),
        ("VBoxVBoxVBox", CpuidVendor::VirtualBox),
        ("Microsoft Hv", CpuidVendor::HyperV),
        (" prl hyperv ", CpuidVendor::Parallels),
        (" lrpepyh vr ", CpuidVendor::ParallelsAlt),
        ("bhyve bhyve ", CpuidVendor::Bhyve),
        (" QNXQVMBSQG ", CpuidVendor::QNX),

        // Unknown
        ("Unknown CPUv", CpuidVendor::Unknown)
    ];

    fn new() -> Self {
        CpuidVendor::Unknown
    }

    fn convert_from_string(string: &str) -> Self {
       for i in Self::STRING_WITH_VENDOR {
            if string == i.0 {
                return i.1
            }
       }

        CpuidVendor::Unknown
    }

    fn set_from_string(&mut self, string: &str) {
        *self = Self::convert_from_string(string);
    }

    fn get_string(&self) -> &str {
        for i in  Self::STRING_WITH_VENDOR {
            if *self == i.1 {
                return i.0
            }
        }

        // This should never fail, but check anyway to maintain consistency with rust
        Self::STRING_WITH_VENDOR.last().unwrap_or(
            &("Error In Val", CpuidVendor::Unknown)
        ).0
    }
}

#[cfg(test)]
pub mod test_case {
    use crate::arch_x86_64::cpuid::CpuidVendor;

    #[test_case]
    pub fn test_conversion_between_strings() {
        // To enum
        assert_eq!(
            CpuidVendor::convert_from_string("TCGTCGTCGTCG"),
            CpuidVendor::Qemu);

        assert_ne!(
            CpuidVendor::convert_from_string("Shanghai"),
            CpuidVendor::Zhaoxin);

        assert_eq!(
            CpuidVendor::convert_from_string("AuthenticAMD"),
            CpuidVendor::AMD);

        // To String
        assert_eq!(
            "TCGTCGTCGTCG",
            CpuidVendor::Qemu.get_string());

        assert_ne!(
            "Shanghai",
            CpuidVendor::Zhaoxin.get_string());

        assert_eq!(
            "AuthenticAMD",
            CpuidVendor::AMD.get_string());
    }

}
