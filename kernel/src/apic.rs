/*
  ____                 __               __ __                 __
 / __ \__ _____ ____  / /___ ____ _    / //_/__ _______  ___ / /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / ,< / -_) __/ _ \/ -_) /
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /_/|_|\__/_/ /_//_/\__/_/
  Part of the Quantum OS Kernel

Copyright 2024 Gavin Kellam

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

// Address   Register Name                                                                                       Software Read/Write
// FEE0 0020 Local APIC ID Register                                                                                   Read/Write.
// FEE0 0030 Local APIC Version Register                                                                              Read Only.
// FEE0 0080 Task Priority Register (TPR)                                                                             Read/Write.
// FEE0 0090 Arbitration Priority Register1 (APR)                                                                     Read Only.
// FEE0 00A0 Processor Priority Register (PPR)                                                                        Read Only.
// FEE0 00B0 EOI Register                                                                                             Write Only.
// FEE0 00C0 Remote Read Register1 (RRD)                                                                              Read Only
// FEE0 00D0 Logical Destination Register                                                                             Read/Write.
// FEE0 00E0 Destination Format Register                                                                              Read/Write (see Section 10.6.2.2)
// FEE0 00F0 Spurious Interrupt Vector Register                                                                       Read/Write (see Section 10.9.
// FEE0 0100 In-Service Register (ISR); bits 31:0                                                                     Read Only.
// FEE0 0110 In-Service Register (ISR); bits 63:32                                                                    Read Only.
// FEE0 0120 In-Service Register (ISR); bits 95:64                                                                    Read Only.
// FEE0 0130 In-Service Register (ISR); bits 127:96                                                                   Read Only.
// FEE0 0140 In-Service Register (ISR); bits 159:128                                                                  Read Only.
// FEE0 0150 In-Service Register (ISR); bits 191:160                                                                  Read Only.
// FEE0 0160 In-Service Register (ISR); bits 223:192                                                                  Read Only.
// FEE0 0170 In-Service Register (ISR); bits 255:224                                                                  Read Only.
// FEE0 0180 Trigger Mode Register (TMR); bits 31:0                                                                   Read Only.
// FEE0 0190 Trigger Mode Register (TMR); bits 63:32                                                                  Read Only.
// FEE0 01A0 Trigger Mode Register (TMR); bits 95:64                                                                  Read Only.
// FEE0 01B0 Trigger Mode Register (TMR); bits 127:96                                                                 Read Only.
// FEE0 01C0 Trigger Mode Register (TMR); bits 159:128                                                                Read Only.
// FEE0 01D0 Trigger Mode Register (TMR); bits 191:160                                                                Read Only.
// FEE0 01E0 Trigger Mode Register (TMR); bits 223:192                                                                Read Only.
// FEE0 01F0 Trigger Mode Register (TMR); bits 255:224                                                                Read Only.
// FEE0 0200 Interrupt Request Register (IRR); bits 31:0                                                              Read Only.
// FEE0 0210 Interrupt Request Register (IRR); bits 63:32                                                             Read Only.
// FEE0 0220 Interrupt Request Register (IRR); bits 95:64                                                             Read Only.
// FEE0 0230 Interrupt Request Register (IRR); bits 127:96                                                            Read Only.
// FEE0 0240 Interrupt Request Register (IRR); bits 159:128                                                           Read Only.
// FEE0 0250 Interrupt Request Register (IRR); bits 191:160                                                           Read Only.
// FEE0 0260 Interrupt Request Register (IRR); bits 223:192                                                           Read Only.
// FEE0 0270 Interrupt Request Register (IRR); bits 255:224                                                           Read Only.
// FEE0 0280 Error Status Register                                                                                    Read Only.
// FEE0 02F0 LVT Corrected Machine Check Interrupt (CMCI) Register                                                    Read/Write.
// FEE0 0300 Interrupt Command Register (ICR); bits 0-31                                                              Read/Write.
// FEE0 0310 Interrupt Command Register (ICR); bits 32-63                                                             Read/Write.
// FEE0 0320 LVT Timer Register                                                                                       Read/Write.
// FEE0 0330 LVT Thermal Sensor Register2                                                                             Read/Write.
// FEE0 0340 LVT Performance Monitoring Counters Register3                                                            Read/Write.
// FEE0 0350 LVT LINT0 Register                                                                                       Read/Write.
// FEE0 0360 LVT LINT1 Register                                                                                       Read/Write.
// FEE0 0370 LVT Error Register                                                                                       Read/Write.
// FEE0 0380 Initial Count Register (for Timer)                                                                       Read/Write.
// FEE0 0390 Current Count Register (for Timer)                                                                       Read Only.
// FEE0 03E0 Divide Configuration Register (for Timer)                                                                Read/Write.
