//! Specific attenuation of atmospheric gases.
//!
//! The attenuation caused by oxygen and water vapour is computed by the
//! line-by-line summation of ITU-R Recommendation P.676. Each absorption
//! line contributes a resonance whose strength depends on the temperature
//! and on the pressure through the line width and the pressure shift.
//!
//! The model is parameterised by four atmospheric quantities:
//!
//! - The dry air pressure, in hectopascals.
//! - The water vapour density, in grams per cubic meter.
//! - The temperature, in kelvin.
//! - The frequency, in gigahertz.
//!
//! The pressure is the dry air pressure, not the total barometric pressure.
//! The partial pressure of water vapour follows from the density and the
//! temperature, and the two are used separately in the line widths.
//!
//! Source: ITU-R Recommendation P.676-12, *Attenuation by atmospheric
//! gases*, sections 1 and 2. The line tables are the ones tabulated in
//! tables 1 and 2 of that recommendation, and the expressions for the line
//! widths and the strengths are equations 1 through 8.

use core::f64::consts::E;

/// One oxygen absorption line.
///
/// The coefficients are the ones tabulated in table 1 of ITU-R P.676-12. The
/// first field is the line frequency in gigahertz and the rest are the
/// dimensionless coefficients of the line.
#[derive(Debug, Clone, Copy)]
struct OxygenLine {
    /// The line frequency, in gigahertz.
    f0: f64,
    /// Line strength coefficient.
    a1: f64,
    /// Temperature dependence of the line strength.
    a2: f64,
    /// Line width coefficient.
    a3: f64,
    /// Pressure dependence of the line width.
    a4: f64,
    /// Pressure shift coefficient.
    a5: f64,
    /// Mixing ratio dependence of the pressure shift.
    a6: f64,
}

/// One water vapour absorption line.
///
/// The coefficients are the ones tabulated in table 2 of ITU-R P.676-12.
#[derive(Debug, Clone, Copy)]
struct WaterLine {
    /// The line frequency, in gigahertz.
    f0: f64,
    /// Line strength coefficient.
    b1: f64,
    /// Temperature dependence of the line strength.
    b2: f64,
    /// Line width coefficient.
    b3: f64,
    /// Pressure dependence of the line width.
    b4: f64,
    /// Water vapour dependence of the line width.
    b5: f64,
    /// Temperature dependence of the water vapour term.
    b6: f64,
}

