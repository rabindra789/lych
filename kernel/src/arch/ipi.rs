use core::sync::atomic::{AtomicU64, Ordering};

/// Types of inter-processor interrupts supported by Lych.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IpiKind {
    /// Temporary interrupt used to verify CPU-to-CPU delivery.
    Test,

    /// Ask another CPU to reschedule its current task.
    Reschedule,

    /// Notify another CPU that it should process pending work.
    Work,
}

impl IpiKind {
    /// Return the GIC software-generated interrupt ID.
    pub const fn interrupt_id(self) -> u32 {
        match self {
            Self::Test => 1,
            Self::Reschedule => 2,
            Self::Work => 3,
        }
    }

    /// Convert a GIC SGI interrupt ID into an IPI kind.
    pub const fn from_interrupt_id(intid: u32) -> Option<Self> {
        match intid {
            1 => Some(Self::Test),
            2 => Some(Self::Reschedule),
            3 => Some(Self::Work),
            _ => None,
        }
    }
}

/// Send an IPI to a target CPU identified by its MPIDR value.
///
/// The target CPU must have the corresponding SGI enabled.
pub fn send(target_mpidr: u64, kind: IpiKind) {
    crate::arch::gic::send_sgi(target_mpidr, kind.interrupt_id());
}

/// Records whether the CPU1 test IPI was received.
pub static CPU1_TEST_RECEIVED: AtomicU64 = AtomicU64::new(0);
/// records whether CPU1 received a Work IPI.
pub static CPU1_WORK_IPI_RECEIVED: AtomicU64 = AtomicU64::new(0);

/// Handle a received IPI.
pub fn handle(kind: IpiKind) {
    match kind {
        IpiKind::Test => {
            if crate::arch::cpu::cpu_affinity() == 1 {
                CPU1_TEST_RECEIVED.store(1, Ordering::Release);
            }
        }

        IpiKind::Reschedule => {
            // Schedular integration will be added later.
        }

        IpiKind::Work => {
            if crate::arch::cpu::cpu_affinity() == 1 {
                CPU1_WORK_IPI_RECEIVED.store(1, Ordering::Release);
            }
        }
    }
}
