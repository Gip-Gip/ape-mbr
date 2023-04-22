//! Types of partitions that are detectable through the SystemID field.
//!
//! Derived from the following:
//!  * [http://www.osdever.net/documents/partitiontypes.php](http://www.osdever.net/documents/partitiontypes.php)
//!  * fdisk utility
//!  
//! Most of these are likely to never be used(eg. NovellNetware286), but shall be implemented for implementation's sake

use num_enum::TryFromPrimitive;

#[derive(Debug, Default, TryFromPrimitive, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
#[non_exhaustive]
pub enum PartitionType {
    #[default]
    Unknown = 0x00,
    Fat12 = 0x01,
    XenixRoot = 0x02,
    XenixUsr = 0x03,
    Fat16Lt32 = 0x04,
    Extended = 0x05,
    Fat16 = 0x06,
    Ntfs = 0x07,
    Aix = 0x08,
    AixBootable = 0x09,
    Os2BootManag = 0x0a,
    W95Fat32 = 0x0b,
    W95Fat32Lba = 0x0c,
    W95Fat16Lba = 0x0e,
    W95ExtendedLba = 0x0f,
    Opus = 0x10,
    HiddenFat12 = 0x11,
    CompaqDiagnostic = 0x12,
    HiddenFat16Lt32 = 0x14,
    HiddenFat16 = 0x16,
    HiddenNtfs = 0x17,
    AstSmartsleep = 0x18,
    HiddenW95Fat32 = 0x1b,
    HiddenW95Fat32Lba = 0x1c,
    HiddenW95Fat16Lba = 0x1e,
    NecDos = 0x24,
    HiddenWinNtfs = 0x27,
    Plan9 = 0x39,
    PartitionMagic = 0x3c,
    Venix80286 = 0x40,
    PpcPrepBoot = 0x41,
    Sfs = 0x42,
    Qnx4X = 0x4d,
    Qnx4XPart2 = 0x4e,
    Qnx4XPart3 = 0x4f,
    OntrackDm = 0x50,
    OntrackDm6Aux1 = 0x51,
    Cpm = 0x52,
    OntrackDm6Aux3 = 0x53,
    OntrackDm6Ddo = 0x54,
    EzDrive = 0x55,
    GoldenBow = 0x56,
    PriamEdisk = 0x5c,
    SpeedStor = 0x61,
    GnuHurd = 0x63,
    NovellNetware286 = 0x64,
    NovellNetware386 = 0x65,
    NovellSMS = 0x66,
    NovellNetware5P = 0x69,
    DiskSecureMultiBoot = 0x70,
    Scramdisk = 0x74,
    PcIx = 0x75,
    OldMinix = 0x80,
    Minix = 0x81,
    LinuxSwap = 0x82,
    Linux = 0x83,
    Os2HiddenCDrive = 0x84,
    LinuxExtended = 0x85,
    NftsVolumeSet1 = 0x86,
    NftsVolumeSet2 = 0x87,
    LinuxKernelPartition = 0x8a,
    LegacyFaultTolerantFat32 = 0x8b,
    LegacyFaultTolerantFat32Int13h = 0x8c,
    LinuxLvm = 0x8e,
    Amoeba = 0x93,
    AmoebaBbt = 0x94,
    BsdOs = 0x9f,
    ThinkpadHibernation = 0xa0,
    FreeBSD = 0xa5,
    OpenBSD = 0xa6,
    NeXTSTEP = 0xa7,
    DarwinUFS = 0xa8,
    NetBSD = 0xa9,
    DarwinBoot = 0xab,
    HFS = 0xaf,
    BsdiFs = 0xb7,
    BsdiSwap = 0xb8,
    BootWizardHidden = 0xbb,
    AcronisFat32 = 0xbc,
    Solaris8Boot = 0xbe,
    Solaris = 0xbf,
    DrDosSecFat12 = 0xc1,
    DrDosSecFat16Lt32 = 0xc2,
    DrDosSecExt = 0xc5,
    DrDosSecFat16 = 0xc6,
    Syrinx = 0xc7,
    NoFsData = 0xda,
    CpmCtos = 0xdb,
    DellUti = 0xde,
    BootIt = 0xdf,
    DosAccess = 0xe1,
    DosRo = 0xe3,
    SpeedStor2 = 0xe4,
    LinuxExtended2 = 0xea,
    BeOsFs = 0xeb,
    GPT = 0xee,
    EFI = 0xef,
    LinuxPaRiscBl = 0xf0,
    SpeedStor3 = 0xf1,
    SpeedStor4 = 0xf4,
    DosSecondary = 0xf2,
    EbbrProtective = 0xf8,
    VMWareVmfs = 0xfb,
    VMWareVmkcore = 0xfc,
    LinuxRaidAuto = 0xfd,
    LanStep = 0xfe,
    Bbt = 0xff,
}
