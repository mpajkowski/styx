use raw_cpuid::{ExtendedFeatures, FeatureInfo};

pub fn init() -> (FeatureInfo, ExtendedFeatures) {
    let cpuid = raw_cpuid::CpuId::default();

    let features = cpuid
        .get_feature_info()
        .expect("failed to get cpuid feature info");

    let extended_features = cpuid
        .get_extended_feature_info()
        .expect("failed to get cpuid extended feature info");

    (features, extended_features)
}
