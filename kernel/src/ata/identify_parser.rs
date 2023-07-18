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

use core::mem;
use qk_alloc::boxed::Box;
use qk_alloc::string::String;
use qk_alloc::vec::Vec;
use quantum_lib::bitset::BitSet;
use crate::ata::identify_parser::SpecificConfig::DiskRequiresSetFeatures;

type Word = u16;
type DoubleWord = u32;
type QuadWord = u64;

/// Based on OpenBSD's **AT Attachment 8 - ATA/ATAPI Command Set** (see pdf for more details)
#[repr(C, packed)]
pub struct RawIdentifyStruct {
    /// General configuration (see 7.12.7.2)
    // F   15 0 = ATA device
    // X 14:8 Retired
    // X  7:6 Obsolete
    // X  5:3 Retired
    // V    2 Incomplete response
    // X    1 Retired
    //      0 Reserved
    pub general_config: Word,
    pub obsolete1: Word,
    /// Specific configuration (see 7.12.7.4)
    pub specific_config: Word,
    pub obsolete2: Word,
    pub retired1: DoubleWord,
    pub obsolete3: Word,
    /// Reserved for CFA (see 7.12.7.8)
    pub reserved_for_cfa1: DoubleWord,
    pub retired2: Word,
    /// Serial number (see 7.12.7.10)
    pub serial_number: [Word; 10],
    pub retired3: DoubleWord,
    pub obsolete4: Word,
    /// Firmware revision (see 7.12.7.13)
    pub firmware_revision: [Word; 4],
    /// Model number (see 7.12.7.14)
    pub model_number: [Word; 20],
    /// logical sectors per drq See 7.12.7.15
    // BF 15:8 80h
    // BF  7:0 00h = Reserved
    //       - 01h-FFh = Maximum number of logical sectors that shall be transferred
    //                   per DRQ data block
    pub logical_sectors_per_drq: Word,
    /// Trusted Computing feature set options (see 7.12.7.16)
    // F   15 Shall be cleared to zero
    // F   14 Shall be set to one
    //   13:1 Reserved for the Trusted Computing Group
    // F    0 Trusted Computing feature set is supported
    pub trusted_computing_features: Word,
    /// Capabilities (see 7.12.7.17)
    //    15:14 Reserved for the IDENTIFY PACKET DEVICE command.
    // BF    13 1 = Standby timer values as specified in this standard are supported.
    //          0 = Standby timer values shall be vendor specific.
    //       12 Reserved for the IDENTIFY PACKET DEVICE command.
    // PF    11 1 = IORDY (see ATA8-APT) supported
    //          0 = IORDY (see ATA8-APT) may be supported
    // PF    10 IORDY (see ATA8-APT) may be disabled
    // BF     9 Shall be set to one (i.e., LBA is supported).
    // PF     8 DMA supported
    //      7:2 Reserved
    // BV   1:0 Long Physical Sector Alignment Error reporting
    pub capabilities1: Word,
    /// Capabilities (see 7.12.7.17)
    // BF   15 Shall be cleared to zero
    // BF   14 Shall be set to one
    //    13:2 Reserved
    // X     1 Obsolete
    // BF    0 1 = There is a minimum Standby time value and it is vendor specific.
    //         0 = There is no minimum Standby timer value.
    pub capabilities2: Word,
    pub obsolete5: DoubleWord,
    /// See 7.12.7.19
    // BV 15:8 Free-fall Control Sensitivity
    //     7:3 Reserved
    // BF   2 the fields reported in word 88 are valid
    // BF   1 the fields reported in words 64..70 are valid
    // X    0 Obsolete
    pub free_fall_control_sensitivity_and_struct_info: Word,
    pub obsolete6: [Word; 5],
    /// See 7.12.7.21
    // BF  15 The BLOCK ERASE EXT command is supported (see 7.36.2)
    // BF  14 The OVERWRITE EXT command is supported (see 7.36.4)
    // BF  13 The CRYPTO SCRAMBLE EXT command is supported (see 7.36.3)
    // BF  12 The Sanitize feature set is supported (see 4.17)
    // BF  11 1 = The commands allowed during a sanitize operation are as specified by
    //           this standard (see 4.17.5)
    //       0 = The commands allowed during a sanitize operation are as specified by
    //           ACS-2
    // BF  10 The SANITIZE ANTIFREEZE LOCK EXT command is supported
    //       (see 7.36.5)
    //      9 Reserved
    // BV   8 Multiple logical sector setting is valid
    // BV 7:0 Current setting for number of logical sectors that shall be transferred per
    //       DRQ data block
    pub ext_capabilities: Word,
    /// Total number of user addressable logical sectors for 28-bit commands (DWord)
    /// (see 7.12.7.22)
    pub user_addressable_logical_sectors_lba28: DoubleWord,
    pub obsolete7: Word,
    /// See 7.12.7.24
    //    15:11 Reserved
    // PV    10 Multiword DMA mode 2 is selected
    // PV     9 Multiword DMA mode 1 is selected
    // PV     8 Multiword DMA mode 0 is selected
    //      7:3 Reserved
    // PF     2 Multiword DMA mode 2 and below are supported
    // PF     1 Multiword DMA mode 1 and below are supported
    // PF     0 Multiword DMA mode 0 is supported
    pub multiword_dma_mode: Word,
    /// See 7.12.7.25
    //    15:2 Reserved
    // PF  1:0 PIO mode 3 and mode 4 supported
    pub supported_pio_mode: Word,
    /// Minimum Multiword DMA transfer cycle time per word (see 7.12.7.26)
    pub min_multiword_dma_transfer_cycle_time_per_word: Word,
    /// Manufacturer’s recommended Multiword DMA transfer cycle time (see 7.12.7.27)
    pub recommended_multiword_dma_transfer_cycle_time: Word,
    /// Minimum PIO transfer cycle time without flow control (see 7.12.7.28)
    pub minimum_pio_transfer_cycle_time_without_flow_control: Word,
    /// Minimum PIO transfer cycle time with IORDY (see ATA8-APT) flow control
    /// (see 7.12.7.29)
    pub minimum_pio_transfer_cycle_time_with_iordy_flow_control: Word,
    /// Additional Supported (see 7.12.7.30)
    // N   15 Reserved for CFA
    // BF  14 Deterministic data in trimmed LBA range(s) is supported
    // BF  13 Long Physical Sector Alignment Error Reporting Control is supported
    // X   12 Obsolete
    // BF  11 READ BUFFER DMA is supported
    // BF  10 WRITE BUFFER DMA is supported
    // X    9 Obsolete
    // BF   8 DOWNLOAD MICROCODE DMA is supported
    //      7 Reserved for IEEE 1667
    // BF   6 0 = Optional ATA device 28-bit commands supported
    // BF   5 Trimmed LBA range(s) returning zeroed data is supported
    // BF   4 Device Encrypts All User Data on the device
    // BF   3 Extended Number of User Addressable Sectors is supported
    // BV   2 All write cache is non-volatile
    //    1:0 Reserved
    pub additional_supported: Word,
    pub reserved1: Word,
    /// Reserved for the IDENTIFY PACKET DEVICE command
    pub reserved_for_identify_packet_device: QuadWord,
    /// Queue depth (see 7.12.7.33)
    //    15:5 Reserved
    // SF  4:0 Maximum queue depth – 1
    pub queue_depth: Word,
    /// Serial ATA Capabilities (see 7.12.7.34)
    // SF  15 Supports READ LOG DMA EXT as equivalent to READ LOG EXT
    // SF  14 Supports Device Automatic Partial to Slumber transitions
    // SF  13 Supports Host Automatic Partial to Slumber transitions
    // SF  12 Supports NCQ priority information
    // SF  11 Supports Unload while NCQ commands are outstanding
    // SF  10 Supports the SATA Phy Event Counters log
    // SF   9 Supports receipt of host initiated power management requests
    // SF   8 Supports the NCQ feature set
    //    7:4 Reserved for Serial ATA
    // SF   3 Supports SATA Gen3 Signaling Speed (6.0Gb/s)
    // SF   2 Supports SATA Gen2 Signaling Speed (3.0Gb/s)
    // SF   1 Supports SATA Gen1 Signaling Speed (1.5Gb/s)
    // SF   0 Shall be cleared to zero
    pub sata_capabilities: Word,
    /// Serial ATA Additional Capabilities (see 7.12.7.35)
    //    15:7 Reserved for Serial ATA
    // SF    6 Supports RECEIVE FPDMA QUEUED and SEND FPDMA QUEUED
    //         commands
    // SF    5 Supports NCQ Queue Management Command
    // SF    4 Supports NCQ Streaming
    // SV  3:1 Coded value indicating current negotiated Serial ATA signal speed
    // SF    0 Shall be cleared to zero
    pub sata_additional_capabilities: Word,
    /// Serial ATA features supported (see 7.12.7.36)
    //    15:8 Reserved for Serial ATA
    // SF    7 Device supports NCQ Autosense
    // SF    6 Device supports Software Settings Preservation
    // SF    5 Device supports Hardware Feature Control
    // SF    4 Device supports in-order data delivery
    // SF    3 Device supports initiating power management
    // SF    2 Device supports DMA Setup auto-activation
    // SF    1 Device supports non-zero buffer offsets
    // SF    0 Shall be cleared to zero
    pub sata_features_supported: Word,
    /// Serial ATA features enabled (see 7.12.7.37)
    //    15:8 Reserved for Serial ATA
    // SV    7 Automatic Partial to Slumber transitions enabled
    // SV    6 Software Settings Preservation enabled
    // SV    5 Hardware Feature Control is enabled
    // SV    4 In-order data delivery enabled
    // SV    3 Device initiated power management enabled
    // SV    2 DMA Setup auto-activation enabled
    // SV    1 Non-zero buffer offsets enabled
    // SF    0 Shall be cleared to zero
    pub sata_features_enabled: Word,
    /// Major version number (see 7.12.7.38)
    //    15:11 Reserved
    // BF    10 supports ACS-3
    // BF     9 supports ACS-2
    // BF     8 supports ATA8-ACS
    // BF     7 supports ATA/ATAPI-7
    // BF     6 supports ATA/ATAPI-6
    // BF     5 supports ATA/ATAPI-5
    // X      4 Obsolete
    // X      3 Obsolete
    // X      2 Obsolete
    // X      1 Obsolete
    //        0 Reserved
    pub major_version_number: Word,
    /// Minor version number (see 7.12.7.39)
    pub minor_version_number: Word,
    /// Commands and feature sets supported (see 7.12.7.40)
    // X     15 Obsolete
    // BF    14 The NOP command is supported
    // BF    13 The READ BUFFER command is supported
    // BF    12 The WRITE BUFFER command is supported
    // X  11:10 Obsolete
    // BF     9 Shall be cleared to zero (i.e., the DEVICE RESET command is not
    //          supported)
    // X    8:7 Obsolete
    // BF     6 Read look-ahead is supported
    // BF     5 The volatile write cache is supported
    // BF     4 Shall be cleared to zero (i.e., the PACKET feature set is not supported)
    // BF     3 Shall be set to one (i.e., the Power Management feature set is supported)
    // X      2 Obsolete
    // BF     1 The Security feature set is supported
    // BF     0 The SMART feature set is supported
    pub commands_and_feature_sets_supported1: Word,
    /// Commands and feature sets supported (see 7.12.7.40)
    // BF  15 Shall be cleared to zero
    // BF  14 Shall be set to one
    // BF  13 The FLUSH CACHE EXT command is supported
    // BF  12 Shall be set to one (i.e., the FLUSH CACHE command is supported)
    // X   11 Obsolete
    // BF  10 The 48-bit Address feature set is supported
    // X  9:8 Obsolete
    // X    7 Obsolete
    // BF   6 SET FEATURES subcommand is required to spin-up after power-up
    // BF   5 The PUIS feature set is supported
    // X    4 Obsolete
    // F    3 The APM feature set is supported
    // N    2 Reserved for CFA
    // X    1 Obsolete
    // F    0 The DOWNLOAD MICROCODE command is supported
    pub commands_and_feature_sets_supported2: Word,
    /// Commands and feature sets supported (see 7.12.7.40)
    // BF    15 Shall be cleared to zero
    // BF    14 Shall be set to one
    // BF    13 The IDLE IMMEDIATE command with UNLOAD feature is supported
    // X  12:11 Obsolete
    // X   10:9 Obsolete
    // BF     8 Shall be set to one (i.e., the World Wide Name is supported)
    // X      7 Obsolete
    // BF     6 The WRITE DMA FUA EXT command and WRITE MULTIPLE FUA EXT
    //          command are supported
    // BF     5 The GPL feature set is supported
    // BF     4 The Streaming feature set is supported
    // X      3 Obsolete
    //        2 Reserved
    // BF     1 The SMART self-test is supported
    // BF     0 SMART error logging is supported
    pub commands_and_feature_sets_supported3: Word,
    /// Commands and feature sets supported or enabled (see 7.12.7.41)
    // X   15 Obsolete
    // BF  14 The NOP command is supported
    // BF  13 The READ BUFFER command is supported
    // BF  12 The WRITE BUFFER command is supported
    // X   11 Obsolete
    // X   10 Obsolete
    // BF   9 Shall be cleared to zero (i.e., the DEVICE RESET command is not
    //        supported)
    // X  8:7 Obsolete
    // BV   6 Read look-ahead is enabled
    // BV   5 The volatile write cache is enabled
    // BF   4 Shall be cleared to zero (i.e., the PACKET feature set is not supported)
    // BF   3 Shall be set to one (i.e., the Power Management feature set is supported)
    // X    2 Obsolete
    // BV   1 The Security feature set is enabled
    // BV   0 The SMART feature set is enabled
    pub commands_and_feature_sets_supported_or_enabled1: Word,
    /// Commands and feature sets supported or enabled (see 7.12.7.41)
    // BF  15 Words 119..120 are valid
    //     14 Reserved
    // BF  13 FLUSH CACHE EXT command supported
    // BF  12 FLUSH CACHE command supported
    // X   11 Obsolete
    // BF  10 The 48-bit Address features set is supported
    // X  9:8 Obsolete
    // X    7 Obsolete
    // BF   6 SET FEATURES subcommand is required to spin-up after power-up
    // BV   5 The PUIS feature set is enabled
    // X    4 Obsolete
    // BV   3 The APM feature set is enabled
    // N    2 Reserved for CFA
    // X    1 Obsolete
    // BF   0 The DOWNLOAD MICROCODE command is supported
    pub commands_and_feature_sets_supported_or_enabled2: Word,
    /// Commands and feature sets supported or enabled(see 7.12.7.41)
    // BF    15 Shall be cleared to zero
    // BF    14 Shall be set to one
    // BF    13 The IDLE IMMEDIATE command with UNLOAD FEATURE is supported
    // X  12:11 Obsolete
    // X   10:9 Obsolete
    // BF     8 Shall be set to one (i.e., the World Wide Name is supported)
    // X      7 Obsolete
    // BF     6 The WRITE DMA FUA EXT command and WRITE MULTIPLE FUA EXT
    //          command are supported
    // BF     5 The GPL feature set is supported
    // X      4 Obsolete
    // X      3 Obsolete
    // BV     2 Media serial number is valid
    // BF     1 SMART self-test supported
    // BF     0 SMART error logging is supported
    pub commands_and_feature_sets_supported_or_enabled3: Word,
    /// Ultra DMA modes (see 7.12.7.42)
    //    15 Reserved
    // PV 14 Ultra DMA mode 6 is selected.
    // PV 13 Ultra DMA mode 5 is selected.
    // PV 12 Ultra DMA mode 4 is selected.
    // PV 11 Ultra DMA mode 3 is selected.
    // PV 10 Ultra DMA mode 2 is selected.
    // PV  9 Ultra DMA mode 1 is selected.
    // PV  8 Ultra DMA mode 0 is selected.
    //     7 Reserved
    // PF  6 Ultra DMA mode 6 and below are supported
    // PF  5 Ultra DMA mode 5 and below are supported
    // PF  4 Ultra DMA mode 4 and below are supported
    // PF  3 Ultra DMA mode 3 and below are supported
    // PF  2 Ultra DMA mode 2 and below are supported
    // PF  1 Ultra DMA mode 1 and below are supported
    // PF  0 Ultra DMA mode 0 is supported
    pub ultra_dma_modes: Word,
    /// See 7.12.7.43
    //    15 1=Extended Time is reported in bits 14:0
    //       0=Time is reported in bits 7:0
    // **If bit 15 is set to one**
    //    14:0 Extended Time required for Normal Erase mode SECURITY ERASE UNIT
    //         command
    // **If bit 15 is set to zero**
    //    14:8 Reserved
    //     7:0 Time required for Normal Erase mode SECURITY ERASE UNIT command
    pub time_erase_config: Word,
    /// See 7.12.7.44
    //    15 1=Extended Time is reported in bits 14:0
    //       0=Time is reported in bits 7:0
    // **If bit 15 is set to one**
    //    14:0 Extended Time required for Normal Erase mode SECURITY ERASE UNIT
    //         command
    // **If bit 15 is set to zero**
    //    14:8 Reserved
    //     7:0 Time required for Normal Erase mode SECURITY ERASE UNIT command
    pub time_erase_config_copy: Word,
    /// APM level
    //     15:8 Reserved
    // OBV  7:0 Current APM level value (see 7.12.7.45)
    pub apm_level_value: Word,
    /// Master Password Identifier (see 7.12.7.46)
    pub master_password_identifier: Word,
    /// Hardware reset results (see 7.12.7.47)
    // ** For SATA devices, word 93 shall be set to the value 0000h. **
    // BF   15 Shall be cleared to zero.
    // BF   14 Shall be set to one for PATA devices.
    // PV   13 1 = device detected the CBLID- above ViHB (see ATA8-APT)
    //         0 = device detected the CBLID- below ViL (see ATA8-APT)
    // P  12:8 Device 1 hardware reset result. Device 0 shall clear these bits to zero.
    //         Device 1 shall set these bits as follows:
    //              12 Reserved
    // V            11 Device 1 asserted PDIAG-.
    // V          10:9 These bits indicate how Device 1 determined the device number:
    //                   00 = Reserved
    //                   01 = a jumper was used.
    //                   10 = the CSEL signal was used.
    //                   11 = some other method was used or the method is unknown.
    // F             8 Shall be set to one.
    // P   7:0 Device 0 hardware reset result. Device 1 shall clear these bits to zero.
    //         Device 0 shall set these bits as follows:
    //               7 Reserved
    // F             6 Device 0 responds when Device 1 is selected.
    // V             5 Device 0 detected the assertion of DASP-.
    // V             4 Device 0 detected the assertion of PDIAG-.
    // V             3 Device 0 passed diagnostics.
    // V           2:1 These bits indicate how Device 0 determined the device number:
    //                   00 = Reserved
    //                   01 = a jumper was used.
    //                   10 = the CSEL signal was used.
    //                   11 = some other method was used or the method is unknown.
    // F     0 Shall be set to one for PATA devices.
    pub hardware_reset_results: Word,
    pub obsolete8: Word,
    /// Stream Minimum Request Size (see 7.12.7.49)
    pub stream_minimum_request_size: Word,
    /// Streaming Transfer Time – DMA (see 7.12.7.50)
    pub stream_transfer_time_dma: Word,
    /// Streaming Access Latency – DMA and PIO (see 7.12.7.51)
    pub stream_access_latency: Word,
    /// Streaming Performance Granularity (DWord) (see 7.12.7.52)
    pub streaming_performance_granularity: DoubleWord,
    /// Number of User Addressable Logical Sectors (QWord) (see 7.12.7.53)
    pub user_addressable_logical_sectors: QuadWord,
    /// Streaming Transfer Time – PIO (see 7.12.7.54)
    pub streaming_transfer_time_pio: Word,
    /// Maximum number of 512-byte blocks per DATA SET MANAGEMENT command
    /// (see 7.5)
    pub max_512_byte_blocks_per_data_set_management_command: Word,
    /// Physical sector size / logical sector size (see 7.12.7.56)
    // BF   15 Shall be cleared to zero
    // BF   14 Shall be set to one
    // BF   13 Device has multiple logical sectors per physical sector.
    // BF   12 Device Logical Sector longer than 256 words
    //    11:4 Reserved
    // BF  3:0 2^X logical sectors per physical sector
    pub physical_or_logical_sector_size: Word,
    /// Inter-seek delay for ISO/IEC 7779 standard acoustic testing (see 7.12.7.57)
    pub inner_seek_delay: Word,
    /// World wide name (see 7.12.7.58)
    pub world_wide_name: QuadWord,
    pub reserved2: QuadWord,
    pub obsolete9: Word,
    /// Logical sector size (DWord) (see 7.12.7.61)
    pub logical_sector_size: DoubleWord,
    /// Commands and feature sets supported (Continued from words 82..84) (see 7.12.7.40)
    // BF    15 Shall be cleared to zero
    // BF    14 Shall be set to one
    //    13:10 Reserved
    // BF     9 DSN feature set is supported
    // SF     8 Accessible Max Address Configuration feature set is supported
    // SF     7 EPC feature set is supported
    // BF     6 Sense Data Reporting feature set is supported
    // BF     5 The Free-fall Control feature set is supported
    // BF     4 Download Microcode mode 3 is supported
    // BF     3 The READ LOG DMA EXT command and WRITE LOG DMA EXT
    //          command are supported
    // BF     2 The WRITE UNCORRECTABLE EXT command is supported
    // BF     1 The Write-Read-Verify feature set is supported
    // X      0 Obsolete
    pub commands_and_feature_sets_supported4: Word,
    /// Commands and feature sets supported or enabled (Continued from words 85..87)
    /// (see 7.12.7.41)
    // BF    15 Shall be cleared to zero
    // BF    14 Shall be set to one
    //    13:10 Reserved
    // BV     9 DSN feature set is enabled
    //        8 Reserved
    // BV     7 EPC feature set is enabled
    // BV     6 Sense Data Reporting feature set is enabled
    // BV     5 The Free-fall Control feature set is enabled
    // BF     4 Download Microcode mode 3 is supported
    // BF     3 The READ LOG DMA EXT command and WRITE LOG DMA EXT
    //          command are supported
    // BF     2 The WRITE UNCORRECTABLE EXT command is supported
    // BV     1 The Write-Read-Verify feature set is enabled
    // X      0 Obsolete
    pub commands_and_feature_sets_supported_or_enabled4: Word,
    /// Reserved for expanded supported and enabled settings
    pub reserved_for_expanded_supported_and_enabled_settings: [Word; 6],
    pub obsolete10: Word,
    /// Security status (see 7.12.7.66)
    //    15:9 Reserved
    // BV    8 Master Password Capability: 0 = High, 1 = Maximum
    //     7:6 Reserved
    // BF    5 Enhanced security erase supported
    // BV    4 Security count expired
    // BV    3 Security frozen
    // BV    2 Security locked
    // BV    1 Security enabled
    // BF    0 Security supported
    pub security_status: Word,
    pub vendor_specific: [Word; 31],
    /// Reserved for CFA (see 7.12.7.68)
    pub reserved_for_cfa2: [Word; 8],
    /// See 7.12.7.69
    //     15:4 Reserved
    // OBF  3:0 Device Nominal Form Factor
    pub device_nominal_form_factor: Word,
    /// DATA SET MANAGEMENT command support (see 7.12.7.70)
    //     15:1 Reserved
    //        0 the TRIM bit in the DATA SET MANAGEMENT command is supported
    pub data_set_management_command_support: Word,
    /// Additional Product Identifier (see 7.12.7.71)
    pub additional_product_identifier: QuadWord,
    pub reserved3: DoubleWord,
    /// Current media serial number (see 7.12.7.73)
    pub current_media_serial_number: [Word; 30],
    /// SCT Command Transport (see 7.12.7.74)
    // X  15:12 Vendor Specific
    //     11:8 Reserved
    //        7 Reserved for Serial ATA
    //        6 Reserved
    // F      5 The SCT Data Tables command is supported
    // F      4 The SCT Feature Control command is supported
    // F      3 The SCT Error Recovery Control command is supported
    // F      2 The SCT Write Same command is supported
    // X      1 Obsolete
    // F      0 The SCT Command Transport is supported
    pub sct_command_transport: Word,
    pub reserved4: DoubleWord,
    /// Alignment of logical sectors within a physical sector (see 7.12.7.75)
    //       15 Shall be cleared to zero
    //       14 Shall be set to one
    //     13:0 Logical sector offset within the first physical sector where the first logical
    //          sector is placed
    pub alignment_of_logical_sectors_with_physical: Word,
    /// Write-Read-Verify Sector Mode 3 Count (DWord) (see 7.12.7.76)
    pub write_read_verify_sector_mode_3_count: DoubleWord,
    /// Write-Read-Verify Sector Mode 2 Count (DWord) (see 7.12.7.77)
    pub write_read_verify_sector_mode_2_count: DoubleWord,
    pub obsolete11: [Word; 3],
    /// Nominal media rotation rate (see 7.12.7.79)
    pub nominal_media_rotation_rate: Word,
    pub reserved5: Word,
    pub obsolete12: Word,
    /// See 7.12.7.82
    //    15:8 Reserved
    // BV  7:0 Write-Read-Verify feature set current mode
    pub write_read_verify_feature_set_current_mode: Word,
    pub reserved6: Word,
    /// Transport major version number (see 7.12.7.84)
    // ** 0000h or FFFFh = device does not report version **
    // F  15:12 Transport Type
    //          0h = Parallel
    //          1h = Serial
    //       2h-Fh = Reserved
    //
    //           Parallel    |    Serial
    // ----------------------+----------------------
    //     11:7  Reserved    |  Reserved
    // F      6  Reserved    |  SATA 3.1
    // F      5  Reserved    |  SATA 3.0
    // F      4  Reserved    |  SATA 2.6
    // F      3  Reserved    |  SATA 2.5
    // F      2  Reserved    |  SATA II: Extensions
    // F      1  ATA/ATAPI-7 |  SATA 1.0a
    // F      0  ATA8-APT    |  ATA8-AST
    pub transport_major_version_number: Word,
    /// Transport minor version number (see 7.12.7.85)
    pub transport_minor_version_number: Word,
    pub reserved7: [Word; 6],
    /// Extended Number of User Addressable Sectors (QWord) (see 7.12.7.87)
    pub ext_user_addressable_sectors: QuadWord,
    /// Minimum number of 512-byte data blocks per Download Microcode operation
    /// (see 7.12.7.88)
    pub minimum_512_byte_data_blocks_per_download_microcode_operation: Word,
    /// Maximum number of 512-byte data blocks per Download Microcode operation
    /// (see 7.12.7.89)
    pub maximum_512_byte_data_blocks_per_download_microcode_operation: Word,
    pub reserved8: [Word; 19],
    /// Integrity word (see 7.12.7.91)
    //     15:8 Checksum
    //      7:0 Checksum Validity Indicator
    pub integrity_word: Word
}

