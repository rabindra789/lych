// QEMU virt machine GICv3 addresses.
const GICD_BASE: usize = 0x0800_0000;
const GICR_BASE: usize = 0x080A_0000;

// GIC Distributor registers
const GICD_CTLR: usize = 0x000;
const GICD_CTLR_ENABLE_GRP1_NS: u32 = 1 << 1;
const GICD_CTLR_ARE_NS: u32 = 1 << 4;
const GICD_CTLR_RWP: u32 = 1 << 31;

// GIC Redistributor registers
const GICR_WAKER: usize = 0x0014;
const GICR_WAKER_PROCESSOR_SLEEP: u32 = 1 << 1;
const GICR_WAKER_CHILDREN_ASLEEP: u32 = 1 << 2;

// SGI and PPI register frame starts at Redistributor + 0x10000
const GICR_SGI_BASE: usize = 0x1_0000;
const GICR_IGROUPR0: usize = GICR_SGI_BASE + 0x0080;
const GICR_ISENABLER0: usize = GICR_SGI_BASE + 0x0100;
const GICR_ICENABLER0: usize = GICR_SGI_BASE + 0x0180;
const GICR_IPRIORITYR: usize = GICR_SGI_BASE + 0x0400;

// QEMU virt physical timer PPI.
pub const TIMER_IRQ: u32 = 30;

#[inline]
unsafe fn read_volatile(addr: usize) -> u32 {
    unsafe { core::ptr::read_volatile(addr as *const u32) }
}

#[inline]
unsafe fn write_volatile(addr: usize, value: u32) {
    unsafe { core::ptr::write_volatile(addr as *mut u32, value) };
}

#[inline]
unsafe fn write_volatile_u8(addr: usize, value: u8) {
    unsafe { core::ptr::write_volatile(addr as *mut u8, value) };
}

#[inline]
fn wait_for_rwp() {
    unsafe {
        while read_volatile(GICD_BASE + GICD_CTLR) & GICD_CTLR_RWP != 0 {
            core::hint::spin_loop();
        }
    }
}

#[inline]
fn read_gicd_ctlr() -> u32 {
    wait_for_rwp();

    unsafe { read_volatile(GICD_BASE + GICD_CTLR) & !GICD_CTLR_RWP }
}

#[inline]
fn write_gicd_ctlr(value: u32) {
    wait_for_rwp();

    unsafe {
        write_volatile(GICD_BASE + GICD_CTLR, value & !GICD_CTLR_RWP);
    }

    wait_for_rwp();
}

#[inline]
fn enable_system_register_interface() {
    unsafe {
        core::arch::asm!(
            "msr ICC_SRE_EL1, {value}",
            "isb",
            value = in(reg) 1u64,
        );
    }
}

/// Set the priority mask for Group 1 interrupts.
pub fn set_priority_mask(mask: u64) {
    unsafe {
        core::arch::asm!(
            "msr ICC_PMR_EL1, {value}",
            "isb",
            value = in(reg) mask,
        );
    }
}

/// Enable Group 1 physical interrupts at the CPU interface.
pub fn enable_group1() {
    unsafe {
        core::arch::asm!(
            "msr ICC_IGRPEN1_EL1, {value}",
            "isb",
            value = in(reg) 1u64,
        );
    }
}

/// Read the Group 1 interrupt acknowledge register.
pub fn acknowledge_interrupt() -> u32 {
    let value: u64;

    unsafe {
        core::arch::asm!(
            "mrs {value}, ICC_IAR1_EL1",
            value = out(reg) value,
        );
    }

    value as u32
}

/// Signal end of interrupt.
pub fn end_interrupt(intid: u32) {
    unsafe {
        core::arch::asm!(
            "msr ICC_EOIR1_EL1, {value}",
            "isb",
            value = in(reg) intid as u64,
        );
    }
}

/// Enable the physical timer PPI on the current core.
pub fn enable_timer_ppi() {
    unsafe {
        write_volatile(GICR_BASE + GICR_ISENABLER0, 1 << TIMER_IRQ);
    }
}

/// Disable the physical timer PPI on the current core.
pub fn disable_timer_ppi() {
    unsafe {
        write_volatile(GICR_BASE + GICR_ICENABLER0, 1 << TIMER_IRQ);
    }
}

pub fn init() {
    unsafe {
        // Wake the current core's Redistributor.
        let waker = GICR_BASE + GICR_WAKER;
        let mut value = read_volatile(waker);

        value &= !GICR_WAKER_PROCESSOR_SLEEP;
        write_volatile(waker, value);

        // Wait until the Redistributor reports that it is awake.
        while read_volatile(waker) & GICR_WAKER_CHILDREN_ASLEEP != 0 {
            core::hint::spin_loop();
        }

        // Configure the physical timer PPI as Group 1.
        let group = GICR_BASE + GICR_IGROUPR0;
        let value = read_volatile(group) | (1 << TIMER_IRQ);
        write_volatile(group, value);

        // Set the timer interrupt priority.
        let priority = GICR_BASE + GICR_IPRIORITYR + TIMER_IRQ as usize;
        write_volatile_u8(priority, 0x80);

        // Configure the Distributor. Disable Non-secure Group 1 first:
        // ARE_NS may only change from 0 to 1 while EnableGrp1NS is 0.
        let ctlr = read_gicd_ctlr() & !GICD_CTLR_ENABLE_GRP1_NS;
        write_gicd_ctlr(ctlr);

        // Enable affinity routing before enabling Non-secure Group 1.
        let ctlr = read_gicd_ctlr();

        if ctlr & GICD_CTLR_ARE_NS == 0 {
            write_gicd_ctlr(ctlr | GICD_CTLR_ARE_NS);
        }

        // Enable Non-secure Group 1 now that affinity routing is active.
        let ctlr = read_gicd_ctlr();
        write_gicd_ctlr(ctlr | GICD_CTLR_ENABLE_GRP1_NS);

        // Enable the CPU interface only after the Redistributor is awake.
        enable_system_register_interface();
    }
}