/// The oxygen absorption lines of table 1 of ITU-R P.676-12.
///
/// The first thirty seven entries are the fine structure of the 60 GHz
/// complex, followed by the isolated line at 118.75 GHz and six lines in the
/// submillimetre region.
const OXYGEN_LINES: &[OxygenLine] = &[
    OxygenLine {
        f0: 50.474_214,
        a1: 0.975_000,
        a2: 9.651_000,
        a3: 6.690_000,
        a4: 0.000_000,
        a5: 2.566_000,
        a6: 6.850_000,
    },
    OxygenLine {
        f0: 50.987_745,
        a1: 2.529_000,
        a2: 8.653_000,
        a3: 7.170_000,
        a4: 0.000_000,
        a5: 2.246_000,
        a6: 6.800_000,
    },
    OxygenLine {
        f0: 51.503_360,
        a1: 6.193_000,
        a2: 7.709_000,
        a3: 7.640_000,
        a4: 0.000_000,
        a5: 1.947_000,
        a6: 6.729_000,
    },
    OxygenLine {
        f0: 52.021_429,
        a1: 14.320_000,
        a2: 6.819_000,
        a3: 8.110_000,
        a4: 0.000_000,
        a5: 1.667_000,
        a6: 6.640_000,
    },
    OxygenLine {
        f0: 52.542_418,
        a1: 31.240_000,
        a2: 5.983_000,
        a3: 8.580_000,
        a4: 0.000_000,
        a5: 1.388_000,
        a6: 6.526_000,
    },
    OxygenLine {
        f0: 53.066_934,
        a1: 64.290_000,
        a2: 5.201_000,
        a3: 9.060_000,
        a4: 0.000_000,
        a5: 1.349_000,
        a6: 6.206_000,
    },
    OxygenLine {
        f0: 53.595_775,
        a1: 124.600_000,
        a2: 4.474_000,
        a3: 9.550_000,
        a4: 0.000_000,
        a5: 2.227_000,
        a6: 5.085_000,
    },
    OxygenLine {
        f0: 54.130_025,
        a1: 227.300_000,
        a2: 3.800_000,
        a3: 9.960_000,
        a4: 0.000_000,
        a5: 3.170_000,
        a6: 3.750_000,
    },
    OxygenLine {
        f0: 54.671_180,
        a1: 389.700_000,
        a2: 3.182_000,
        a3: 10.370_000,
        a4: 0.000_000,
        a5: 3.558_000,
        a6: 2.654_000,
    },
    OxygenLine {
        f0: 55.221_384,
        a1: 627.100_000,
        a2: 2.618_000,
        a3: 10.890_000,
        a4: 0.000_000,
        a5: 2.560_000,
        a6: 2.952_000,
    },
    OxygenLine {
        f0: 55.783_815,
        a1: 945.300_000,
        a2: 2.109_000,
        a3: 11.340_000,
        a4: 0.000_000,
        a5: -1.172_000,
        a6: 6.135_000,
    },
    OxygenLine {
        f0: 56.264_774,
        a1: 543.400_000,
        a2: 0.014_000,
        a3: 17.030_000,
        a4: 0.000_000,
        a5: 3.525_000,
        a6: -0.978_000,
    },
    OxygenLine {
        f0: 56.363_399,
        a1: 1_331.800_000,
        a2: 1.654_000,
        a3: 11.890_000,
        a4: 0.000_000,
        a5: -2.378_000,
        a6: 6.547_000,
    },
    OxygenLine {
        f0: 56.968_211,
        a1: 1_746.600_000,
        a2: 1.255_000,
        a3: 12.230_000,
        a4: 0.000_000,
        a5: -3.545_000,
        a6: 6.451_000,
    },
    OxygenLine {
        f0: 57.612_486,
        a1: 2_120.100_000,
        a2: 0.910_000,
        a3: 12.620_000,
        a4: 0.000_000,
        a5: -5.416_000,
        a6: 6.056_000,
    },
    OxygenLine {
        f0: 58.323_877,
        a1: 2_363.700_000,
        a2: 0.621_000,
        a3: 12.950_000,
        a4: 0.000_000,
        a5: -1.932_000,
        a6: 0.436_000,
    },
    OxygenLine {
        f0: 58.446_588,
        a1: 1_442.100_000,
        a2: 0.083_000,
        a3: 14.910_000,
        a4: 0.000_000,
        a5: 6.768_000,
        a6: -1.273_000,
    },
    OxygenLine {
        f0: 59.164_204,
        a1: 2_379.900_000,
        a2: 0.387_000,
        a3: 13.530_000,
        a4: 0.000_000,
        a5: -6.561_000,
        a6: 2.309_000,
    },
    OxygenLine {
        f0: 59.590_983,
        a1: 2_090.700_000,
        a2: 0.207_000,
        a3: 14.080_000,
        a4: 0.000_000,
        a5: 6.957_000,
        a6: -0.776_000,
    },
    OxygenLine {
        f0: 60.306_056,
        a1: 2_103.400_000,
        a2: 0.207_000,
        a3: 14.150_000,
        a4: 0.000_000,
        a5: -6.395_000,
        a6: 0.699_000,
    },
    OxygenLine {
        f0: 60.434_778,
        a1: 2_438.000_000,
        a2: 0.386_000,
        a3: 13.390_000,
        a4: 0.000_000,
        a5: 6.342_000,
        a6: -2.825_000,
    },
    OxygenLine {
        f0: 61.150_562,
        a1: 2_479.500_000,
        a2: 0.621_000,
        a3: 12.920_000,
        a4: 0.000_000,
        a5: 1.014_000,
        a6: -0.584_000,
    },
    OxygenLine {
        f0: 61.800_158,
        a1: 2_275.900_000,
        a2: 0.910_000,
        a3: 12.630_000,
        a4: 0.000_000,
        a5: 5.014_000,
        a6: -6.619_000,
    },
    OxygenLine {
        f0: 62.411_220,
        a1: 1_915.400_000,
        a2: 1.255_000,
        a3: 12.170_000,
        a4: 0.000_000,
        a5: 3.029_000,
        a6: -6.759_000,
    },
    OxygenLine {
        f0: 62.486_253,
        a1: 1_503.000_000,
        a2: 0.083_000,
        a3: 15.130_000,
        a4: 0.000_000,
        a5: -4.499_000,
        a6: 0.844_000,
    },
    OxygenLine {
        f0: 62.997_984,
        a1: 1_490.200_000,
        a2: 1.654_000,
        a3: 11.740_000,
        a4: 0.000_000,
        a5: 1.856_000,
        a6: -6.675_000,
    },
    OxygenLine {
        f0: 63.568_526,
        a1: 1_078.000_000,
        a2: 2.108_000,
        a3: 11.340_000,
        a4: 0.000_000,
        a5: 0.658_000,
        a6: -6.139_000,
    },
    OxygenLine {
        f0: 64.127_775,
        a1: 728.700_000,
        a2: 2.617_000,
        a3: 10.880_000,
        a4: 0.000_000,
        a5: -3.036_000,
        a6: -2.895_000,
    },
    OxygenLine {
        f0: 64.678_910,
        a1: 461.300_000,
        a2: 3.181_000,
        a3: 10.380_000,
        a4: 0.000_000,
        a5: -3.968_000,
        a6: -2.590_000,
    },
    OxygenLine {
        f0: 65.224_078,
        a1: 274.000_000,
        a2: 3.800_000,
        a3: 9.960_000,
        a4: 0.000_000,
        a5: -3.528_000,
        a6: -3.680_000,
    },
    OxygenLine {
        f0: 65.764_779,
        a1: 153.000_000,
        a2: 4.473_000,
        a3: 9.550_000,
        a4: 0.000_000,
        a5: -2.548_000,
        a6: -5.002_000,
    },
    OxygenLine {
        f0: 66.302_096,
        a1: 80.400_000,
        a2: 5.200_000,
        a3: 9.060_000,
        a4: 0.000_000,
        a5: -1.660_000,
        a6: -6.091_000,
    },
    OxygenLine {
        f0: 66.836_834,
        a1: 39.800_000,
        a2: 5.982_000,
        a3: 8.580_000,
        a4: 0.000_000,
        a5: -1.680_000,
        a6: -6.393_000,
    },
    OxygenLine {
        f0: 67.369_601,
        a1: 18.560_000,
        a2: 6.818_000,
        a3: 8.110_000,
        a4: 0.000_000,
        a5: -1.956_000,
        a6: -6.475_000,
    },
    OxygenLine {
        f0: 67.900_868,
        a1: 8.172_000,
        a2: 7.708_000,
        a3: 7.640_000,
        a4: 0.000_000,
        a5: -2.216_000,
        a6: -6.545_000,
    },
    OxygenLine {
        f0: 68.431_006,
        a1: 3.397_000,
        a2: 8.652_000,
        a3: 7.170_000,
        a4: 0.000_000,
        a5: -2.492_000,
        a6: -6.600_000,
    },
    OxygenLine {
        f0: 68.960_312,
        a1: 1.334_000,
        a2: 9.650_000,
        a3: 6.690_000,
        a4: 0.000_000,
        a5: -2.773_000,
        a6: -6.650_000,
    },
    OxygenLine {
        f0: 118.750_334,
        a1: 940.300_000,
        a2: 0.010_000,
        a3: 16.640_000,
        a4: 0.000_000,
        a5: -0.439_000,
        a6: 0.079_000,
    },
    OxygenLine {
        f0: 368.498_246,
        a1: 67.400_000,
        a2: 0.048_000,
        a3: 16.400_000,
        a4: 0.000_000,
        a5: 0.000_000,
        a6: 0.000_000,
    },
    OxygenLine {
        f0: 424.763_020,
        a1: 637.700_000,
        a2: 0.044_000,
        a3: 16.400_000,
        a4: 0.000_000,
        a5: 0.000_000,
        a6: 0.000_000,
    },
    OxygenLine {
        f0: 487.249_273,
        a1: 237.400_000,
        a2: 0.049_000,
        a3: 16.000_000,
        a4: 0.000_000,
        a5: 0.000_000,
        a6: 0.000_000,
    },
    OxygenLine {
        f0: 715.392_902,
        a1: 98.100_000,
        a2: 0.145_000,
        a3: 16.000_000,
        a4: 0.000_000,
        a5: 0.000_000,
        a6: 0.000_000,
    },
    OxygenLine {
        f0: 773.839_490,
        a1: 572.300_000,
        a2: 0.141_000,
        a3: 16.200_000,
        a4: 0.000_000,
        a5: 0.000_000,
        a6: 0.000_000,
    },
    OxygenLine {
        f0: 834.145_546,
        a1: 183.100_000,
        a2: 0.145_000,
        a3: 14.700_000,
        a4: 0.000_000,
        a5: 0.000_000,
        a6: 0.000_000,
    },
];

