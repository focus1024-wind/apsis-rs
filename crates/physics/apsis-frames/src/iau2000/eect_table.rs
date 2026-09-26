//! The IAU 2000 complementary terms of the equation of the equinoxes.
//!
//! The table is generated from the reference implementation by
//! `validation/tools/generate_eect_table.py` and is reproduced without
//! change.
//!
//! Source: the MHB2000 model, as distributed with the IAU SOFA library
//! and implemented by ERFA.

use super::ComplementaryTerm;

/// The terms that do not depend on time.
///
/// The eight multipliers apply to the arguments `l`, `l'`, `F`, `D`, `Om`,
/// the mean longitude of Venus, the mean longitude of the Earth, and the
/// general accumulated precession, in that order.
pub static CONSTANT_TERMS: [ComplementaryTerm; 33] = [
    ComplementaryTerm {
        argument: [0, 0, 0, 0, 1, 0, 0, 0],
        coefficients: [2640.96e-6, -0.39e-6],
    },
    ComplementaryTerm {
        argument: [0, 0, 0, 0, 2, 0, 0, 0],
        coefficients: [63.52e-6, -0.02e-6],
    },
    ComplementaryTerm {
        argument: [0, 0, 2, -2, 3, 0, 0, 0],
        coefficients: [11.75e-6, 0.01e-6],
    },
    ComplementaryTerm {
        argument: [0, 0, 2, -2, 1, 0, 0, 0],
        coefficients: [11.21e-6, 0.01e-6],
    },
    ComplementaryTerm {
        argument: [0, 0, 2, -2, 2, 0, 0, 0],
        coefficients: [-4.55e-6, 0.00e-6],
    },
    ComplementaryTerm {
        argument: [0, 0, 2, 0, 3, 0, 0, 0],
        coefficients: [2.02e-6, 0.00e-6],
    },
    ComplementaryTerm {
        argument: [0, 0, 2, 0, 1, 0, 0, 0],
        coefficients: [1.98e-6, 0.00e-6],
    },
    ComplementaryTerm {
        argument: [0, 0, 0, 0, 3, 0, 0, 0],
        coefficients: [-1.72e-6, 0.00e-6],
    },
    ComplementaryTerm {
        argument: [0, 1, 0, 0, 1, 0, 0, 0],
        coefficients: [-1.41e-6, -0.01e-6],
    },
    ComplementaryTerm {
        argument: [0, 1, 0, 0, -1, 0, 0, 0],
        coefficients: [-1.26e-6, -0.01e-6],
    },
    ComplementaryTerm {
        argument: [1, 0, 0, 0, -1, 0, 0, 0],
        coefficients: [-0.63e-6, 0.00e-6],
    },
    ComplementaryTerm {
        argument: [1, 0, 0, 0, 1, 0, 0, 0],
        coefficients: [-0.63e-6, 0.00e-6],
    },
    ComplementaryTerm {
        argument: [0, 1, 2, -2, 3, 0, 0, 0],
        coefficients: [0.46e-6, 0.00e-6],
    },
    ComplementaryTerm {
        argument: [0, 1, 2, -2, 1, 0, 0, 0],
        coefficients: [0.45e-6, 0.00e-6],
    },
    ComplementaryTerm {
        argument: [0, 0, 4, -4, 4, 0, 0, 0],
        coefficients: [0.36e-6, 0.00e-6],
    },
    ComplementaryTerm {
        argument: [0, 0, 1, -1, 1, -8, 12, 0],
        coefficients: [-0.24e-6, -0.12e-6],
    },
    ComplementaryTerm {
        argument: [0, 0, 2, 0, 0, 0, 0, 0],
        coefficients: [0.32e-6, 0.00e-6],
    },
    ComplementaryTerm {
        argument: [0, 0, 2, 0, 2, 0, 0, 0],
        coefficients: [0.28e-6, 0.00e-6],
    },
    ComplementaryTerm {
        argument: [1, 0, 2, 0, 3, 0, 0, 0],
        coefficients: [0.27e-6, 0.00e-6],
    },
    ComplementaryTerm {
        argument: [1, 0, 2, 0, 1, 0, 0, 0],
        coefficients: [0.26e-6, 0.00e-6],
    },
    ComplementaryTerm {
        argument: [0, 0, 2, -2, 0, 0, 0, 0],
        coefficients: [-0.21e-6, 0.00e-6],
    },
    ComplementaryTerm {
        argument: [0, 1, -2, 2, -3, 0, 0, 0],
        coefficients: [0.19e-6, 0.00e-6],
    },
    ComplementaryTerm {
        argument: [0, 1, -2, 2, -1, 0, 0, 0],
        coefficients: [0.18e-6, 0.00e-6],
    },
    ComplementaryTerm {
        argument: [0, 0, 0, 0, 0, 8, -13, -1],
        coefficients: [-0.10e-6, 0.05e-6],
    },
    ComplementaryTerm {
        argument: [0, 0, 0, 2, 0, 0, 0, 0],
        coefficients: [0.15e-6, 0.00e-6],
    },
    ComplementaryTerm {
        argument: [2, 0, -2, 0, -1, 0, 0, 0],
        coefficients: [-0.14e-6, 0.00e-6],
    },
    ComplementaryTerm {
        argument: [1, 0, 0, -2, 1, 0, 0, 0],
        coefficients: [0.14e-6, 0.00e-6],
    },
    ComplementaryTerm {
        argument: [0, 1, 2, -2, 2, 0, 0, 0],
        coefficients: [-0.14e-6, 0.00e-6],
    },
    ComplementaryTerm {
        argument: [1, 0, 0, -2, -1, 0, 0, 0],
        coefficients: [0.14e-6, 0.00e-6],
    },
    ComplementaryTerm {
        argument: [0, 0, 4, -2, 4, 0, 0, 0],
        coefficients: [0.13e-6, 0.00e-6],
    },
    ComplementaryTerm {
        argument: [0, 0, 2, -2, 4, 0, 0, 0],
        coefficients: [-0.11e-6, 0.00e-6],
    },
    ComplementaryTerm {
        argument: [1, 0, -2, 0, -3, 0, 0, 0],
        coefficients: [0.11e-6, 0.00e-6],
    },
    ComplementaryTerm {
        argument: [1, 0, -2, 0, -1, 0, 0, 0],
        coefficients: [0.11e-6, 0.00e-6],
    },
];

/// The terms whose amplitude grows linearly with time.
///
/// The arguments are the same as for the constant terms.
pub static LINEAR_TERMS: [ComplementaryTerm; 1] = [ComplementaryTerm {
    argument: [0, 0, 0, 0, 1, 0, 0, 0],
    coefficients: [-0.87e-6, 0.00e-6],
}];
