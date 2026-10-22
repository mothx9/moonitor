use core_foundation::{
    base::{CFType, CFTypeRef, TCFType},
    dictionary::{CFDictionary, CFDictionaryRef},
    number::CFNumber,
    string::{CFString, CFStringRef},
};
use moonitor_core::*;
use std::{
    ffi::{c_char, c_void},
    ptr,
};

#[link(name = "IOKit", kind = "framework")]
unsafe extern "C" {
    fn IOServiceMatching(name: *const c_char) -> *mut c_void;
    fn IOServiceGetMatchingService(port: u32, matching: *mut c_void) -> u32;
    fn IORegistryEntryCreateCFProperty(
        entry: u32,
        key: CFStringRef,
        allocator: *const c_void,
        options: u32,
    ) -> CFTypeRef;
    fn IOObjectRelease(object: u32) -> i32;
    fn IOPSCopyPowerSourcesInfo() -> CFTypeRef;
    fn IOPSCopyPowerSourcesList(info: CFTypeRef) -> core_foundation::array::CFArrayRef;
    fn IOPSGetPowerSourceDescription(info: CFTypeRef, source: CFTypeRef) -> CFDictionaryRef;
}

fn number(dict: &CFDictionary<CFString, CFType>, key: &str) -> Option<f64> {
    dict.find(CFString::new(key))?
        .downcast::<CFNumber>()?
        .to_f64()
        .filter(|x| x.is_finite())
}

pub fn gpu(total_memory: u64, ts: u64) -> Option<GpuSnapshot> {
    // IOServiceGetMatchingService consumes the matching dictionary. All returned
    // CF properties follow Create ownership; the service is released separately.
    unsafe {
        let matching = IOServiceMatching(c"IOAccelerator".as_ptr());
        if matching.is_null() {
            return None;
        }
        let service = IOServiceGetMatchingService(0, matching);
        if service == 0 {
            return None;
        }
        let key = CFString::new("PerformanceStatistics");
        let raw =
            IORegistryEntryCreateCFProperty(service, key.as_concrete_TypeRef(), ptr::null(), 0);
        IOObjectRelease(service);
        if raw.is_null() {
            return None;
        }
        let object = CFType::wrap_under_create_rule(raw);
        if object.type_of() != CFDictionary::<CFString, CFType>::type_id() {
            return None;
        }
        let stats = CFDictionary::<CFString, CFType>::wrap_under_get_rule(raw.cast());
        let utilization =
            number(&stats, "Device Utilization %").filter(|x| (0.0..=100.0).contains(x));
        let allocated = number(&stats, "In use system memory").filter(|x| *x >= 0.0);
        Some(GpuSnapshot {
            id: "apple-unified".into(),
            name: "Apple GPU".into(),
            memory_kind: GpuMemoryKind::Unified,
            utilization_percent: utilization
                .map(|v| Reading::reported(v, "IOKit PerformanceStatistics", ts))
                .unwrap_or_else(|| Reading::unavailable(UnavailableReason::Unsupported)),
            memory_total_bytes: Reading::reported(total_memory, "shared host physical memory", ts),
            memory_used_bytes: allocated
                .map(|v| Reading::reported(v as u64, "IOKit In use system memory", ts))
                .unwrap_or_else(|| Reading::unavailable(UnavailableReason::Unsupported)),
            temperature_celsius: Reading::unavailable(UnavailableReason::Unsupported),
            clock_mhz: Reading::unavailable(UnavailableReason::Unsupported),
            throttling: Reading::unavailable(UnavailableReason::Unsupported),
        })
    }
}

pub fn battery(ts: u64) -> Reading<f64> {
    use core_foundation::array::CFArray;
    let unavailable = || Reading::unavailable(UnavailableReason::Unsupported);
    unsafe {
        let raw_info = IOPSCopyPowerSourcesInfo();
        if raw_info.is_null() {
            return unavailable();
        }
        let info = CFType::wrap_under_create_rule(raw_info);
        let raw_list = IOPSCopyPowerSourcesList(info.as_CFTypeRef());
        if raw_list.is_null() {
            return unavailable();
        }
        let sources = CFArray::<CFType>::wrap_under_create_rule(raw_list);
        for source in sources.iter() {
            let raw = IOPSGetPowerSourceDescription(info.as_CFTypeRef(), source.as_CFTypeRef());
            if raw.is_null() {
                continue;
            }
            let dict = CFDictionary::<CFString, CFType>::wrap_under_get_rule(raw);
            if let (Some(current), Some(max)) = (
                number(&dict, "Current Capacity"),
                number(&dict, "Max Capacity"),
            ) && max > 0.0
                && current >= 0.0
                && current <= max
            {
                return Reading::reported(current / max * 100.0, "IOKit power source", ts);
            }
        }
    }
    unavailable()
}