/// The water vapour absorption lines of table 2 of ITU-R P.676-12.
const WATER_LINES: &[WaterLine] = &[
    WaterLine {
        f0: 22.235_080,
        b1: 0.107_900,
        b2: 2.144_000,
        b3: 26.380_000,
        b4: 0.760_000,
        b5: 5.087_000,
        b6: 1.000_000,
    },
    WaterLine {
        f0: 67.803_960,
        b1: 0.001_100,
        b2: 8.732_000,
        b3: 28.580_000,
        b4: 0.690_000,
        b5: 4.930_000,
        b6: 0.820_000,
    },
    WaterLine {
        f0: 119.995_940,
        b1: 0.000_700,
        b2: 8.353_000,
        b3: 29.480_000,
        b4: 0.700_000,
        b5: 4.780_000,
        b6: 0.790_000,
    },
    WaterLine {
        f0: 183.310_087,
        b1: 2.273_000,
        b2: 0.668_000,
        b3: 29.060_000,
        b4: 0.770_000,
        b5: 5.022_000,
        b6: 0.850_000,
    },
    WaterLine {
        f0: 321.225_630,
        b1: 0.047_000,
        b2: 6.179_000,
        b3: 24.040_000,
        b4: 0.670_000,
        b5: 4.398_000,
        b6: 0.540_000,
    },
    WaterLine {
        f0: 325.152_888,
        b1: 1.514_000,
        b2: 1.541_000,
        b3: 28.230_000,
        b4: 0.640_000,
        b5: 4.893_000,
        b6: 0.740_000,
    },
    WaterLine {
        f0: 336.227_764,
        b1: 0.001_000,
        b2: 9.825_000,
        b3: 26.930_000,
        b4: 0.690_000,
        b5: 4.740_000,
        b6: 0.610_000,
    },
    WaterLine {
        f0: 380.197_353,
        b1: 11.670_000,
        b2: 1.048_000,
        b3: 28.110_000,
        b4: 0.540_000,
        b5: 5.063_000,
        b6: 0.890_000,
    },
    WaterLine {
        f0: 390.134_508,
        b1: 0.004_500,
        b2: 7.347_000,
        b3: 21.520_000,
        b4: 0.630_000,
        b5: 4.810_000,
        b6: 0.550_000,
    },
    WaterLine {
        f0: 437.346_667,
        b1: 0.063_200,
        b2: 5.048_000,
        b3: 18.450_000,
        b4: 0.600_000,
        b5: 4.230_000,
        b6: 0.480_000,
    },
    WaterLine {
        f0: 439.150_807,
        b1: 0.909_800,
        b2: 3.595_000,
        b3: 20.070_000,
        b4: 0.630_000,
        b5: 4.483_000,
        b6: 0.520_000,
    },
    WaterLine {
        f0: 443.018_343,
        b1: 0.192_000,
        b2: 5.048_000,
        b3: 15.550_000,
        b4: 0.600_000,
        b5: 5.083_000,
        b6: 0.500_000,
    },
    WaterLine {
        f0: 448.001_085,
        b1: 10.410_000,
        b2: 1.405_000,
        b3: 25.640_000,
        b4: 0.660_000,
        b5: 5.028_000,
        b6: 0.670_000,
    },
    WaterLine {
        f0: 470.888_999,
        b1: 0.325_400,
        b2: 3.597_000,
        b3: 21.340_000,
        b4: 0.660_000,
        b5: 4.506_000,
        b6: 0.650_000,
    },
    WaterLine {
        f0: 474.689_092,
        b1: 1.260_000,
        b2: 2.379_000,
        b3: 23.200_000,
        b4: 0.650_000,
        b5: 4.804_000,
        b6: 0.640_000,
    },
    WaterLine {
        f0: 488.490_108,
        b1: 0.252_900,
        b2: 2.852_000,
        b3: 25.860_000,
        b4: 0.690_000,
        b5: 5.201_000,
        b6: 0.720_000,
    },
    WaterLine {
        f0: 503.568_532,
        b1: 0.037_200,
        b2: 6.731_000,
        b3: 16.120_000,
        b4: 0.610_000,
        b5: 3.980_000,
        b6: 0.430_000,
    },
    WaterLine {
        f0: 504.482_692,
        b1: 0.012_400,
        b2: 6.731_000,
        b3: 16.120_000,
        b4: 0.610_000,
        b5: 4.010_000,
        b6: 0.450_000,
    },
    WaterLine {
        f0: 547.676_440,
        b1: 0.978_500,
        b2: 0.158_000,
        b3: 26.000_000,
        b4: 0.700_000,
        b5: 4.500_000,
        b6: 1.000_000,
    },
    WaterLine {
        f0: 552.020_960,
        b1: 0.184_000,
        b2: 0.158_000,
        b3: 26.000_000,
        b4: 0.700_000,
        b5: 4.500_000,
        b6: 1.000_000,
    },
    WaterLine {
        f0: 556.935_985,
        b1: 497.000_000,
        b2: 0.159_000,
        b3: 30.860_000,
        b4: 0.690_000,
        b5: 4.552_000,
        b6: 1.000_000,
    },
    WaterLine {
        f0: 620.700_807,
        b1: 5.015_000,
        b2: 2.391_000,
        b3: 24.380_000,
        b4: 0.710_000,
        b5: 4.856_000,
        b6: 0.680_000,
    },
    WaterLine {
        f0: 645.766_085,
        b1: 0.006_700,
        b2: 8.633_000,
        b3: 18.000_000,
        b4: 0.600_000,
        b5: 4.000_000,
        b6: 0.500_000,
    },
    WaterLine {
        f0: 658.005_280,
        b1: 0.273_200,
        b2: 7.816_000,
        b3: 32.100_000,
        b4: 0.690_000,
        b5: 4.140_000,
        b6: 1.000_000,
    },
    WaterLine {
        f0: 752.033_113,
        b1: 243.400_000,
        b2: 0.396_000,
        b3: 30.860_000,
        b4: 0.680_000,
        b5: 4.352_000,
        b6: 0.840_000,
    },
    WaterLine {
        f0: 841.051_732,
        b1: 0.013_400,
        b2: 8.177_000,
        b3: 15.900_000,
        b4: 0.330_000,
        b5: 5.760_000,
        b6: 0.450_000,
    },
    WaterLine {
        f0: 859.965_698,
        b1: 0.132_500,
        b2: 8.055_000,
        b3: 30.600_000,
        b4: 0.680_000,
        b5: 4.090_000,
        b6: 0.840_000,
    },
    WaterLine {
        f0: 899.303_175,
        b1: 0.054_700,
        b2: 7.914_000,
        b3: 29.850_000,
        b4: 0.680_000,
        b5: 4.530_000,
        b6: 0.900_000,
    },
    WaterLine {
        f0: 902.611_085,
        b1: 0.038_600,
        b2: 8.429_000,
        b3: 28.650_000,
        b4: 0.700_000,
        b5: 5.100_000,
        b6: 0.950_000,
    },
    WaterLine {
        f0: 906.205_957,
        b1: 0.183_600,
        b2: 5.110_000,
        b3: 24.080_000,
        b4: 0.700_000,
        b5: 4.700_000,
        b6: 0.530_000,
    },
    WaterLine {
        f0: 916.171_582,
        b1: 8.400_000,
        b2: 1.441_000,
        b3: 26.730_000,
        b4: 0.700_000,
        b5: 5.150_000,
        b6: 0.780_000,
    },
    WaterLine {
        f0: 923.112_692,
        b1: 0.007_900,
        b2: 10.293_000,
        b3: 29.000_000,
        b4: 0.700_000,
        b5: 5.000_000,
        b6: 0.800_000,
    },
    WaterLine {
        f0: 970.315_022,
        b1: 9.009_000,
        b2: 1.919_000,
        b3: 25.500_000,
        b4: 0.640_000,
        b5: 4.940_000,
        b6: 0.670_000,
    },
    WaterLine {
        f0: 987.926_764,
        b1: 134.600_000,
        b2: 0.257_000,
        b3: 29.850_000,
        b4: 0.680_000,
        b5: 4.550_000,
        b6: 0.900_000,
    },
    WaterLine {
        f0: 1_780.000_000,
        b1: 17_506.000_000,
        b2: 0.952_000,
        b3: 196.300_000,
        b4: 2.000_000,
        b5: 24.150_000,
        b6: 5.000_000,
    },
];

