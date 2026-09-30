# Units and coordinates

English | [简体中文](./units.zh.md)

This file records the unit and coordinate conventions that cross layers. The
conventions inside a single crate are in its module documentation; what
belongs here is the part that must agree **across a layer boundary**, because
a unit that disagrees at a boundary is the easiest mistake to introduce in
this repository and the hardest to find.

## Two units of length

The repository uses both kilometres and metres, with the boundary between the
orbit layer and the physics layer:

| Layer / crate | Unit of length |
| --- | --- |
| `apsis-kepler`, `apsis-sgp4`, `apsis-tle` | kilometres |
| `apsis-ephemeris` | kilometres |
| `apsis-maneuver` | kilometres |
| `apsis-constellation`, `apsis-coverage`, `apsis-access`, `apsis-isl` | kilometres |
| `apsis-topology`, `apsis-routing`, `apsis-mesh` | length is computed only in `apsis-isl`; an edge's `cost` is a dimensionless value the caller defines |
| `apsis-handover` | no length; `quality` is defined by the caller and time is in seconds |
| `apsis-coord` (`Eci`, `Ecef`) | metres |
| `apsis-geodesy` (geodetic height, ellipsoid semi-axes, Cartesian position) | metres |
| `apsis-frames` | dimensionless (angles and rotations) |
| `apsis-rf`, `apsis-antenna`, `apsis-atmosphere`, `apsis-noise`, `apsis-doppler`, `apsis-interference`, `apsis-link` | metres, hertz |

The choice is not arbitrary. Satellite orbits read naturally in kilometres,
which is the magnitude `apsis-sgp4` produces, and a radio link reads
naturally in metres, which is what the ITU-R recommendations use. The cost is
that the boundary needs an explicit conversion.

### Converting at the boundary

Multiply by `1000.0` on the way from the orbit layer into the physics layer,
and divide on the way back:

```rust
// A position in kilometres from the orbit layer becomes metres in physics.
let position_km = satellite.propagate_minutes(0.0)?.position;  // kilometres
let ecef = Ecef::new(position_km * 1_000.0);                    // metres
```

Omitting this step puts the satellite inside the Earth, and the numbers still
look plausible rather than failing at once. The `integrate_network` example
asserts on this boundary: the radius after the conversion must match the
radius the propagator reports, which is the only check that finds a missing
multiply or divide automatically.

## Angles

**Every angle used in a calculation is in radians.** Latitude, longitude,
inclination, right ascension of the ascending node, argument of perigee, true
anomaly, elevation, azimuth and antenna pattern angle are all radians.

The only exception is the two-line element set, whose raw fields are in
degrees and revolutions per day because that is how the file encodes them.
`apsis-sgp4` converts them to radians and radians per minute on
initialisation.

## Time

Time is in seconds throughout, with the origin chosen by the caller:

- The ops and sim layers count seconds from the scenario epoch.
- The orbit layer uses minutes from the element set epoch in `apsis-sgp4`;
  `apsis-ephemeris` and the rest use seconds.
- Calendar time: `apsis-time::Epoch` holds a TAI Julian date, and a
  conversion to UTC goes through the leap second table in `apsis-leap`.

`Orbit::sample_ephemeris` is the boundary between the two representations: it
takes minutes from the element epoch and the ephemeris it returns is in
seconds, with the conversion done inside the method.

## Coordinate frames

| Frame | Type | Notes |
| --- | --- | --- |
| TEME | `apsis_sgp4::State` | the true equator, mean equinox frame the SGP4 model defines |
| ECI | `apsis_coord::Eci` | inertial Cartesian coordinates |
| ECEF | `apsis_coord::Ecef` | Earth-fixed Cartesian coordinates |
| Geodetic | `apsis_geodesy::Geodetic` | latitude, longitude, height above the ellipsoid |
| Ground point | `apsis_coverage::GroundPoint` | a geodetic position plus an elevation mask |

Going from TEME to ECEF needs the Earth rotation angle
(`apsis-frames::EarthRotationAngle`) together with precession and nutation.
`apsis-coord` provides the transformation and `apsis-frames` provides the
angle. TEME differs from ECI by the equation of the equinoxes, which is below
a thousandth of a degree; the `integrate_network` example treats it as the
inertial frame and says so, because for that example it is an approximation.

## Other quantities

- The gravitational parameter is `apsis-kepler::GravityParameter`, in
  cubic kilometres per second squared. WGS 72 goes with SGP4 and WGS 84 with
  everything else; choosing the wrong one shifts the osculating semi-major
  axis by about one part in a million. The `apsis-orbit::Orbit` facade binds
  the parameter to the propagator so that this cannot go wrong.
- Velocity follows the unit of length in its own layer, so km/s or m/s.
- Power is in watts, frequency in hertz and temperature in kelvin. A decibel
  is a power ratio; `dBi` is relative to an isotropic radiator and `dBd` to a
  half-wave dipole, which is 2.15 dB more directive.
- Every relative quantity is dimensionless: eccentricity, fill rate,
  utilisation and a signal to noise ratio in decibels.

## Checks

Nothing in the type system protects a unit boundary, so the tests and this
document carry it:

- Each layer's tests cover the conversions inside that layer.
- `integrate_network` asserts that the kilometre to metre boundary conversion
  preserves the radius.
- `integrate_ops` and `integrate_sim` cover the time axis conventions of the
  ops and sim layers.
- `cargo run --release -p apsis-rs --features full --example bench` touches
  the hot path of every layer, so an interface change in any of them shows up
  there as a build failure.
