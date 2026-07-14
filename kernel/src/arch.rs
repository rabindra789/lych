use core::arch::global_asm;

global_asm!(include_str!("../../arch/arm64/boot.S"));