use core::sync::atomic::{AtomicU64, Ordering};

static TICKS: AtomicU64 = AtomicU64::new(0);

const TICK_INTERVAL_MS: u64 = 100;

fn tick_interval() -> u32 {
    let interval = crate::arch::timer::frequency() * TICK_INTERVAL_MS / 1000;

    assert!(
        interval <= u32::MAX as u64,
        "timer interval exceeds CNTP_TVAL_EL0 range"
    );

    interval as u32
}

/// Start the timer tick subsystem.
pub fn init() {
    crate::arch::gic::set_priority_mask(0xff);
    crate::arch::gic::enable_timer_ppi();
    crate::arch::gic::enable_group1();

    crate::arch::timer::set_physical_timer(tick_interval());
    crate::arch::timer::enable_physical_timer();

    crate::arch::cpu::enable_irq();
}

/// Called when the physical timer interrupt arrives.
pub fn on_timer_interrupt() {
    TICKS.fetch_add(1, Ordering::Relaxed);

    crate::arch::timer::set_physical_timer(tick_interval());
}

/// Return the number of timer ticks since initialization.
pub fn ticks() -> u64 {
    TICKS.load(Ordering::Relaxed)
}

/// Stop the timer tick subsystem.
pub fn shutdown() {
    crate::arch::cpu::disable_irq();

    crate::arch::timer::disable_physical_timer();
    crate::arch::gic::disable_timer_ppi();
}