/// The atmospheric state that the gaseous attenuation depends on.
///
/// The pressure is the dry air pressure, as the recommendation specifies. The
/// water vapour density is separate because the partial pressure of water
/// vapour is derived from it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Atmosphere {
    /// The dry air pressure, in hectopascals.
    pub pressure_hpa: f64,
    /// The water vapour density, in grams per cubic meter.
    pub water_vapour_density: f64,
    /// The temperature, in kelvin.
    pub temperature_k: f64,
}

impl Atmosphere {
    /// The ITU-R reference atmosphere at mean sea level.
    ///
    /// The values are the ones the recommendation uses for its reference
    /// curves: a dry air pressure of 1013.25 hPa, a water vapour density of
    /// 7.5 g/m^3, and a temperature of 288.15 K.
    pub const REFERENCE: Self = Self {
        pressure_hpa: 1_013.25,
        water_vapour_density: 7.5,
        temperature_k: 288.15,
    };

    /// Creates an atmosphere from its three parameters.
    pub const fn new(pressure_hpa: f64, water_vapour_density: f64, temperature_k: f64) -> Self {
        Self {
            pressure_hpa,
            water_vapour_density,
            temperature_k,
        }
    }

    /// Returns the partial pressure of water vapour, in hectopascals.
    ///
    /// The relation is `e = rho T / 216.7`, which follows from the ideal gas
    /// law with the molar masses of water and dry air.
    pub fn water_vapour_partial_pressure(self) -> f64 {
        self.water_vapour_density * self.temperature_k / 216.7
    }

