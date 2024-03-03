//! Local APIC

use easybit::set_range;
use raw_cpuid::FeatureInfo;
use spin::Once;

use crate::x86_64::{
    interrupts,
    msr::{rdmsr, wrmsr},
    phys_to_io,
    sync::{Mutex, MutexGuard},
    PhysAddr, VirtAddr,
};

use super::InterruptStack;

/// Local APIC handle
pub static LOCAL_APIC: Once<Mutex<LocalApic>> = Once::new();

/// Local X2APIC Architecture, 2.3.2 APIC Register Address Space
const X2APIC_MSR_BASE: u32 = 0x800;

// The MSR address range between 0000_0800H through 0000_0BFFH is architecturally
// reserved and dedicated for accessing APIC registers in x2APIC mode.

/// Intel(R) manual Vol 3, Table 2-2. IA-32 Architectural MSRs (Contd.)
const IA32_APIC_BASE: u32 = 0x1b;

const TASK_PRIORITY_REGISTER_MSR: u32 = 0x080;
const EOI: u32 = 0x0B0;
const SPURIOUS_INTERRUPT_VECTOR_REGISTER: u32 = 0x0f0;
const ID: u32 = 0x020;
const ICR: u32 = 0x300;
const LVT_ERROR: u32 = 0x370;

/// Initializes LAPIC
pub fn init(feat: &FeatureInfo) {
    let local_apic = if feat.has_x2apic() {
        log::info!("APIC: X2APIC detected");
        LocalApic::X2Apic
    } else if feat.has_apic() {
        log::info!("APIC: XAPIC detected");
        let apic_base = unsafe { rdmsr(IA32_APIC_BASE) };
        let apic_phys = PhysAddr::new_unchecked(apic_base & 0xffff_0000);
        let virt_base = phys_to_io(apic_phys);
        LocalApic::XApic { addr: virt_base }
    } else {
        panic!("APIC: X2APIC nor XAPIC detected");
    };

    local_apic.init();
    LOCAL_APIC.call_once(|| Mutex::new(local_apic));
}

/// Returns `LocalApic` handle
pub fn local_apic() -> MutexGuard<'static, LocalApic> {
    LOCAL_APIC.get().expect("Local APIC not initialized").lock()
}

/// LocalApic variant present on machine
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalApic {
    /// XAPIC variant
    XApic {
        /// Virtual address of X2APIC MMIO
        addr: VirtAddr,
    },

    /// X2APIC variant
    X2Apic,
}

impl LocalApic {
    /// Returns BSP LAPIC ID
    pub fn bsp_id(&self) -> u8 {
        let bsp_id_raw = self.read_u32(ID);
        bsp_id_raw.try_into().unwrap()
    }

    /// Sends IPI to given AP denoted by `apic_id`
    pub fn send_init_ipi(&mut self, apic_id: u64) {
        let val = self.combine_val(0x4500, apic_id);
        self.write_u64(ICR, val)
    }

    /// Sends SIPI to given AP denoted by `apic_id`
    pub fn send_startup_ipi(&mut self, apic_id: u64) {
        let val = self.combine_val(0x4601, apic_id);
        self.write_u64(ICR, val)
    }

    /// Notifies LAPIC that the interrupt has ended
    pub fn notify_end_of_interrupt(&mut self) {
        self.write_u32(EOI, 0);
    }

    fn combine_val(&self, val: u64, apic_id: u64) -> u64 {
        match self {
            LocalApic::XApic { .. } => {
                let mut out = 0;
                out = set_range!(out, 0..32, apic_id << 24);
                out = set_range!(out, 32..64, val);
                out
            }
            LocalApic::X2Apic => (apic_id << 32) | val,
        }
    }

    fn init(&self) {
        if *self == Self::X2Apic {
            unsafe {
                let base = rdmsr(IA32_APIC_BASE);
                // set local APIC in x2APIC mode
                wrmsr(IA32_APIC_BASE, base | 1 << 10);
            }
        }

        // to enable all interrupts
        self.write_u32(TASK_PRIORITY_REGISTER_MSR, 0);

        // enable local apic
        self.write_u32(SPURIOUS_INTERRUPT_VECTOR_REGISTER, 0x1ff);

        let vec = interrupts::register_interrupt(on_interrupt);
        self.write_u32(LVT_ERROR, vec as u32);

        interrupts::switch_to_apic();
    }

    fn read_u32(&self, register: u32) -> u32 {
        match self {
            Self::XApic { addr } => {
                let addr = (addr.to_u64() + register as u64) as *mut u32;
                unsafe { addr.read_volatile() }
            }
            Self::X2Apic => unsafe { rdmsr(X2APIC_MSR_BASE + (register >> 4)) as u32 },
        }
    }

    fn write_u64(&self, register: u32, value: u64) {
        match self {
            Self::XApic { addr } => {
                let addr_low = (addr.to_u64() + register as u64) as *mut u32;
                let addr_high = (addr.to_u64() + register as u64 + 0x10) as *mut u32;

                unsafe {
                    addr_high.write_volatile(value as u32);
                    addr_low.write_volatile((value >> 32) as u32);
                }
            }
            Self::X2Apic => {
                let msr = X2APIC_MSR_BASE + (register >> 4);
                unsafe { wrmsr(msr, value) }
            }
        }
    }

    fn write_u32(&self, register: u32, value: u32) {
        match self {
            Self::XApic { addr } => unsafe {
                let addr = (addr.to_u64() + register as u64) as *mut u32;
                addr.write_volatile(value);
            },
            Self::X2Apic => self.write_u64(register, value as u64),
        }
    }
}

fn on_interrupt(_: &mut InterruptStack) {
    log::info!("lapic interrupt");
}
