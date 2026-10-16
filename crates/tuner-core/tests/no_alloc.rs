//! `PitchDetector::detect` and `MonoTuner::process` run on the audio thread
//! and must not allocate.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::f32::consts::TAU;

use tuner_core::{MonoTuner, PitchConfig, PitchDetector};

struct Counting;

thread_local! {
    // Per thread: the test harness allocates on its own threads meanwhile.
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
}

fn allocations() -> usize {
    ALLOCATIONS.with(Cell::get)
}

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.with(|c| c.set(c.get() + 1));
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

    let before = allocations();
    let found = det.detect(&tone);
    let none = det.detect(&silence);
    let after = allocations();

    assert!(found.is_some() && none.is_none());
    assert_eq!(after - before, 0);
}

#[test]
fn mono_tuner_does_not_allocate() {
    let rate = 48_000.0;
    let mut tuner = MonoTuner::new(rate, 1024);
    let n = tuner.frame_size();
    let tone: Vec<f32> = (0..n)
        .map(|i| 0.1 * (TAU * 110.0 * i as f32 / rate).sin())
        .collect();
    let quiet = vec![1e-4; n];
    // Warm up: establish a floor, lock (narrowing the band) and release.
    for _ in 0..5 {
        tuner.process(&quiet);
    }
    let before = allocations();
    for _ in 0..5 {
        tuner.process(&tone);
    }
    for _ in 0..20 {
        tuner.process(&quiet);
    }
    let after = allocations();
    assert_eq!(after - before, 0);
}
