//! `PitchDetector::detect` runs on the audio thread and must not allocate.

use std::alloc::{GlobalAlloc, Layout, System};
use std::f32::consts::TAU;
use std::sync::atomic::{AtomicUsize, Ordering};

use tuner_core::{PitchConfig, PitchDetector};

struct Counting;

static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::SeqCst);
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

#[test]
fn detect_does_not_allocate() {
    let rate = 48_000.0;
    let mut det = PitchDetector::new(PitchConfig::new(rate));
    let tone: Vec<f32> = (0..2048)
        .map(|i| 0.5 * (TAU * 110.0 * i as f32 / rate).sin())
        .collect();
    let silence = vec![0.0; 2048];

    let before = ALLOCATIONS.load(Ordering::SeqCst);
    let found = det.detect(&tone);
    let none = det.detect(&silence);
    let after = ALLOCATIONS.load(Ordering::SeqCst);

    assert!(found.is_some() && none.is_none());
    assert_eq!(after - before, 0);
}
