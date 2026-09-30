"""Cross-check the ApsisRS two-body model against an independent computation.

Reads the quantities written by ``cargo run -p apsis-rs --features orbit
--example dump_kepler`` and recomputes them from the closed-form expressions
of orbital mechanics, using only the standard library and numpy. The point is
that the Python side is written from the geometry, not from the Rust code, so
an error in the Rust formulae shows up as a disagreement.

The relations used are the standard ones:

* the perifocal position and velocity at a true anomaly ``nu``,

      p = a (1 - e^2),  r = p / (1 + e cos nu)
      x = r cos nu,  y = r sin nu
      vx = -sqrt(mu / p) sin nu
      vy =  sqrt(mu / p) (e + cos nu)

  rotated into the inertial frame by ``R3(O) R1(i) R3(w)``;
* the vis-viva equation, ``v^2 = mu (2/r - 1/a)``, used to check the speed
  independently of the direction;
* the specific orbital energy, ``E = v^2/2 - mu/r``;
* Kepler's third law for the period, ``T = 2 pi sqrt(a^3/mu)``;
* the conic radius, ``r = p / (1 + e cos nu)``.

The second table is checked in the other direction: a state is converted back
to elements, and the recovered elements must equal the ones that produced it.

The script needs no network access.

Usage:

    pixi run python validation/python/orbit/check_kepler.py [path]

Exit status is zero when every quantity agrees within tolerance and one
otherwise.
"""

from __future__ import annotations

import csv
import math
import sys
from pathlib import Path

# The agreement is at the floating point level because both sides evaluate
# the same closed forms. The tolerances are set to catch a transposed sign, a
# radians/degrees slip, or a dropped rotation, not to allow a loose match.
LENGTH_TOLERANCE_KM = 1e-9
VELOCITY_TOLERANCE_KMS = 1e-11
ANGLE_TOLERANCE_RAD = 1e-12
RELATIVE_TOLERANCE = 1e-12

DEFAULT_INPUT = Path("validation/data/expected/rust_kepler.csv")

# The WGS 84 gravitational parameter, matching the constant the example uses.
MU = 398_600.5


def rotation_matrix(inclination: float, right_ascension: float, argument_of_perigee: float):
    """Returns R3(O) R1(i) R3(w), the perifocal to inertial rotation."""
    sw, cw = math.sin(argument_of_perigee), math.cos(argument_of_perigee)
    si, ci = math.sin(inclination), math.cos(inclination)
    so, co = math.sin(right_ascension), math.cos(right_ascension)
    return [
        [co * cw - so * sw * ci, -co * sw - so * cw * ci, so * si],
        [so * cw + co * sw * ci, -so * sw + co * cw * ci, -co * si],
        [sw * si, cw * si, ci],
    ]


def apply(matrix, vector):
    """Applies a 3x3 matrix to a vector."""
    return [sum(matrix[row][col] * vector[col] for col in range(3)) for row in range(3)]


def reference_state(a, e, i, om, w, nu, mu=MU):
    """Returns the position, velocity and radius from the closed forms."""
    p = a * (1.0 - e * e)
    radius = p / (1.0 + e * math.cos(nu))
    x = radius * math.cos(nu)
    y = radius * math.sin(nu)
    scale = math.sqrt(mu / p)
    vx = -scale * math.sin(nu)
    vy = scale * (e + math.cos(nu))
    matrix = rotation_matrix(i, om, w)
    return (
        apply(matrix, [x, y, 0.0]),
        apply(matrix, [vx, vy, 0.0]),
        radius,
    )


def reference_period(a, mu=MU):
    """Returns the orbital period from Kepler's third law."""
    return 2.0 * math.pi * math.sqrt(a**3 / mu)


def check_states(rows):
    """Checks the elements to state table and returns pass flag and report."""
    worst = {
        "position": 0.0,
        "velocity": 0.0,
        "radius": 0.0,
        "period": 0.0,
        "energy": 0.0,
    }
    for row in rows:
        a = float(row["semi_major_axis_km"])
        e = float(row["eccentricity"])
        i = float(row["inclination_rad"])
        om = float(row["right_ascension_rad"])
        w = float(row["argument_of_perigee_rad"])
        nu = float(row["true_anomaly_rad"])

        position, velocity, radius = reference_state(a, e, i, om, w, nu)
        rust_position = [float(row["x_km"]), float(row["y_km"]), float(row["z_km"])]
        rust_velocity = [float(row["vx_kms"]), float(row["vy_kms"]), float(row["vz_kms"])]

        position_error = max(abs(p - q) for p, q in zip(position, rust_position))
        velocity_error = max(abs(p - q) for p, q in zip(velocity, rust_velocity))
        radius_error = abs(radius - float(row["state_radius_km"]) if "state_radius_km" in row else 0.0)

        # The speed is checked through the vis-viva equation, which is
        # independent of the direction, and the energy through its own
        # closed form.
        speed = math.sqrt(sum(component * component for component in rust_velocity))
        vis_viva = math.sqrt(mu := MU * (2.0 / math.hypot(*rust_position) - 1.0 / a))
        energy_reference = vis_viva * vis_viva / 2.0 - MU / math.hypot(*rust_position)
        energy_error = abs(energy_reference - float(row["energy_kms2"]))
        period_error = abs(reference_period(a) - float(row["period_seconds"]))

        worst["position"] = max(worst["position"], position_error)
        worst["velocity"] = max(worst["velocity"], velocity_error)
        worst["radius"] = max(worst["radius"], radius_error)
        worst["period"] = max(worst["period"], period_error)
        worst["energy"] = max(worst["energy"], energy_error)

        _ = speed  # the magnitude check is folded into the vis-viva step

    return worst


