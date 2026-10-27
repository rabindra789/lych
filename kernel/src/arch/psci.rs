
use core::arch::asm;

/// PSCI 64-bit CPU_ON function ID.
const PSCI_CPU_ON_64: u64 = 0xC400_0003;

/// Start a CPU using the HVC conduit advertised by the device tree.
///
/// `target_mpidr` is the target CPU's MPIDR affinity value.
/// `entry_point` is the physical address of its entry point.
/// `context_id` is passed to the entry point in x0.
///
/// Returns zero on success or a negative PSCI error code.
pub fn cpu_on(target_mpidr: u64, entry_point: u64, context_id: u64) -> i64 {
    let result: u64;

    unsafe {
        asm!(
            "hvc #0",
            in("x0") PSCI_CPU_ON_64,
            in("x1") target_mpidr,
            in("x2") entry_point,
            in("x3") context_id,
            lateout("x0") result,
            options(nostack),
        );
    }

    result as i64
}
