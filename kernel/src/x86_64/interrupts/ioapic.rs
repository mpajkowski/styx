//! Interfaces for machine's IOAPIC

use crate::arch::PhysAddr;
use acpi::platform::interrupt::Apic;
use acpi::platform::interrupt::InterruptSourceOverride;
use acpi::platform::interrupt::IoApic;
use acpi::platform::interrupt::Polarity;
use acpi::platform::interrupt::TriggerMode;
use spin::Once;

trait IoApicExt {
    unsafe fn read(&self, register: u32) -> u32;
    unsafe fn write(&self, register: u32, value: u32);
}

impl IoApicExt for IoApic {
    unsafe fn read(&self, register: u32) -> u32 {
        let addr = PhysAddr::new_unchecked(self.address as u64)
            .to_io()
            .as_mut_ptr::<u32>();
        addr.write_volatile(register);
        addr.add(4).read()
    }

    unsafe fn write(&self, register: u32, value: u32) {
        let addr = PhysAddr::new_unchecked(self.address as u64)
            .to_io()
            .as_mut_ptr::<u32>();
        addr.write_volatile(register);
        addr.add(4).write_volatile(value);
    }
}

/// Interrupt model built during ACPI parse stage
pub static INTERRUPT_MODEL: Once<Apic> = Once::new();

fn interrupt_model() -> &'static Apic {
    INTERRUPT_MODEL
        .get()
        .expect("INTERRUPT_MODEL not initialized")
}

#[derive(Debug)]
enum IoApicRedirect<'a> {
    Irq(u8),
    InterruptSourceOverride(&'a InterruptSourceOverride),
}

/// Registers legacy IRQ by providing proper ioapic redirect
///
/// # Arguments
///
/// - `irq` - IRQ number
/// - `vec` - IDT vector
/// - `enable` - Enable or mask given IRQ
pub fn register_legacy_irq(irq: u8, vec: u8, enable: bool) {
    let iso = interrupt_model()
        .interrupt_source_overrides
        .iter()
        .find(|iso| iso.isa_source == irq);

    let redirect = match iso {
        Some(iso) => IoApicRedirect::InterruptSourceOverride(iso),
        None => IoApicRedirect::Irq(irq),
    };

    ioapic_redirect(vec, redirect, enable)
}

fn ioapic_redirect(vec: u8, redirect: IoApicRedirect, enable: bool) {
    let gsi = match &redirect {
        IoApicRedirect::Irq(irq) => *irq as u32,
        IoApicRedirect::InterruptSourceOverride(iso) => iso.global_system_interrupt,
    };

    let ioapic = match find_ioapic_handler(gsi) {
        Some(x) => x,
        None => {
            log::warn!("IOAPIC: ioapic for gsi={gsi} not found");
            return;
        }
    };

    let mut redirect_entry = 0_u64;

    redirect_entry |= vec as u64;
    redirect_entry |= (!enable as u64) << 16;

    if let IoApicRedirect::InterruptSourceOverride(iso) = redirect {
        redirect_entry |= (matches!(iso.polarity, Polarity::ActiveHigh) as u64) << 13;
        redirect_entry |= (matches!(iso.trigger_mode, TriggerMode::Level) as u64) << 15;
    }

    let ioredtbl = (gsi - ioapic.global_system_interrupt_base) * 2 + 16;

    unsafe {
        ioapic.write(ioredtbl, redirect_entry as u32);
        ioapic.write(ioredtbl + 1, (redirect_entry >> 32) as u32);
    }

    log::debug!("IOAPIC: registered entry [vec={vec}, gsi={gsi}]");
}

fn find_ioapic_handler(gsi: u32) -> Option<&'static IoApic> {
    interrupt_model().io_apics.iter().find(|ioapic| {
        let gsi_base = ioapic.global_system_interrupt_base;
        let max_redirect = ioapic_max_redirect(ioapic);

        (gsi_base..max_redirect).contains(&gsi)
    })
}

fn ioapic_max_redirect(ioapic: &IoApic) -> u32 {
    unsafe { ioapic.read(1) & 0xff0000 >> 16 }
}
