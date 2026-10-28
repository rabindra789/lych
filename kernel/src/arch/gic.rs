// QEMU virt machine GICv3 addresses.
const GICD_BASE: usize = 0x0800_0000;
const GICR_BASE: usize = 0x080A_0000;

const GICR_TYPER: usize = 0x0008;
const GICR_TYPER_VLPIS: u64 = 1 << 1;
const GICR_TYPER_LAST: u64 = 1 << 4;

const GICR_FRAME_STRIDE: usize = 0x2_0000; // 128 KiB
const GICR_FRAME_STRIDE_VLPI: usize = 0x4_0000; // 256 KiB

// Exclusive end of the Redistributor region currently mapped by the MMU.
const GICR_MAPPED_END: usize = 0x08A0_0000;

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
unsafe fn read_volatile64(addr: usize) -> u64 {
    unsafe { core::ptr::read_volatile(addr as *const u64) }
}

/// Convert MPIDR_EL1 affinity fields into GICR_TYPER.Affinity_Value format.
#[inline]
fn gic_affinity_from_mpidr(mpidr: u64) -> u32 {
    (mpidr & 0x00FF_FFFF) as u32 | ((((mpidr >> 32) & 0xFF) as u32) << 24)
}

/// Find the current CPU's GICv3 Redistributor.
///
/// The scan is bounded by the region mapped in memory/mmu.rs.
/// Returns None if no matching Redistributor is found.
pub fn current_redistributor_base() -> Option<usize> {
    let wanted = gic_affinity_from_mpidr(crate::arch::cpu::read_mpidr_el1());
    let mut base = GICR_BASE;

    while base < GICR_MAPPED_END {
        let typer = unsafe { read_volatile64(base + GICR_TYPER) };
        let affinity = (typer >> 32) as u32;

        if affinity == wanted {
            return Some(base);
        }

        if typer & GICR_TYPER_LAST != 0 {
            return None;
        }

        let stride = if typer & GICR_TYPER_VLPIS != 0 {
            GICR_FRAME_STRIDE_VLPI
        } else {
            GICR_FRAME_STRIDE
        };

        base += stride;
    }

    None
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
    let redistributor =
        current_redistributor_base().expect("GIC Redistributor not found for current CPU");

    unsafe {
        write_volatile(redistributor + GICR_ISENABLER0, 1 << TIMER_IRQ);
    }
}

/// Disable the physical timer PPI on the current core.
pub fn disable_timer_ppi() {
    let redistributor =
        current_redistributor_base().expect("GIC Redistributor not found for current CPU");

    unsafe {
        write_volatile(redistributor + GICR_ICENABLER0, 1 << TIMER_IRQ);
    }
}

/// Initialize the shared GIC Distributor.
///
/// Call only on CPU0, once, before starting secondary CPUs.
pub fn init_distributor() {
    // Disable Group 1 Non-secure interrupts before changing
    // the Distributor's affinity-routing configuration.
    let ctlr = read_gicd_ctlr() & !GICD_CTLR_ENABLE_GRP1_NS;
    write_gicd_ctlr(ctlr);
    wait_for_rwp();

    // Enable affinity routing for Non-secure interrupts.
    let ctlr = read_gicd_ctlr();
    if ctlr & GICD_CTLR_ARE_NS == 0 {
        write_gicd_ctlr(ctlr | GICD_CTLR_ARE_NS);
        wait_for_rwp();
    }

    // Re-enable Group 1 Non-secure interrupts.
    let ctlr = read_gicd_ctlr();
    write_gicd_ctlr(ctlr | GICD_CTLR_ENABLE_GRP1_NS);
    wait_for_rwp();
}

/// Initialize the GIC Redistributor and CPU interface for this CPU.
///
/// Returns the discovered Redistributor base, or None if no matching
/// Redistributor exists in the currently mapped region.
pub fn init_redistributor() -> Option<usize> {
    let redistributor = current_redistributor_base()?;

    unsafe {
        // Wake this CPU's Redistributor.
        let waker = redistributor + GICR_WAKER;
        let mut value = read_volatile(waker);
        value &= !GICR_WAKER_PROCESSOR_SLEEP;
        write_volatile(waker, value);

        // Ensure the wake request is visible before polling its status.
        core::arch::asm!("dsb sy", options(nostack, preserves_flags));

        while read_volatile(waker) & GICR_WAKER_CHILDREN_ASLEEP != 0 {
            core::hint::spin_loop();
        }

        // Configure the physical timer PPI as Group 1 Non-secure.
        let group = redistributor + GICR_IGROUPR0;
        let value = read_volatile(group) | (1 << TIMER_IRQ);
        write_volatile(group, value);

        // Set the timer PPI priority.
        let priority = redistributor + GICR_IPRIORITYR + TIMER_IRQ as usize;
        write_volatile_u8(priority, 0x80);

        // Enable the system-register CPU interface on this CPU.
        enable_system_register_interface();
    }

    Some(redistributor)
}