    /// Returns `true` when the state describes a physical atmosphere.
    pub fn is_valid(self) -> bool {
        self.pressure_hpa.is_finite()
            && self.pressure_hpa > 0.0
            && self.water_vapour_density.is_finite()
            && self.water_vapour_density >= 0.0
            && self.temperature_k.is_finite()
            && self.temperature_k > 0.0
    }
}

/// Returns the specific attenuation of dry air, in decibels per kilometer.
///
/// This is the oxygen contribution, computed by the line-by-line sum of
/// equation 1 of ITU-R P.676-12, plus the non-resonant Debye term that
/// accounts for the far wings of all the lines together.
///
/// The frequency is in gigahertz.
///
/// 频率单位为吉赫。
///
/// Returns `None` when the frequency or atmosphere is not usable.
///
/// 频率或大气不可用时返回 `None`。
pub fn specific_attenuation_dry_air(frequency_ghz: f64, atmosphere: Atmosphere) -> Option<f64> {
    if !frequency_ghz.is_finite() || frequency_ghz <= 0.0 || !atmosphere.is_valid() {
        return None;
    }
    let theta = 300.0 / atmosphere.temperature_k;
    let e = atmosphere.water_vapour_partial_pressure();
    let p = atmosphere.pressure_hpa;

    let mut sum = 0.0;
    for line in OXYGEN_LINES {
        // Equations 5 and 6: the line width and the pressure shift.
        let width = line.a3 * 1.0e-4 * (p * theta.powf(0.8 - line.a4) + 1.1 * e * theta);
        let width = (width * width + 2.25e-6).sqrt();
        let shift = (line.a5 + line.a6 * theta) * 1.0e-4 * (p + e) * theta.powf(0.8);

        // Equation 4: the line shape factor.
        let shape = frequency_ghz / line.f0
            * ((width - shift * (line.f0 - frequency_ghz))
                / ((line.f0 - frequency_ghz).powi(2) + width * width)
                + (width - shift * (line.f0 + frequency_ghz))
                    / ((line.f0 + frequency_ghz).powi(2) + width * width));

        // Equation 3: the line strength.
        let strength = line.a1 * 1.0e-7 * p * theta.powi(3) * E.powf(line.a2 * (1.0 - theta));
        sum += strength * shape;
    }

    // The non-resonant Debye spectrum, equation 8.
    let d = 5.6e-4 * (p + e) * theta.powf(0.8);
    let debye = frequency_ghz
        * p
        * theta.powi(2)
        * (6.14e-5 / (d * (1.0 + (frequency_ghz / d).powi(2)))
            + 1.4e-12 * p * theta.powf(1.5) / (1.0 + 1.9e-5 * frequency_ghz.powf(1.5)));

    Some(0.182 * frequency_ghz * (sum + debye))
}

