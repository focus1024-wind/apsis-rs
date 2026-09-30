//! Writes the element fields this workspace parses from a two-line set.
//!
//! The output is CSV on standard output, with a header row. It is consumed
//! by `validation/python/orbit/check_tle.py`, which reparses the same sets
//! with an independent, column-indexed reader and compares every field, and
//! recomputes the modulo ten checksum of each line.
//!
//! The sets include the historical test cases distributed with the SGP4
//! reference implementation plus synthetic ones that exercise the corners of
//! the format: a circular orbit with an implied leading zero in the
//! eccentricity, a retrograde inclination, a negative drag term, and a century
//! rollover in the two digit epoch year.
//!
//! Usage:
//!
//! ```text
//! cargo run -p apsis-rs --features orbit --example dump_tle \
//!     > validation/data/expected/rust_tle.csv
//! ```

use apsis_time::TimeScale;
use apsis_tle::{Classification, TwoLineElement, line_checksum};

/// One element set to parse, with a name for the report.
struct Sample {
    /// A name for the sample, so a failure in the report is readable.
    name: &'static str,
    /// The two lines of the set, without a trailing carriage return.
    lines: [&'static str; 2],
}

const fn sample(name: &'static str, first: &'static str, second: &'static str) -> Sample {
    Sample {
        name,
        lines: [first, second],
    }
}

/// The element sets parsed.
///
/// The first four are the historical test cases distributed with the SGP4
/// reference implementation. The rest are synthetic and cover the edges of
/// the format: an implied leading zero in the eccentricity, a retrograde
/// inclination, a negative first derivative of the mean motion, a century
/// rollover in the two digit epoch year, and a name line.
const SAMPLES: &[Sample] = &[
    sample(
        "vanguard_1",
        "1 00005U 58002B   00179.78495062  .00000023  00000-0  28098-4 0  4753",
        "2 00005  34.2682 348.7242 1859667 331.7664  19.3264 10.82419157413667",
    ),
    sample(
        "iss_2008",
        "1 25544U 98067A   08264.51782528 -.00002182  00000-0 -11606-4 0  2927",
        "2 25544  51.6416 247.4627 0006703 130.5360 325.0288 15.72125391563537",
    ),
    sample(
        "teme_94_deep",
        "1 27651U 03004A   06185.99374200  .00000109  00000-0  19909-4 0   445",
        "2 27651   7.1743 182.9443 7867251 269.0386  18.6601  1.03461646  9884",
    ),
    sample(
        "lrs_4_deep",
        "1 09880U 77021A   06174.85710247  .00000090  00000-0  10000-3 0  7322",
        "2 09880  64.1526 313.1296 6546925 103.4683 305.3980  2.00754276 13047",
    ),
    sample(
        "circular_implied_zero",
        "1 20500U 90001A   90001.00000000  .00000000  00000-0  00000-0 0    00",
        "2 20500  82.0000 100.0000 0000000  90.0000  90.0000 14.90000000    13",
    ),
    sample(
        "retrograde",
        "1 20600U 90002B   12345.50000000 -.00001234  00000-0  12345-6 0  9991",
        "2 20600 120.0000 200.0000 0010000  30.0000 330.0000 15.00000000 12341",
    ),
    sample(
        // A two digit year of 99 must expand to 1999, the last year of the
        // twentieth century branch of the convention.
        "two_digit_year_1999",
        "1 10000U 99001A   99365.00000000  .00000000  00000-0  00000-0 0    05",
        "2 10000  45.0000  10.0000 0001000   0.0000   0.0000 14.00000000    09",
    ),
    sample(
        "with_name_line",
        "1 12345U 80001C   20001.00000000  .00010000  00000-0  00000-0 0  9998",
        "2 12345  60.0000  30.0000 0002000 100.0000 200.0000 12.00000000 12344",
    ),
];

/// Returns the day of the year for a calendar date, as a fraction.
///
/// The epoch field of an element set stores the day of the year and its
/// fraction, so the year and month are combined back into a day of year
/// before the fraction of the day is added.
fn day_of_year_fraction(year: i32, month: u32, day: u32) -> f64 {
    use apsis_time::days_in_month;
    let mut total = 0_u32;
    for current in 1..month {
        total += days_in_month(year, current);
    }
    f64::from(total + day - 1)
}

fn main() {
    println!(
        "name,satellite_number,classification,international_designator,\
         epoch_year,epoch_month,epoch_day,epoch_fraction_day,e_mean_motion_dot,\
         e_mean_motion_double_dot,bstar,element_set_number,inclination_deg,\
         right_ascension_deg,eccentricity,argument_of_perigee_deg,mean_anomaly_deg,\
         mean_motion,revolution_number,line1_checksum,line2_checksum"
    );

    for entry in SAMPLES {
        let text = format!("{}\r\n{}\r\n", entry.lines[0], entry.lines[1]);
        let element = TwoLineElement::parse(&text)
            .unwrap_or_else(|error| panic!("{} does not parse: {error}", entry.name));
        let classification = match element.classification {
            Classification::Unclassified => "U".to_string(),
            Classification::Secret => "S".to_string(),
            Classification::Other(letter) => letter.to_string(),
        };
        // The epoch is stored as a Modified Julian Date; the calendar form
        // is written so the reader can compare the year and the day of year
        // against the two digit year and fraction the line encodes.
        let calendar = element
            .epoch
            .calendar(TimeScale::Utc)
            .expect("the TLE epoch is inside the leap second table");
        let fraction_day = day_of_year_fraction(calendar.year, calendar.month, calendar.day)
            + (f64::from(calendar.hour) * 3600.0
                + f64::from(calendar.minute) * 60.0
                + calendar.second)
                / 86_400.0;
        let fields = [
            entry.name.to_string(),
            element.satellite_number.to_string(),
            classification,
            element.international_designator.clone(),
            calendar.year.to_string(),
            calendar.month.to_string(),
            calendar.day.to_string(),
            format!("{fraction_day:.12}"),
            format!("{:.12e}", element.mean_motion_dot),
            format!("{:.12e}", element.mean_motion_double_dot),
            format!("{:.12e}", element.bstar),
            element.element_set_number.to_string(),
            format!("{:.12}", element.inclination),
            format!("{:.12}", element.right_ascension),
            format!("{:.12}", element.eccentricity),
            format!("{:.12}", element.argument_of_perigee),
            format!("{:.12}", element.mean_anomaly),
            format!("{:.12}", element.mean_motion),
            element.revolution_number.to_string(),
            line_checksum(entry.lines[0]).to_string(),
            line_checksum(entry.lines[1]).to_string(),
        ];
        println!("{}", fields.join(","));
    }
}
