use core::arch::global_asm;

global_asm!(include_str!("../../../arch/arm64/boot.S"));
global_asm!(include_str!("../../../arch/arm64/exception.S"));

pub mod cpu;