/// Returns the specific attenuation of water vapour, in decibels per
/// kilometer.
///
/// This is the water vapour contribution, computed by the line-by-line sum of
/// equation 1 of ITU-R P.676-12.
///
/// The frequency is in gigahertz.
///
/// 频率单位为吉赫。
///
/// Returns `None` when the frequency or atmosphere is not usable.
///
/// 频率或大气不可用时返回 `None`。
pub fn specific_attenuation_water_vapour(
    frequency_ghz: f64,
    atmosphere: Atmosphere,
) -> Option<f64> {
    if !frequency_ghz.is_finite() || frequency_ghz <= 0.0 || !atmosphere.is_valid() {
        return None;
    }
    let theta = 300.0 / atmosphere.temperature_k;
    let e = atmosphere.water_vapour_partial_pressure();
    let p = atmosphere.pressure_hpa;

    let mut sum = 0.0;
    for line in WATER_LINES {
        // Equation 7: the line width, which has a different form from the
        // oxygen case.
        let width =
            line.b3 * 1.0e-4 * (p * theta.powf(line.b4) + line.b5 * e * theta.powf(line.b6));
        let width = 0.535 * width
            + (0.217 * width * width + 2.131_6e-12 * line.f0 * line.f0 / theta).sqrt();

        let shape = frequency_ghz / line.f0
            * (width / ((line.f0 - frequency_ghz).powi(2) + width * width)
                + width / ((line.f0 + frequency_ghz).powi(2) + width * width));

        // Equation 3 for the water vapour lines.
        let strength = line.b1 * 1.0e-1 * e * theta.powf(3.5) * E.powf(line.b2 * (1.0 - theta));
        sum += strength * shape;
    }

    Some(0.182 * frequency_ghz * sum)
}

/// Returns the total specific attenuation of the atmosphere, in decibels per
/// kilometer.
///
/// This is the sum of the dry air and water vapour contributions.
///
/// Returns `None` when the frequency or atmosphere is not usable.
///
/// 频率或大气不可用时返回 `None`。
pub fn specific_attenuation(frequency_ghz: f64, atmosphere: Atmosphere) -> Option<f64> {
    Some(
        specific_attenuation_dry_air(frequency_ghz, atmosphere)?
            + specific_attenuation_water_vapour(frequency_ghz, atmosphere)?,
    )
}
