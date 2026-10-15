//! Pitch analysis for the tuner. Pure Rust with no web dependencies, so it
//! can be tested natively and wrapped for WASM by `tuner-wasm`.

/// Version of the analysis core, exposed so the web app can confirm the
/// module it loaded.
pub const VERSION: u32 = 1;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_set() {
        assert_eq!(VERSION, 1);
    }
}
