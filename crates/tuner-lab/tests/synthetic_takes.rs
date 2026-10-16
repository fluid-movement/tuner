//! End-to-end: synthetic takes written to WAV files and analysed like real
//! recordings.

use std::path::PathBuf;
use tuner_core::tuning::{Instrument, cents_between};
use tuner_lab::analyze::{TakeReport, analyze_file};
use tuner_lab::audio::write_wav;
use tuner_lab::demo::{SyntheticTake, synthetic_take, synthetic_take_at};
use tuner_lab::segment::Kind;

fn temp_path(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("tuner-lab-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

fn check(take: &SyntheticTake, report: &TakeReport, instrument: Instrument, strums: usize) {
    let n = instrument.strings().len();
    let kinds: Vec<Kind> = report.events.iter().map(|e| e.kind).collect();
    let mut expected = vec![Kind::Pluck; n];
    expected.extend(vec![Kind::Strum; strums]);
    assert_eq!(
        kinds, expected,
        "{}: events {:?}",
        report.file, report.events
    );
    for (e, t) in report
        .events
        .iter()
        .zip(take.pluck_onsets.iter().chain(&take.strum_onsets))
    {
        assert!((e.onset - t).abs() < 0.05, "onset {} vs {t}", e.onset);
    }

    assert_eq!(report.plucks.len(), n);
    for (i, p) in report.plucks.iter().enumerate() {
        assert_eq!(p.string_index, i);
        let off = cents_between(p.frequency, take.frequencies[i]);
        assert!(off.abs() <= 2.0, "{} truth {:+.2} c off", p.string, off);
        assert!(
            p.shown_share >= 0.9,
            "{} shown {:.0}%",
            p.string,
            p.shown_share * 100.0
        );
        assert!(
            p.first_reading_ms.is_some_and(|ms| ms <= 100.0),
            "{} latency {:?}",
            p.string,
            p.first_reading_ms
        );
    }

    assert_eq!(report.strums.len(), strums);
    for strum in &report.strums {
        for st in &strum.strings {
            assert_ne!(
                st.agree,
                Some(false),
                "strum at {:.2}s: {} verdicts differ",
                strum.onset,
                st.string
            );
            let err = st.error_cents.expect("estimate and truth");
            assert!(
                err.abs() <= 3.0,
                "strum at {:.2}s: {} off by {err:+.2} c",
                strum.onset,
                st.string
            );
        }
    }
}

#[test]
fn guitar_take_with_flat_e4_and_sharp_b3() {
    let detunes = [1.0, -3.0, 0.0, 2.0, 9.0, -12.0];
    let take = synthetic_take(Instrument::Guitar, &detunes, 3, 1);
    let path = temp_path("guitar_e4-flat_b3-sharp.wav");
    write_wav(&path, &take.audio).unwrap();
    let report = analyze_file(&path).unwrap();
    check(&take, &report, Instrument::Guitar, 3);
}

#[test]
fn ukulele_take_mixed_detuning() {
    let detunes = [-15.0, 3.0, 9.0, -1.0];
    let take = synthetic_take(Instrument::Ukulele, &detunes, 3, 2);
    let path = temp_path("ukulele_mixed.wav");
    write_wav(&path, &take.audio).unwrap();
    let report = analyze_file(&path).unwrap();
    check(&take, &report, Instrument::Ukulele, 3);
}

#[test]
fn stereo_44k1_wav_is_mixed_to_mono() {
    let detunes = [0.0, 4.0, -7.0, 1.0];
    let take = synthetic_take_at(Instrument::Ukulele, &detunes, 2, 3, 44_100.0);
    let path = temp_path("uke_stereo.wav");
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: 44_100,
        bits_per_sample: 24,
        sample_format: hound::SampleFormat::Int,
    };
    let mut w = hound::WavWriter::create(&path, spec).unwrap();
    for &s in &take.audio.samples {
        let v = (s * 8_388_607.0) as i32;
        w.write_sample(v).unwrap();
        w.write_sample(v).unwrap();
    }
    w.finalize().unwrap();
    let report = analyze_file(&path).unwrap();
    assert_eq!(report.sample_rate, 44_100.0);
    check(&take, &report, Instrument::Ukulele, 2);
}

#[test]
fn file_names_must_name_the_instrument() {
    let take = synthetic_take(Instrument::Ukulele, &[0.0; 4], 1, 4);
    let path = temp_path("take_01.wav");
    write_wav(&path, &take.audio).unwrap();
    let err = analyze_file(&path).unwrap_err();
    assert!(err.contains("guitar"), "{err}");
}
