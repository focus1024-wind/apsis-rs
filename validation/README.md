# Cross-check validation

English | [简体中文](./README.zh.md)

This directory holds the numerical cross-checks of the Rust implementation
against public reference implementations. The Python side is for validation
only: it is not a Rust dependency and takes no part in the Rust build.

## The Python environment

It is managed with [pixi](https://pixi.sh/):

```bash
cd validation
pixi install
```

The libraries that are allowed:

- `sgp4`: the SGP4 propagation cross-check
- `astropy`: the time and coordinate cross-checks
- `numpy`: numerical computation
- `skyfield`: the orbit and visibility cross-checks, optional
- `itur`: the ITU-R reference implementations of P.676, P.838 and P.618, for
  the atmosphere and rain cross-checks

Adding a new Python library needs to be assessed and agreed first.

## Layout

```text
validation/
├── pixi.toml
├── python/
│   ├── orbit/           # SGP4 and two-body cross-checks
│   ├── time/            # time scale and Julian date cross-checks
│   ├── coord/           # coordinate transform cross-checks
│   ├── link/            # link budget cross-checks
│   └── constellation/   # coverage geometry cross-checks
└── data/
    └── expected/        # data dumped by the Rust side for comparison
```

## The flow

The Rust side writes the data through an example of the crate, as CSV on
standard output, formatted by hand without pulling in serde:

```bash
# Geodetic -> ECEF
cargo run --features physics --example dump_geodesy \
    > validation/data/expected/rust_geodetic.csv

# Time scale conversions
cargo run --features physics --example dump_time \
    > validation/data/expected/rust_time.csv

# SGP4 and SDP4 propagation
cargo run --features orbit --example dump_orbit \
    > validation/data/expected/rust_sgp4.csv

# Link budget
cargo run --features rf --example dump_link \
    > validation/data/expected/rust_link.csv

# ITU-R P.676-12 gaseous absorption
cargo run --features rf --example dump_gas \
    > validation/data/expected/rust_gas.csv

# ITU-R P.838-3 rain attenuation coefficients
cargo run --features rf --example dump_rain \
    > validation/data/expected/rust_rain.csv

# ITU-R P.618-13 rain on a slant path
cargo run --features rf --example dump_rain_path \
    > validation/data/expected/rust_rain_path.csv

# ITU-R S.465 reference antenna pattern
cargo run --features rf --example dump_s465 \
    > validation/data/expected/rust_s465.csv

# ITU-R P.839-4 zero degree isotherm height
cargo run --features rf --example dump_p839 \
    > validation/data/expected/rust_p839.csv

# ITU-R P.676-12 exact slant path, layered integration with a refractive
# index ray trace
cargo run --features rf --example dump_gas_exact \
    > validation/data/expected/rust_gas_exact.csv

# IAU 2000/2006 Earth orientation: precession, nutation, frame offsets,
# GCRS to ITRF
cargo run --features physics --example dump_iau2000 \
    > validation/data/expected/rust_iau2000.csv

# TEME to the Earth-fixed frame, the SGP4 output chain
cargo run --features physics --example dump_teme \
    > validation/data/expected/rust_teme.csv

# Coverage geometry: elevation, nadir angle, central angle, slant range and
# footprint area
cargo run --features network --example dump_coverage \
    > validation/data/expected/rust_coverage.csv

# The two-body model: elements to a state vector and back
cargo run --features orbit --example dump_kepler \
    > validation/data/expected/rust_kepler.csv

# Two-line element parsing: fields, epoch expansion, modulo ten checksum
cargo run --features orbit --example dump_tle \
    > validation/data/expected/rust_tle.csv
```

The Python side reads the same inputs, recomputes them with a reference
implementation, and reports the differences:

```bash
pixi run python validation/python/coord/check_geodetic.py
pixi run python validation/python/time/check_time.py
pixi run python validation/python/orbit/check_sgp4.py
pixi run python validation/python/link/check_link.py
pixi run python validation/python/atmosphere/check_gas.py
pixi run python validation/python/atmosphere/check_rain.py
pixi run python validation/python/atmosphere/check_rain_path.py
pixi run python validation/python/link/check_s465.py
pixi run python validation/python/atmosphere/check_p839.py
pixi run python validation/python/atmosphere/check_gas_exact.py
pixi run python validation/python/time/check_iau2000.py
pixi run python validation/python/coord/check_teme.py
pixi run python validation/python/constellation/check_coverage.py
pixi run python validation/python/orbit/check_kepler.py
pixi run python validation/python/orbit/check_tle.py
```

Every script exits with zero to pass, one to exceed the tolerance, and two
when the input file is missing.

## Tolerances

| Quantity | Tolerance |
| --- | --- |
| Position and distance | < 1e-3 km for SGP4 propagation, 1e-9 km for a geometric quantity |
| Velocity | < 1e-6 km/s |
| Time | < 1e-6 s |
| Coverage angle | 1e-12 rad |
| Area and coverage fraction | 1e-12 relative |
| Link budget and attenuation | 1e-6 to 1e-9 dB, depending on the model |
| Earth orientation matrix | 1e-9 |

Each script names its tolerance as a constant at the top of its source, set
by the physical quantity it compares; see the "Tolerances of the
cross-checks" section of `docs/design/errors.md`.

When a check exceeds its tolerance the fix is in the Rust algorithm. The
tolerance is not widened unless that has been agreed.

Every script must run offline on its own and reach no network.
`check_time.py` disables the astropy Earth orientation download, which none
of the conversions in this crate needs.

## The checks that exist

| Script | What it compares | Reference | Worst difference |
| --- | --- | --- | --- |
| `coord/check_geodetic.py` | `Geodetic` to ECEF, 280 points across latitude, longitude and height | `astropy.coordinates.EarthLocation` | 1.1e-8 m |
| `time/check_time.py` | UTC to TAI, 117 instants covering every leap second boundary | `astropy.time.Time` | 5.3e-7 s |
| `orbit/check_sgp4.py` | SGP4 and SDP4 propagation, 4 element sets (near Earth, highly eccentric, deep space, GTO) at 9 instants | the `sgp4` reference library | position 1.2e-8 km, velocity 5.3e-12 km/s |
| `link/check_link.py` | link budget, 4 scenarios (Ku downlink, S band, Ka wideband, L band) | the closed form link equation, rewritten in Python | 5.0e-13 dB |
| `atmosphere/check_gas.py` | gaseous absorption, 18 frequencies covering the 60 GHz oxygen complex and the 22 and 183 GHz water lines | the ITU-R P.676-12 implementation in `itur` | dry air 6.1e-11, water 6.4e-9, relative |
| `atmosphere/check_rain.py` | rain coefficients, 17 frequencies at 4 elevation and polarisation pairs | the ITU-R P.838-3 implementation in `itur` | k 7.5e-9, alpha 6.5e-13, relative |
| `atmosphere/check_rain_path.py` | rain on a slant path, 8 scenarios | the ITU-R P.618-13 equations, transcribed to Python with coefficients from `itur` | 3.4e-13 km and 4.3e-13 dB |
| `link/check_s465.py` | reference antenna pattern, 3 apertures at 8 off-axis angles | the ITU-R S.465 equations, transcribed to Python | 4.0e-10 dB |
| `atmosphere/check_p839.py` | zero degree isotherm height, 15 points worldwide | the ITU-R P.839-4 implementation in `itur` | 3.3e-13 km |
| `atmosphere/check_gas_exact.py` | exact slant path, 11 scenarios from 1 to 100 GHz at 1 to 90 degrees elevation | the exact method of ITU-R P.676-12 in `itur` | 9.6e-9, relative |
| `time/check_iau2000.py` | precession, nutation, frame offsets, GAST and GCRS to ITRF, 8 epochs by 28 quantities | ERFA 2.0.1, derived from IAU SOFA | nutation 6.6e-15 rad, BPN matrix 6.0e-15, GAST 1.1e-11 rad |
| `coord/check_teme.py` | TEME to the Earth-fixed frame, 4 epochs | `gmst82` and `pom00` from ERFA, both with zero polar motion | 4.9e-10 km |
| `constellation/check_coverage.py` | coverage geometry, 6 scenarios of LEO, MEO and GEO at low, usual and strict masks, plus a round trip between the central angle and the elevation | the spherical triangle relations, rewritten in Python from the geometry | angle 0, area 2.2e-16 relative |
| `orbit/check_kepler.py` | two-body conversion over 7 orbits (circular, low, moderate and high eccentricity, retrograde, near polar and geostationary), plus a state back to elements at 36 true anomalies | the perifocal formulae, the rotation matrix and the vis-viva equation, rewritten in Python | position, velocity and period 0; energy 3.0e-14 km^2/s^2; recovered elements about 1e-11 km |
| `orbit/check_tle.py` | element parsing, 8 element sets by 17 fields, covering the two digit year expansion, the implied leading zero of the eccentricity, the exponential drag term and the modulo ten checksum | an independent column indexed reader of the NORAD format | exact, field by field |

### A note on the radio frequency cross-checks

The radio frequency layer divides into two kinds of model: those with an
authoritative reference implementation, and those that are purely a closed
form expression.

- Where a reference implementation exists it is compared directly: Earth
  orientation uses `erfa`, derived from IAU SOFA, and the atmosphere and rain
  use `itur`. Every step of the link budget is elementary arithmetic, and
  `check_link.py` rewrites the same equation independently in Python.
- The rest are checked by asserting analytic properties: the integration tests
  of each module verify what the model must satisfy, such as the 3 dB point, the
  square law falloff, the behaviour of a Friis cascade, and the value of the
  P.838-3 coefficients at the tabulated frequencies.

Every recommendation is at its **current revision**: P.676-12, P.838-3,
P.618-13, P.839-4 and S.465. The coefficients of P.838-3 are a closed form
curve fit, not the superseded numerical table of P.838-1.

### A note on the SGP4 cross-check convention

The epoch of a TLE is a UTC calendar instant, but SGP4 treats it as a uniform
time origin, which is the convention the theory was fitted with. Converting
the epoch to TAI before handing it to the propagator would introduce a leap
second offset of about 33 seconds, and that offset would propagate into the
luni-solar series of the deep space branch. Both sides of the cross-check
therefore use the UTC Modified Julian Date as the element set states it.

### A note on the difference in the time cross-check conventions

There are two conventions for a floating point day number of UTC:

- This repository uses a linear Modified Julian Date, where `TAI - UTC` is a
  step function and the offset is constant for the rest of a leap second
  day.
- astropy and ERFA use a quasi-Julian date, where a leap second day has
  86401 seconds and their `tai - utc` difference varies through the day.

Comparing the two Julian dates directly would therefore measure a difference
of convention rather than a numerical error. `check_time.py` compares the
**TAI instant** instead: both sides take the same UTC calendar instant and
the resulting TAI Modified Julian Date is compared. The comparison uses an
integer day plus a fraction of a day in two parts, which avoids the roughly
40 us of resolution a single `f64` loses near an MJD of 60000.

`dump_time` writes the TAI instant in the same two parts for the same
reason.
