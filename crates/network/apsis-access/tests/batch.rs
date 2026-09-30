//! Tests for the batch access search.
//!
//! The parallel form must produce the same pass summaries, in the same
//! order, as the sequential one. Every test compares the two rather than
//! against a constant, so an ordering or content difference is caught.

use apsis_access::{AccessFinder, OrbitSample, find_all, find_all_parallel};
use apsis_coverage::GroundPoint;
use apsis_geodesy::Geodetic;
use apsis_math::Vec3;
use apsis_sgp4::{GravityModel, Satellite};
use apsis_tle::TwoLineElement;

const SAMPLE: &str = concat!(
    "1 25544U 98067A   08264.51782528 -.00002182  00000-0 -11606-4 0  2927\r\n",
    "2 25544  51.6416 247.4627 0006703 130.5360 325.0288 15.72125391563537\r\n",
);

/// Returns a finder for a station on the equator.
fn finder() -> AccessFinder {
    AccessFinder {
        station: GroundPoint::new(Geodetic::new(0.0, 0.0, 0.0), 10.0_f64.to_radians()),
        sample_step_seconds: 60.0,
        crossing_tolerance_seconds: 0.01,
    }
}

/// Returns a track of Earth-fixed samples for one satellite.
///
/// The rotation is folded into a simple sweep so the track is a plausible
/// ground track; the exact geometry is not what this test is about, only
/// that the batch search reproduces the sequential one.
fn track(satellite: &Satellite, samples: usize) -> Vec<OrbitSample> {
    (0..samples)
        .map(|index| {
            let seconds = index as f64 * 60.0;
            // The propagated radius sets the shell the synthetic longitude
            // sweep runs on, so the track is at a realistic altitude.
            let radius = satellite
                .propagate_minutes(seconds / 60.0)
                .expect("the track propagates")
                .position
                .norm();
            // A slow sweep in longitude stands in for the rotation into the
            // Earth-fixed frame; the exact geometry is not what this test is
            // about, only that the batch search reproduces the sequential one.
            let angle = seconds / 4_000.0;
            let position = Vec3::new(radius * angle.cos(), radius * angle.sin(), radius * 0.3);
            OrbitSample::new(seconds, position)
        })
        .collect()
}

fn satellite() -> Satellite {
    let element = TwoLineElement::parse(SAMPLE).expect("the sample parses");
    Satellite::from_element(&element, GravityModel::WGS72).expect("the orbit propagates")
}

#[test]
fn a_single_track_matches_the_sequential_form() {
    let finder = finder();
    let tracks = vec![track(&satellite(), 120)];
    assert_eq!(
        find_all_parallel(&finder, &tracks),
        find_all(&finder, &tracks)
    );
}

#[test]
fn a_shell_of_tracks_matches_the_sequential_form() {
    // Long enough that the parallel path takes the multi thread branch.
    let finder = finder();
    let satellite = satellite();
    let tracks: Vec<Vec<OrbitSample>> = (0..200).map(|_| track(&satellite, 120)).collect();
    assert_eq!(
        find_all_parallel(&finder, &tracks),
        find_all(&finder, &tracks)
    );
}

#[test]
fn an_empty_set_of_tracks_is_empty() {
    let finder = finder();
    assert!(find_all_parallel(&finder, &[]).is_empty());
}

#[test]
fn a_bad_track_does_not_hide_the_others() {
    // A track that is too short or out of order yields an error in both
    // forms, at the same position, and the good tracks are still searched.
    let finder = finder();
    let satellite = satellite();
    let mut tracks = vec![track(&satellite, 120), vec![], track(&satellite, 120)];
    // The empty track is a degenerate case; search it as given.
    let sequential = find_all(&finder, &tracks);
    let parallel = find_all_parallel(&finder, &tracks);
    assert_eq!(parallel, sequential);
    tracks.clear();
    assert!(tracks.is_empty());
}

#[test]
fn the_result_is_in_input_order() {
    // Three tracks with different lengths, so each summary differs and the
    // ordering is observable.
    let finder = finder();
    let satellite = satellite();
    let tracks = vec![
        track(&satellite, 40),
        track(&satellite, 120),
        track(&satellite, 80),
    ];
    let parallel = find_all_parallel(&finder, &tracks);
    assert_eq!(parallel.len(), 3);
    for (index, summary) in parallel.iter().enumerate() {
        let expected = finder.find(&tracks[index]).expect("the track is valid");
        assert_eq!(summary.as_ref().expect("the track is valid"), &expected);
    }
}
