//! Raw `extern "C"` exports of `tuner-core`. No wasm-bindgen: the module runs
//! inside an AudioWorkletGlobalScope, which lacks the APIs its JS glue needs,
//! so the worklet instantiates the bytes directly and calls these functions.

/// Returns [`tuner_core::VERSION`].
#[unsafe(no_mangle)]
pub extern "C" fn tuner_version() -> u32 {
    tuner_core::VERSION
}

#[cfg(test)]
mod tests {
    #[test]
    fn exports_core_version() {
        assert_eq!(super::tuner_version(), tuner_core::VERSION);
    }
}