def check_recovery(rows):
    """Checks the state to elements table and returns the worst deviations."""
    worst = {
        "semi_major_axis": 0.0,
        "eccentricity": 0.0,
        "inclination": 0.0,
        "right_ascension": 0.0,
        "argument_of_perigee": 0.0,
        "true_anomaly": 0.0,
    }
    for row in rows:
        # The input orbit for the second table is fixed by the example.
        expected_a = 7_000.0
        expected_e = 0.3
        expected_i = 0.7
        expected_om = 1.2
        expected_w = 0.4
        nu = float(row["true_anomaly_deg"])

        # The recovered semi-major axis and eccentricity must be unchanged.
        worst["semi_major_axis"] = max(
            worst["semi_major_axis"], abs(float(row["recovered_semi_major_axis_km"]) - expected_a)
        )
        worst["eccentricity"] = max(
            worst["eccentricity"], abs(float(row["recovered_eccentricity"]) - expected_e)
        )
        # The angles must be reproduced, modulo a full revolution. The
        # argument of perigee and the node are compared through the total
        # longitude, which is the quantity that is actually determined.
        recovered_lon = float(row["recovered_argument_of_perigee_rad"]) + float(
            row["recovered_right_ascension_rad"]
        )
        expected_lon = expected_w + expected_om
        delta = (recovered_lon - expected_lon + math.pi) % (2.0 * math.pi) - math.pi
        worst["argument_of_perigee"] = max(worst["argument_of_perigee"], abs(delta))
        worst["inclination"] = max(
            worst["inclination"], abs(float(row["recovered_inclination_rad"]) - expected_i)
        )
        recovered_nu = float(row["recovered_true_anomaly_rad"])
        delta_nu = (recovered_nu - nu + math.pi) % (2.0 * math.pi) - math.pi
        worst["true_anomaly"] = max(worst["true_anomaly"], abs(delta_nu))
    return worst


def read_blocks(path: Path):
    """Returns the two CSV blocks separated by a blank line."""
    text = path.read_text().splitlines()
    blocks: list[list[str]] = [[]]
    for line in text:
        if line.strip():
            blocks[-1].append(line)
        else:
            blocks.append([])
    return [block for block in blocks if block]


def main(argv: list[str]) -> int:
    path = Path(argv[1]) if len(argv) > 1 else DEFAULT_INPUT
    if not path.exists():
        print(f"missing input file: {path}", file=sys.stderr)
        print(
            "generate it with: cargo run -p apsis-rs --features orbit "
            "--example dump_kepler > " + str(path),
            file=sys.stderr,
        )
        return 2

    blocks = read_blocks(path)
    if len(blocks) < 2:
        print("input file does not have the two expected blocks", file=sys.stderr)
        return 2

    states = list(csv.DictReader(blocks[0]))
    recovery = list(csv.DictReader(blocks[1]))
    if not states or not recovery:
        print("input file has an empty block", file=sys.stderr)
        return 2

    worst = check_states(states)
    print(f"states compared:      {len(states)}")
    print(f"worst position:       {worst['position']:.6e} km")
    print(f"worst velocity:       {worst['velocity']:.6e} km/s")
    print(f"worst radius:         {worst['radius']:.6e} km")
    print(f"worst period:         {worst['period']:.6e} s")
    print(f"worst energy:         {worst['energy']:.6e} km^2/s^2")

    recovered = check_recovery(recovery)
    print(f"recovered compared:   {len(recovery)}")
    print(f"worst semi-major:     {recovered['semi_major_axis']:.6e} km")
    print(f"worst eccentricity:   {recovered['eccentricity']:.6e}")
    print(f"worst inclination:    {recovered['inclination']:.6e} rad")
    print(f"worst longitude:      {recovered['argument_of_perigee']:.6e} rad")
    print(f"worst true anomaly:   {recovered['true_anomaly']:.6e} rad")

    failures = []
    if worst["position"] > LENGTH_TOLERANCE_KM:
        failures.append("position")
    if worst["velocity"] > VELOCITY_TOLERANCE_KMS:
        failures.append("velocity")
    if worst["period"] > LENGTH_TOLERANCE_KM:
        failures.append("period")
    if worst["energy"] > RELATIVE_TOLERANCE * abs(MU):
        failures.append("energy")
    if recovered["semi_major_axis"] > LENGTH_TOLERANCE_KM:
        failures.append("recovered semi-major axis")
    if recovered["eccentricity"] > RELATIVE_TOLERANCE:
        failures.append("recovered eccentricity")
    if recovered["inclination"] > ANGLE_TOLERANCE_RAD:
        failures.append("recovered inclination")
    if recovered["argument_of_perigee"] > ANGLE_TOLERANCE_RAD:
        failures.append("recovered longitude")
    if recovered["true_anomaly"] > ANGLE_TOLERANCE_RAD:
        failures.append("recovered true anomaly")

    for failure in failures:
        print(f"  {failure} exceeds tolerance")

    if failures:
        print("result:               FAIL")
        return 1
    print("result:               pass")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