const _: () = assert!(mem::size_of::<RawIdentifyStruct>() == 512, "RawIdentifyStruct should be 512 bytes!");

impl RawIdentifyStruct {

    pub fn new() -> Self {
        // This is safe because the entire struct is primitive integers,
        // so their 'zero' *is* 0.
        unsafe { mem::zeroed() }
    }

    pub fn from_vec(vec: Vec<u16>) -> Box<Self> {
        Box::new( unsafe { (vec.as_ptr() as *const Self).read() })
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Interconnect {
    Parallel,
    Serial,
    Unknown(u16)
}

#[derive(Clone, Copy, Debug)]
pub enum SpecificConfig {
    DiskRequiresSetFeatures(CompletionStatus),
    DiskDoesNotRequireSetFeatures(CompletionStatus)
}

#[derive(Clone, Copy, Debug)]
pub enum CompletionStatus {
    Complete,
    Incomplete
}

pub struct IdentifyParser {
    raw: Box<RawIdentifyStruct>
}

impl IdentifyParser {
    pub fn new(raw_data: Vec<u16>) -> Self {
        Self {
            raw: RawIdentifyStruct::from_vec(raw_data)
        }
    }

    pub fn identify_completion_status(&self) -> CompletionStatus {
        let raw_value = self.raw.general_config;

        match raw_value.get_bit(2) {
            true => CompletionStatus::Incomplete,
            false => CompletionStatus::Complete
        }
    }

    pub fn specific_config(&self) -> Option<SpecificConfig> {
        let raw_value = self.raw.specific_config;

        use SpecificConfig::*;
        use CompletionStatus::*;

        match raw_value {
            0x37C8 => Some(DiskRequiresSetFeatures(Incomplete)),
            0x738C => Some(DiskRequiresSetFeatures(Complete)),
            0x8C73 => Some(DiskDoesNotRequireSetFeatures(Incomplete)),
            0xC837 => Some(DiskDoesNotRequireSetFeatures(Complete)),

            _ => None
        }
    }

    pub fn model_number(&self) -> String {
        let mut string = String::new();
        let model_number = self.raw.model_number;
        'outer: for word in model_number {
            let bytes = word.to_be_bytes();

            for byte in bytes {
                if byte == 0 || byte == 16 {
                    break 'outer;
                }

                if !byte.is_ascii() {
                    continue;
                }

                string.push(byte as char);
            }
        }

        String::from(string.trim())
    }

    pub fn interconnect(&self) -> Interconnect {
        let raw_word = self.raw.transport_major_version_number;

        if raw_word == 0 || raw_word == Word::MAX {
            return Interconnect::Unknown(raw_word);
        }

        let interconnect_value = raw_word.get_bits(12..16);

        match interconnect_value {
            0 => Interconnect::Parallel,
            1 => Interconnect::Serial,

            _ => Interconnect::Unknown(raw_word)
        }
    }

    pub fn is_48_bit_addressing_supported(&self) -> bool {
        let feature_opt_copy = self.raw.commands_and_feature_sets_supported_or_enabled2;

        feature_opt_copy.get_bit(10)
    }

}