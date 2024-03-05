use core::ptr::NonNull;

use acpi::{AcpiHandler, InterruptModel, PlatformInfo};

use crate::arch::{ap, interrupts::ioapic, PhysAddr, VirtAddr};

use super::limine::Limine;

pub fn init(boot_info: &Limine) -> Result<(), acpi::AcpiError> {
    let rsdp = boot_info.rsdp.address().expect("null rsdp address");

    log::info!("rsdp address: {rsdp:?}");

    let tables = unsafe {
        acpi::AcpiTables::from_rsdp(
            VirtToPhysAcpiHandler,
            VirtAddr::new_unchecked(rsdp.as_ptr() as u64)
                .to_phys()
                .to_u64() as usize,
        )
    }?;

    let PlatformInfo {
        power_profile: _,
        interrupt_model,
        processor_info,
        pm_timer: _,
    } = tables.platform_info()?;

    if let InterruptModel::Apic(apic) = interrupt_model {
        ioapic::INTERRUPT_MODEL.call_once(|| apic);
    } else {
        log::error!("apic not found");
    }

    if let Some(processor_info) = processor_info {
        ap::PROCESSOR_INFO.call_once(|| processor_info);
    } else {
        log::warn!("processor info not found");
    }

    Ok(())
}

#[derive(Clone, Copy)]
struct VirtToPhysAcpiHandler;

impl AcpiHandler for VirtToPhysAcpiHandler {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> acpi::PhysicalMapping<Self, T> {
        let addr = PhysAddr::new_unchecked(physical_address as u64);
        let virt = NonNull::new(addr.to_io().as_mut_ptr()).expect("null addr");

        acpi::PhysicalMapping::new(physical_address, virt, size, size, *self)
    }

    fn unmap_physical_region<T>(_region: &acpi::PhysicalMapping<Self, T>) {
        // no operation for now
    }
}
