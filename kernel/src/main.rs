#![no_std]
#![no_main]

extern crate alloc;

mod arch;
mod drivers;
mod memory;
mod platform;

use core::panic::PanicInfo;
use drivers::uart;

use crate::arch::exception::ExceptionFrame;
use core::sync::atomic::{AtomicU64, Ordering};

static SECONDARY_ONLINE: AtomicU64 = AtomicU64::new(0);
static CPU0_WORK_DONE: AtomicU64 = AtomicU64::new(0);
static CPU1_WORK_DONE: AtomicU64 = AtomicU64::new(0);

const SMP_WORK_ITERATIONS: u64 = 1_000_000;

static CPU0_READY: AtomicU64 = AtomicU64::new(0);
static CPU1_READY: AtomicU64 = AtomicU64::new(0);
static SMP_START: AtomicU64 = AtomicU64::new(0);

fn run_smp_workload(seed: u64) -> u64 {
    let mut value = seed;

    for i in 0..SMP_WORK_ITERATIONS {
        value = value.wrapping_mul(6364136223846793005).wrapping_add(i | 1);
        core::hint::spin_loop();
    }

    value
}

/// First Rust func to executed by the kernel
#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
    uart::puts("Lych kernel\n");
    uart::puts("Booting...\n");
    let el = arch::cpu::current_el();

    uart::puts("Current EL: ");
    uart::put_hex(el);
    uart::putc(b'\n');

    arch::cpu::init();

    let mpidr = arch::cpu::read_mpidr_el1();
    let affinity = arch::cpu::cpu_affinity();

    uart::puts("MPIDR_EL1: ");
    uart::put_hex(mpidr);
    uart::putc(b'\n');

    uart::puts("CPU affinity: ");
    uart::put_hex(affinity);
    uart::putc(b'\n');

    let timer_frequency = arch::timer::frequency();
    let start = arch::timer::counter();

    for _ in 0..100_000 {
        core::hint::spin_loop();
    }

    let end = arch::timer::counter();

    uart::puts("Timer frequency: ");
    uart::put_hex(timer_frequency);
    uart::putc(b'\n');

    uart::puts("Timer start: ");
    uart::put_hex(start);
    uart::putc(b'\n');

    uart::puts("Timer end: ");
    uart::put_hex(end);
    uart::putc(b'\n');

    assert!(timer_frequency > 0);
    assert!(end > start);

    uart::puts("Initializing physical memory...\n");
    memory::init();
    uart::puts("Physical memory initialized\n");

    uart::puts("Initializing heap...\n");
    memory::init_heap();
    uart::puts("Heap initialized\n");

    uart::puts("Initializing page tables...\n");
    memory::mmu::init_page_tables();
    uart::puts("Page tables initialized\n");

    uart::puts("Installing page table root...\n");
    memory::mmu::install_page_table_root();
    uart::puts("Page table root installed\n");

    uart::puts("Enabling MMU...\n");
    arch::cpu::enable_mmu();
    uart::puts("MMU enabled\n");

    arch::gic::init_distributor();

    let redistributor = arch::gic::init_redistributor().expect("CPU0 GIC Redistributor not found");

    uart::puts("CPU0 GIC Redistributor: ");
    uart::put_hex(redistributor as u64);
    uart::putc(b'\n');

    uart::puts("GICv3 initialized\n");

    arch::timer_tick::init();

    uart::puts("Timer tick subsystem started\n");

    while arch::timer_tick::ticks() < 5 {
        core::hint::spin_loop();
    }

    arch::timer_tick::shutdown();

    uart::puts("Timer ticks: ");
    uart::put_hex(arch::timer_tick::ticks());
    uart::puts("\n");

    uart::puts("Starting CPU1 through PSCI...\n");

    let entry_point = secondary_entry as *const () as u64;
    let status = arch::psci::cpu_on(1, entry_point, 1);

    if status != 0 {
        uart::puts("PSCI CPU_ON failed: ");
        uart::put_hex(status as u64);
        uart::putc(b'\n');
    } else {
        // Wait for CPU1 to finish its CPU-local initialization.
        // Use the architectural counter for a bounded timeout.
        let start = arch::timer::counter();
        let timeout = arch::timer::frequency().saturating_mul(5);

        while SECONDARY_ONLINE.load(Ordering::Acquire) != 1 {
            if arch::timer::counter().wrapping_sub(start) >= timeout {
                break;
            }

            core::hint::spin_loop();
        }

        if SECONDARY_ONLINE.load(Ordering::Acquire) == 1 {
            uart::puts("CPU1 online: secondary entry confirmed\n");

            uart::puts("CPU0: Ready for SMP barrier\n");
            CPU0_READY.store(1, Ordering::Release);

            let start = arch::timer::counter();
            let timeout = arch::timer::frequency().saturating_mul(5);

            while CPU1_READY.load(Ordering::Acquire) == 0 {
                if arch::timer::counter().wrapping_sub(start) >= timeout {
                    uart::puts("SMP TEST FAILED: CPU1 barrier timeout\n");
                    break;
                }

                core::hint::spin_loop();
            }

            if CPU1_READY.load(Ordering::Acquire) != 0 {
                SMP_START.store(1, Ordering::Release);
                uart::puts("CPU0: Released SMP barrier\n");

                let cpu0_result = run_smp_workload(0x1234);
                CPU0_WORK_DONE.store(cpu0_result, Ordering::Release);

                let start = arch::timer::counter();
                let timeout = arch::timer::frequency().saturating_mul(5);

                while CPU1_WORK_DONE.load(Ordering::Acquire) == 0 {
                    if arch::timer::counter().wrapping_sub(start) >= timeout {
                        break;
                    }

                    core::hint::spin_loop();
                }

                let cpu1_result = CPU1_WORK_DONE.load(Ordering::Acquire);

                if cpu1_result != 0 && cpu0_result != cpu1_result {
                    uart::puts("SMP BARRIER TEST PASSED\n");
                } else {
                    uart::puts("SMP BARRIER TEST FAILED\n");
                }
            }
        } else {
            uart::puts("CPU1 startup timed out\n");
        }
    }

    let start = arch::timer::counter();
    let timeout = arch::timer::frequency().saturating_mul(5);

    while CPU1_READY.load(Ordering::Acquire) == 0 {
        if arch::timer::counter().wrapping_sub(start) >= timeout {
            uart::puts("SMP TEST FAILED: CPU1 barrier timeout\n");
            break;
        }

        core::hint::spin_loop();
    }

    if CPU1_READY.load(Ordering::Acquire) != 0 {
        SMP_START.store(1, Ordering::Release);
        uart::puts("CPU0: Released SMP barrier\n");

        let cpu0_result = run_smp_workload(0x1234);
        CPU0_WORK_DONE.store(cpu0_result, Ordering::Release);

        let start = arch::timer::counter();
        let timeout = arch::timer::frequency().saturating_mul(5);

        while CPU1_WORK_DONE.load(Ordering::Acquire) == 0 {
            if arch::timer::counter().wrapping_sub(start) >= timeout {
                break;
            }

            core::hint::spin_loop();
        }

        let cpu1_result = CPU1_WORK_DONE.load(Ordering::Acquire);

        if cpu1_result != 0 && cpu0_result != cpu1_result {
            uart::puts("SMP BARRIER TEST PASSED\n");
        } else {
            uart::puts("SMP BARRIER TEST FAILED\n");
        }
    }

    loop {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    halt()
}

fn halt() -> ! {
    loop {
        core::hint::spin_loop();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn exception_handler(frame: &mut ExceptionFrame) {
    let ec = arch::exception::exception_class(frame.esr);

    uart::puts("\n");
    uart::puts("Lych Kernel Exception\n");
    uart::puts("-------------------------------\n");

    uart::puts("Exception : ");
    uart::puts(arch::exception::exception_name(ec));
    uart::putc(b'\n');

    uart::puts("ESR_EL1   : ");
    uart::put_hex(frame.esr);
    uart::putc(b'\n');

    uart::puts("ELR_EL1   : ");
    uart::put_hex(frame.elr);
    uart::putc(b'\n');

    uart::puts("SPSR_EL1  : ");
    uart::put_hex(frame.spsr);
    uart::putc(b'\n');

    let previous_el = arch::exception::previous_exception_level(frame.spsr);

    uart::puts("Previous EL: ");
    uart::puts(arch::exception::exception_level_name(previous_el));
    uart::putc(b'\n');

    match ec {
        arch::exception::EC_INSTRUCTION_ABORT_LOWER_EL
        | arch::exception::EC_INSTRUCTION_ABORT_SAME_EL => {
            uart::puts("FAR_EL1   : ");
            uart::put_hex(arch::cpu::read_far_el1());
            uart::putc(b'\n');

            uart::puts("IFSC      : ");
            uart::put_hex(frame.esr & 0x3f);
            uart::putc(b'\n');

            uart::puts("Fault     : ");
            uart::puts(arch::exception::fault_status_name(frame.esr));
            uart::putc(b'\n');
        }

        arch::exception::EC_DATA_ABORT_LOWER_EL | arch::exception::EC_DATA_ABORT_SAME_EL => {
            uart::puts("FAR_EL1   : ");
            uart::put_hex(arch::cpu::read_far_el1());
            uart::putc(b'\n');

            uart::puts("DFSC      : ");
            uart::put_hex(frame.esr & 0x3f);
            uart::putc(b'\n');

            uart::puts("Fault     : ");
            uart::puts(arch::exception::fault_status_name(frame.esr));
            uart::putc(b'\n');
        }

        _ => {}
    }

    if ec == arch::exception::EC_BREAKPOINT {
        // BRK is the only synchronous exception with a safe resume path:
        // the instruction has completed, so skip it and return.
        // ARM64 instructions are fixed-width (4 bytes).
        frame.elr += 4;
        return;
    }

    // Instruction/data aborts, unhandled traps, and unknown ECs have no
    // recovery path here. Returning would re-execute the faulting
    // instruction indefinitely, so stop after printing diagnostics.
    uart::puts("Unhandled synchronous exception; halting\n");
    halt();
}

#[unsafe(no_mangle)]
pub extern "C" fn irq_handler() {
    let intid = arch::gic::acknowledge_interrupt();

    // IDs 1020-1023 are special interrupt IDs.
    if (1020..=1023).contains(&intid) {
        return;
    }

    if intid == arch::gic::TIMER_IRQ {
        arch::timer_tick::on_timer_interrupt();
    }

    arch::gic::end_interrupt(intid);
}

unsafe extern "C" {
    fn secondary_entry();
}

/// Rust entry point for a CPU started through PSCI.
#[unsafe(no_mangle)]
pub extern "C" fn secondary_rust_entry(cpu_id: u64) -> ! {
    // This first milestone intentionally supports CPU1 only.
    if cpu_id != 1 {
        loop {
            core::hint::spin_loop();
        }
    }

    // Initialize CPU-local architectural state only.
    // Do not initialize global memory, heap, or the GIC Distributor.
    arch::cpu::init();
    memory::mmu::install_page_table_root();
    arch::cpu::enable_mmu();

    uart::puts("CPU1: Initializing GIC Redistributor...\n");

    let redistributor = match arch::gic::init_redistributor() {
        Some(base) => base,
        None => {
            uart::puts("CPU1: GIC Redistributor not found!\n");
            loop {
                arch::cpu::wait_for_interrupt();
            }
        }
    };

    uart::puts("CPU1 GIC Redistributor: ");
    uart::put_hex(redistributor as u64);
    uart::putc(b'\n');

    let start_ticks = arch::timer_tick::ticks();
    let start_counter = arch::timer::counter();
    let timeout_ticks = arch::timer::frequency() * 2;

    arch::timer_tick::init();

    while arch::timer_tick::ticks().wrapping_sub(start_ticks) < 5
        && arch::timer::counter().wrapping_sub(start_counter) < timeout_ticks
    {
        core::hint::spin_loop();
    }

    arch::timer_tick::shutdown();

    let elapsed_ticks = arch::timer_tick::ticks().wrapping_sub(start_ticks);

    let elapsed_counter = arch::timer::counter().wrapping_sub(start_counter);

    uart::puts("CPU1 timer tick delta: ");
    uart::put_hex(elapsed_ticks);
    uart::putc(b'\n');

    uart::puts("CPU1 timer elapsed counter: ");
    uart::put_hex(elapsed_counter);
    uart::putc(b'\n');

    uart::puts("CPU1 timer timeout counter: ");
    uart::put_hex(timeout_ticks);
    uart::putc(b'\n');

    SECONDARY_ONLINE.store(cpu_id, Ordering::Release);

    while CPU0_READY.load(Ordering::Acquire) == 0 {
        core::hint::spin_loop();
    }

    uart::puts("CPU1: Ready for SMP barrier\n");
    CPU1_READY.store(1, Ordering::Release);

    while SMP_START.load(Ordering::Acquire) == 0 {
        core::hint::spin_loop();
    }

    uart::puts("CPU1: Starting workload\n");

    let cpu1_result = run_smp_workload(0x5678);
    CPU1_WORK_DONE.store(cpu1_result, Ordering::Release);

    uart::puts("CPU1: Workload complete\n");

    loop {
        arch::cpu::wait_for_interrupt();
    }
}
