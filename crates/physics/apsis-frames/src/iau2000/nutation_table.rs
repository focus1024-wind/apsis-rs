//! The IAU 2000A nutation series.
//!
//! The table is generated from the reference implementation of the
//! model by `validation/tools/generate_nutation_table.py` and is
//! reproduced without change.
//!
//! The field order follows the reference, which is not the order the
//! coefficients are listed in the paper.
//!
//! Source: the MHB2000 model of Mathews, Herring and Buffett, adopted by
//! the International Astronomical Union in 2000 and distributed as part
//! of the IAU SOFA library, as implemented by ERFA.

use super::{PlanetaryTerm, SeriesTerm};

/// The luni-solar terms of the series.
///
/// The multipliers apply to the arguments `l`, `l'`, `F`, `D`, and `Om`
/// in that order. The coefficients are the longitude sine, its time
/// derivative, the longitude cosine, the obliquity cosine, its time
/// derivative, and the obliquity sine, in units of 0.1 microarcsecond.
pub static LUNI_SOLAR: [SeriesTerm; 678] = [
    SeriesTerm {
        argument: [0, 0, 0, 0, 1],
        coefficients: [
            -172064161.0,
            -174666.0,
            33386.0,
            92052331.0,
            9086.0,
            15377.0,
        ],
    },
    SeriesTerm {
        argument: [0, 0, 2, -2, 2],
        coefficients: [-13170906.0, -1675.0, -13696.0, 5730336.0, -3015.0, -4587.0],
    },
    SeriesTerm {
        argument: [0, 0, 2, 0, 2],
        coefficients: [-2276413.0, -234.0, 2796.0, 978459.0, -485.0, 1374.0],
    },
    SeriesTerm {
        argument: [0, 0, 0, 0, 2],
        coefficients: [2074554.0, 207.0, -698.0, -897492.0, 470.0, -291.0],
    },
    SeriesTerm {
        argument: [0, 1, 0, 0, 0],
        coefficients: [1475877.0, -3633.0, 11817.0, 73871.0, -184.0, -1924.0],
    },
    SeriesTerm {
        argument: [0, 1, 2, -2, 2],
        coefficients: [-516821.0, 1226.0, -524.0, 224386.0, -677.0, -174.0],
    },
    SeriesTerm {
        argument: [1, 0, 0, 0, 0],
        coefficients: [711159.0, 73.0, -872.0, -6750.0, 0.0, 358.0],
    },
    SeriesTerm {
        argument: [0, 0, 2, 0, 1],
        coefficients: [-387298.0, -367.0, 380.0, 200728.0, 18.0, 318.0],
    },
    SeriesTerm {
        argument: [1, 0, 2, 0, 2],
        coefficients: [-301461.0, -36.0, 816.0, 129025.0, -63.0, 367.0],
    },
    SeriesTerm {
        argument: [0, -1, 2, -2, 2],
        coefficients: [215829.0, -494.0, 111.0, -95929.0, 299.0, 132.0],
    },
    SeriesTerm {
        argument: [0, 0, 2, -2, 1],
        coefficients: [128227.0, 137.0, 181.0, -68982.0, -9.0, 39.0],
    },
    SeriesTerm {
        argument: [-1, 0, 2, 0, 2],
        coefficients: [123457.0, 11.0, 19.0, -53311.0, 32.0, -4.0],
    },
    SeriesTerm {
        argument: [-1, 0, 0, 2, 0],
        coefficients: [156994.0, 10.0, -168.0, -1235.0, 0.0, 82.0],
    },
    SeriesTerm {
        argument: [1, 0, 0, 0, 1],
        coefficients: [63110.0, 63.0, 27.0, -33228.0, 0.0, -9.0],
    },
    SeriesTerm {
        argument: [-1, 0, 0, 0, 1],
        coefficients: [-57976.0, -63.0, -189.0, 31429.0, 0.0, -75.0],
    },
    SeriesTerm {
        argument: [-1, 0, 2, 2, 2],
        coefficients: [-59641.0, -11.0, 149.0, 25543.0, -11.0, 66.0],
    },
    SeriesTerm {
        argument: [1, 0, 2, 0, 1],
        coefficients: [-51613.0, -42.0, 129.0, 26366.0, 0.0, 78.0],
    },
    SeriesTerm {
        argument: [-2, 0, 2, 0, 1],
        coefficients: [45893.0, 50.0, 31.0, -24236.0, -10.0, 20.0],
    },
    SeriesTerm {
        argument: [0, 0, 0, 2, 0],
        coefficients: [63384.0, 11.0, -150.0, -1220.0, 0.0, 29.0],
    },
    SeriesTerm {
        argument: [0, 0, 2, 2, 2],
        coefficients: [-38571.0, -1.0, 158.0, 16452.0, -11.0, 68.0],
    },
    SeriesTerm {
        argument: [0, -2, 2, -2, 2],
        coefficients: [32481.0, 0.0, 0.0, -13870.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, 0, 0, 2, 0],
        coefficients: [-47722.0, 0.0, -18.0, 477.0, 0.0, -25.0],
    },
    SeriesTerm {
        argument: [2, 0, 2, 0, 2],
        coefficients: [-31046.0, -1.0, 131.0, 13238.0, -11.0, 59.0],
    },
    SeriesTerm {
        argument: [1, 0, 2, -2, 2],
        coefficients: [28593.0, 0.0, -1.0, -12338.0, 10.0, -3.0],
    },
    SeriesTerm {
        argument: [-1, 0, 2, 0, 1],
        coefficients: [20441.0, 21.0, 10.0, -10758.0, 0.0, -3.0],
    },
    SeriesTerm {
        argument: [2, 0, 0, 0, 0],
        coefficients: [29243.0, 0.0, -74.0, -609.0, 0.0, 13.0],
    },
    SeriesTerm {
        argument: [0, 0, 2, 0, 0],
        coefficients: [25887.0, 0.0, -66.0, -550.0, 0.0, 11.0],
    },
    SeriesTerm {
        argument: [0, 1, 0, 0, 1],
        coefficients: [-14053.0, -25.0, 79.0, 8551.0, -2.0, -45.0],
    },
    SeriesTerm {
        argument: [-1, 0, 0, 2, 1],
        coefficients: [15164.0, 10.0, 11.0, -8001.0, 0.0, -1.0],
    },
    SeriesTerm {
        argument: [0, 2, 2, -2, 2],
        coefficients: [-15794.0, 72.0, -16.0, 6850.0, -42.0, -5.0],
    },
    SeriesTerm {
        argument: [0, 0, -2, 2, 0],
        coefficients: [21783.0, 0.0, 13.0, -167.0, 0.0, 13.0],
    },
    SeriesTerm {
        argument: [1, 0, 0, -2, 1],
        coefficients: [-12873.0, -10.0, -37.0, 6953.0, 0.0, -14.0],
    },
    SeriesTerm {
        argument: [0, -1, 0, 0, 1],
        coefficients: [-12654.0, 11.0, 63.0, 6415.0, 0.0, 26.0],
    },
    SeriesTerm {
        argument: [-1, 0, 2, 2, 1],
        coefficients: [-10204.0, 0.0, 25.0, 5222.0, 0.0, 15.0],
    },
    SeriesTerm {
        argument: [0, 2, 0, 0, 0],
        coefficients: [16707.0, -85.0, -10.0, 168.0, -1.0, 10.0],
    },
    SeriesTerm {
        argument: [1, 0, 2, 2, 2],
        coefficients: [-7691.0, 0.0, 44.0, 3268.0, 0.0, 19.0],
    },
    SeriesTerm {
        argument: [-2, 0, 2, 0, 0],
        coefficients: [-11024.0, 0.0, -14.0, 104.0, 0.0, 2.0],
    },
    SeriesTerm {
        argument: [0, 1, 2, 0, 2],
        coefficients: [7566.0, -21.0, -11.0, -3250.0, 0.0, -5.0],
    },
    SeriesTerm {
        argument: [0, 0, 2, 2, 1],
        coefficients: [-6637.0, -11.0, 25.0, 3353.0, 0.0, 14.0],
    },
    SeriesTerm {
        argument: [0, -1, 2, 0, 2],
        coefficients: [-7141.0, 21.0, 8.0, 3070.0, 0.0, 4.0],
    },
    SeriesTerm {
        argument: [0, 0, 0, 2, 1],
        coefficients: [-6302.0, -11.0, 2.0, 3272.0, 0.0, 4.0],
    },
    SeriesTerm {
        argument: [1, 0, 2, -2, 1],
        coefficients: [5800.0, 10.0, 2.0, -3045.0, 0.0, -1.0],
    },
    SeriesTerm {
        argument: [2, 0, 2, -2, 2],
        coefficients: [6443.0, 0.0, -7.0, -2768.0, 0.0, -4.0],
    },
    SeriesTerm {
        argument: [-2, 0, 0, 2, 1],
        coefficients: [-5774.0, -11.0, -15.0, 3041.0, 0.0, -5.0],
    },
    SeriesTerm {
        argument: [2, 0, 2, 0, 1],
        coefficients: [-5350.0, 0.0, 21.0, 2695.0, 0.0, 12.0],
    },
    SeriesTerm {
        argument: [0, -1, 2, -2, 1],
        coefficients: [-4752.0, -11.0, -3.0, 2719.0, 0.0, -3.0],
    },
    SeriesTerm {
        argument: [0, 0, 0, -2, 1],
        coefficients: [-4940.0, -11.0, -21.0, 2720.0, 0.0, -9.0],
    },
    SeriesTerm {
        argument: [-1, -1, 0, 2, 0],
        coefficients: [7350.0, 0.0, -8.0, -51.0, 0.0, 4.0],
    },
    SeriesTerm {
        argument: [2, 0, 0, -2, 1],
        coefficients: [4065.0, 0.0, 6.0, -2206.0, 0.0, 1.0],
    },
    SeriesTerm {
        argument: [1, 0, 0, 2, 0],
        coefficients: [6579.0, 0.0, -24.0, -199.0, 0.0, 2.0],
    },
    SeriesTerm {
        argument: [0, 1, 2, -2, 1],
        coefficients: [3579.0, 0.0, 5.0, -1900.0, 0.0, 1.0],
    },
    SeriesTerm {
        argument: [1, -1, 0, 0, 0],
        coefficients: [4725.0, 0.0, -6.0, -41.0, 0.0, 3.0],
    },
    SeriesTerm {
        argument: [-2, 0, 2, 0, 2],
        coefficients: [-3075.0, 0.0, -2.0, 1313.0, 0.0, -1.0],
    },
    SeriesTerm {
        argument: [3, 0, 2, 0, 2],
        coefficients: [-2904.0, 0.0, 15.0, 1233.0, 0.0, 7.0],
    },
    SeriesTerm {
        argument: [0, -1, 0, 2, 0],
        coefficients: [4348.0, 0.0, -10.0, -81.0, 0.0, 2.0],
    },
    SeriesTerm {
        argument: [1, -1, 2, 0, 2],
        coefficients: [-2878.0, 0.0, 8.0, 1232.0, 0.0, 4.0],
    },
    SeriesTerm {
        argument: [0, 0, 0, 1, 0],
        coefficients: [-4230.0, 0.0, 5.0, -20.0, 0.0, -2.0],
    },
    SeriesTerm {
        argument: [-1, -1, 2, 2, 2],
        coefficients: [-2819.0, 0.0, 7.0, 1207.0, 0.0, 3.0],
    },
    SeriesTerm {
        argument: [-1, 0, 2, 0, 0],
        coefficients: [-4056.0, 0.0, 5.0, 40.0, 0.0, -2.0],
    },
    SeriesTerm {
        argument: [0, -1, 2, 2, 2],
        coefficients: [-2647.0, 0.0, 11.0, 1129.0, 0.0, 5.0],
    },
    SeriesTerm {
        argument: [-2, 0, 0, 0, 1],
        coefficients: [-2294.0, 0.0, -10.0, 1266.0, 0.0, -4.0],
    },
    SeriesTerm {
        argument: [1, 1, 2, 0, 2],
        coefficients: [2481.0, 0.0, -7.0, -1062.0, 0.0, -3.0],
    },
    SeriesTerm {
        argument: [2, 0, 0, 0, 1],
        coefficients: [2179.0, 0.0, -2.0, -1129.0, 0.0, -2.0],
    },
    SeriesTerm {
        argument: [-1, 1, 0, 1, 0],
        coefficients: [3276.0, 0.0, 1.0, -9.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 1, 0, 0, 0],
        coefficients: [-3389.0, 0.0, 5.0, 35.0, 0.0, -2.0],
    },
    SeriesTerm {
        argument: [1, 0, 2, 0, 0],
        coefficients: [3339.0, 0.0, -13.0, -107.0, 0.0, 1.0],
    },
    SeriesTerm {
        argument: [-1, 0, 2, -2, 1],
        coefficients: [-1987.0, 0.0, -6.0, 1073.0, 0.0, -2.0],
    },
    SeriesTerm {
        argument: [1, 0, 0, 0, 2],
        coefficients: [-1981.0, 0.0, 0.0, 854.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 0, 0, 1, 0],
        coefficients: [4026.0, 0.0, -353.0, -553.0, 0.0, -139.0],
    },
    SeriesTerm {
        argument: [0, 0, 2, 1, 2],
        coefficients: [1660.0, 0.0, -5.0, -710.0, 0.0, -2.0],
    },
    SeriesTerm {
        argument: [-1, 0, 2, 4, 2],
        coefficients: [-1521.0, 0.0, 9.0, 647.0, 0.0, 4.0],
    },
    SeriesTerm {
        argument: [-1, 1, 0, 1, 1],
        coefficients: [1314.0, 0.0, 0.0, -700.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, -2, 2, -2, 1],
        coefficients: [-1283.0, 0.0, 0.0, 672.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 0, 2, 2, 1],
        coefficients: [-1331.0, 0.0, 8.0, 663.0, 0.0, 4.0],
    },
    SeriesTerm {
        argument: [-2, 0, 2, 2, 2],
        coefficients: [1383.0, 0.0, -2.0, -594.0, 0.0, -2.0],
    },
    SeriesTerm {
        argument: [-1, 0, 0, 0, 2],
        coefficients: [1405.0, 0.0, 4.0, -610.0, 0.0, 2.0],
    },
    SeriesTerm {
        argument: [1, 1, 2, -2, 2],
        coefficients: [1290.0, 0.0, 0.0, -556.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, 0, 2, 4, 2],
        coefficients: [-1214.0, 0.0, 5.0, 518.0, 0.0, 2.0],
    },
    SeriesTerm {
        argument: [-1, 0, 4, 0, 2],
        coefficients: [1146.0, 0.0, -3.0, -490.0, 0.0, -1.0],
    },
    SeriesTerm {
        argument: [2, 0, 2, -2, 1],
        coefficients: [1019.0, 0.0, -1.0, -527.0, 0.0, -1.0],
    },
    SeriesTerm {
        argument: [2, 0, 2, 2, 2],
        coefficients: [-1100.0, 0.0, 9.0, 465.0, 0.0, 4.0],
    },
    SeriesTerm {
        argument: [1, 0, 0, 2, 1],
        coefficients: [-970.0, 0.0, 2.0, 496.0, 0.0, 1.0],
    },
    SeriesTerm {
        argument: [3, 0, 0, 0, 0],
        coefficients: [1575.0, 0.0, -6.0, -50.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [3, 0, 2, -2, 2],
        coefficients: [934.0, 0.0, -3.0, -399.0, 0.0, -1.0],
    },
    SeriesTerm {
        argument: [0, 0, 4, -2, 2],
        coefficients: [922.0, 0.0, -1.0, -395.0, 0.0, -1.0],
    },
    SeriesTerm {
        argument: [0, 1, 2, 0, 1],
        coefficients: [815.0, 0.0, -1.0, -422.0, 0.0, -1.0],
    },
    SeriesTerm {
        argument: [0, 0, -2, 2, 1],
        coefficients: [834.0, 0.0, 2.0, -440.0, 0.0, 1.0],
    },
    SeriesTerm {
        argument: [0, 0, 2, -2, 3],
        coefficients: [1248.0, 0.0, 0.0, -170.0, 0.0, 1.0],
    },
    SeriesTerm {
        argument: [-1, 0, 0, 4, 0],
        coefficients: [1338.0, 0.0, -5.0, -39.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 0, -2, 0, 1],
        coefficients: [716.0, 0.0, -2.0, -389.0, 0.0, -1.0],
    },
    SeriesTerm {
        argument: [-2, 0, 0, 4, 0],
        coefficients: [1282.0, 0.0, -3.0, -23.0, 0.0, 1.0],
    },
    SeriesTerm {
        argument: [-1, -1, 0, 2, 1],
        coefficients: [742.0, 0.0, 1.0, -391.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 0, 0, 1, 1],
        coefficients: [1020.0, 0.0, -25.0, -495.0, 0.0, -10.0],
    },
    SeriesTerm {
        argument: [0, 1, 0, 0, 2],
        coefficients: [715.0, 0.0, -4.0, -326.0, 0.0, 2.0],
    },
    SeriesTerm {
        argument: [0, 0, -2, 0, 1],
        coefficients: [-666.0, 0.0, -3.0, 369.0, 0.0, -1.0],
    },
    SeriesTerm {
        argument: [0, -1, 2, 0, 1],
        coefficients: [-667.0, 0.0, 1.0, 346.0, 0.0, 1.0],
    },
    SeriesTerm {
        argument: [0, 0, 2, -1, 2],
        coefficients: [-704.0, 0.0, 0.0, 304.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, 2, 4, 2],
        coefficients: [-694.0, 0.0, 5.0, 294.0, 0.0, 2.0],
    },
    SeriesTerm {
        argument: [-2, -1, 0, 2, 0],
        coefficients: [-1014.0, 0.0, -1.0, 4.0, 0.0, -1.0],
    },
    SeriesTerm {
        argument: [1, 1, 0, -2, 1],
        coefficients: [-585.0, 0.0, -2.0, 316.0, 0.0, -1.0],
    },
    SeriesTerm {
        argument: [-1, 1, 0, 2, 0],
        coefficients: [-949.0, 0.0, 1.0, 8.0, 0.0, -1.0],
    },
    SeriesTerm {
        argument: [-1, 1, 0, 1, 2],
        coefficients: [-595.0, 0.0, 0.0, 258.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, -1, 0, 0, 1],
        coefficients: [528.0, 0.0, 0.0, -279.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, -1, 2, 2, 2],
        coefficients: [-590.0, 0.0, 4.0, 252.0, 0.0, 2.0],
    },
    SeriesTerm {
        argument: [-1, 1, 2, 2, 2],
        coefficients: [570.0, 0.0, -2.0, -244.0, 0.0, -1.0],
    },
    SeriesTerm {
        argument: [3, 0, 2, 0, 1],
        coefficients: [-502.0, 0.0, 3.0, 250.0, 0.0, 2.0],
    },
    SeriesTerm {
        argument: [0, 1, -2, 2, 0],
        coefficients: [-875.0, 0.0, 1.0, 29.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 0, 0, -2, 1],
        coefficients: [-492.0, 0.0, -3.0, 275.0, 0.0, -1.0],
    },
    SeriesTerm {
        argument: [0, 1, 2, 2, 2],
        coefficients: [535.0, 0.0, -2.0, -228.0, 0.0, -1.0],
    },
    SeriesTerm {
        argument: [-1, -1, 2, 2, 1],
        coefficients: [-467.0, 0.0, 1.0, 240.0, 0.0, 1.0],
    },
    SeriesTerm {
        argument: [0, -1, 0, 0, 2],
        coefficients: [591.0, 0.0, 0.0, -253.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 0, 2, -4, 1],
        coefficients: [-453.0, 0.0, -1.0, 244.0, 0.0, -1.0],
    },
    SeriesTerm {
        argument: [-1, 0, -2, 2, 0],
        coefficients: [766.0, 0.0, 1.0, 9.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, -1, 2, 2, 1],
        coefficients: [-446.0, 0.0, 2.0, 225.0, 0.0, 1.0],
    },
    SeriesTerm {
        argument: [2, -1, 2, 0, 2],
        coefficients: [-488.0, 0.0, 2.0, 207.0, 0.0, 1.0],
    },
    SeriesTerm {
        argument: [0, 0, 0, 2, 2],
        coefficients: [-468.0, 0.0, 0.0, 201.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, -1, 2, 0, 1],
        coefficients: [-421.0, 0.0, 1.0, 216.0, 0.0, 1.0],
    },
    SeriesTerm {
        argument: [-1, 1, 2, 0, 2],
        coefficients: [463.0, 0.0, 0.0, -200.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 1, 0, 2, 0],
        coefficients: [-673.0, 0.0, 2.0, 14.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, -1, -2, 2, 0],
        coefficients: [658.0, 0.0, 0.0, -2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 3, 2, -2, 2],
        coefficients: [-438.0, 0.0, 0.0, 188.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, 0, 1, 1],
        coefficients: [-390.0, 0.0, 0.0, 205.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 0, 2, 2, 0],
        coefficients: [639.0, -11.0, -2.0, -19.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 1, 2, 0, 2],
        coefficients: [412.0, 0.0, -2.0, -176.0, 0.0, -1.0],
    },
    SeriesTerm {
        argument: [1, 1, 0, 0, 1],
        coefficients: [-361.0, 0.0, 0.0, 189.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 1, 2, 0, 1],
        coefficients: [360.0, 0.0, -1.0, -185.0, 0.0, -1.0],
    },
    SeriesTerm {
        argument: [2, 0, 0, 2, 0],
        coefficients: [588.0, 0.0, -3.0, -24.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 0, -2, 2, 0],
        coefficients: [-578.0, 0.0, 1.0, 5.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 0, 0, 2, 2],
        coefficients: [-396.0, 0.0, 0.0, 171.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 1, 0, 1, 0],
        coefficients: [565.0, 0.0, -1.0, -6.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 1, 0, -2, 1],
        coefficients: [-335.0, 0.0, -1.0, 184.0, 0.0, -1.0],
    },
    SeriesTerm {
        argument: [-1, 0, 2, -2, 2],
        coefficients: [357.0, 0.0, 1.0, -154.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, 0, -1, 1],
        coefficients: [321.0, 0.0, 1.0, -174.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 1, 0, 0, 1],
        coefficients: [-301.0, 0.0, -1.0, 162.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 0, 2, -1, 2],
        coefficients: [-334.0, 0.0, 0.0, 144.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, -1, 0, 2, 0],
        coefficients: [493.0, 0.0, -2.0, -15.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, 0, 4, 0],
        coefficients: [494.0, 0.0, -2.0, -19.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 0, 2, 1, 2],
        coefficients: [337.0, 0.0, -1.0, -143.0, 0.0, -1.0],
    },
    SeriesTerm {
        argument: [0, 0, 2, 1, 1],
        coefficients: [280.0, 0.0, -1.0, -144.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 0, 0, -2, 2],
        coefficients: [309.0, 0.0, 1.0, -134.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 0, 2, 4, 1],
        coefficients: [-263.0, 0.0, 2.0, 131.0, 0.0, 1.0],
    },
    SeriesTerm {
        argument: [1, 0, -2, 0, 1],
        coefficients: [253.0, 0.0, 1.0, -138.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 1, 2, -2, 1],
        coefficients: [245.0, 0.0, 0.0, -128.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, 2, 2, 0],
        coefficients: [416.0, 0.0, -2.0, -17.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 0, 2, -1, 1],
        coefficients: [-229.0, 0.0, 0.0, 128.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, 0, 2, 2, 1],
        coefficients: [231.0, 0.0, 0.0, -120.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [4, 0, 2, 0, 2],
        coefficients: [-259.0, 0.0, 2.0, 109.0, 0.0, 1.0],
    },
    SeriesTerm {
        argument: [2, -1, 0, 0, 0],
        coefficients: [375.0, 0.0, -1.0, -8.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 1, 2, -2, 2],
        coefficients: [252.0, 0.0, 0.0, -108.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 1, 2, 1, 2],
        coefficients: [-245.0, 0.0, 1.0, 104.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 0, 4, -2, 2],
        coefficients: [243.0, 0.0, -1.0, -104.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, -1, 0, 0, 1],
        coefficients: [208.0, 0.0, 1.0, -112.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 1, 0, 2, 1],
        coefficients: [199.0, 0.0, 0.0, -102.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, 0, 2, 4, 1],
        coefficients: [-208.0, 0.0, 1.0, 105.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 0, 2, 0, 0],
        coefficients: [335.0, 0.0, -2.0, -14.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 0, 0, 1, 0],
        coefficients: [-325.0, 0.0, 1.0, 7.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 0, 0, 4, 1],
        coefficients: [-187.0, 0.0, 0.0, 96.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 0, 4, 0, 1],
        coefficients: [197.0, 0.0, -1.0, -100.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 0, 2, 2, 1],
        coefficients: [-192.0, 0.0, 2.0, 94.0, 0.0, 1.0],
    },
    SeriesTerm {
        argument: [0, 0, 2, -3, 2],
        coefficients: [-188.0, 0.0, 0.0, 83.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, -2, 0, 2, 0],
        coefficients: [276.0, 0.0, 0.0, -2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 1, 0, 0, 0],
        coefficients: [-286.0, 0.0, 1.0, 6.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, 4, 0, 2],
        coefficients: [186.0, 0.0, -1.0, -79.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, 0, 0, 3],
        coefficients: [-219.0, 0.0, 0.0, 43.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 3, 0, 0, 0],
        coefficients: [276.0, 0.0, 0.0, 2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, 2, -4, 1],
        coefficients: [-153.0, 0.0, -1.0, 84.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, -1, 0, 2, 1],
        coefficients: [-156.0, 0.0, 0.0, 81.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, 0, 4, 1],
        coefficients: [-154.0, 0.0, 1.0, 78.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, -1, 2, 4, 2],
        coefficients: [-174.0, 0.0, 1.0, 75.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 0, 2, 4, 2],
        coefficients: [-163.0, 0.0, 2.0, 69.0, 0.0, 1.0],
    },
    SeriesTerm {
        argument: [-2, 2, 0, 2, 0],
        coefficients: [-228.0, 0.0, 0.0, 1.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, -1, 2, 0, 1],
        coefficients: [91.0, 0.0, -4.0, -54.0, 0.0, -2.0],
    },
    SeriesTerm {
        argument: [-2, 0, 0, 2, 2],
        coefficients: [175.0, 0.0, 0.0, -75.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, -1, 2, 0, 2],
        coefficients: [-159.0, 0.0, 0.0, 69.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, 4, -2, 1],
        coefficients: [141.0, 0.0, 0.0, -72.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [3, 0, 2, -2, 1],
        coefficients: [147.0, 0.0, 0.0, -75.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, -1, 0, 2, 1],
        coefficients: [-132.0, 0.0, 0.0, 69.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 0, 0, -1, 1],
        coefficients: [159.0, 0.0, -28.0, -54.0, 0.0, 11.0],
    },
    SeriesTerm {
        argument: [0, -2, 0, 2, 0],
        coefficients: [213.0, 0.0, 0.0, -4.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, 0, 0, 4, 1],
        coefficients: [123.0, 0.0, 0.0, -64.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-3, 0, 0, 0, 1],
        coefficients: [-118.0, 0.0, -1.0, 66.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 1, 2, 2, 2],
        coefficients: [144.0, 0.0, -1.0, -61.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, 2, 4, 1],
        coefficients: [-121.0, 0.0, 1.0, 60.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [3, 0, 2, 2, 2],
        coefficients: [-134.0, 0.0, 1.0, 56.0, 0.0, 1.0],
    },
    SeriesTerm {
        argument: [-1, 1, 2, -2, 1],
        coefficients: [-105.0, 0.0, 0.0, 57.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 0, 0, -4, 1],
        coefficients: [-102.0, 0.0, 0.0, 56.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, 0, -2, 2],
        coefficients: [120.0, 0.0, 0.0, -52.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 0, 2, -4, 1],
        coefficients: [101.0, 0.0, 0.0, -54.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 1, 0, 2, 1],
        coefficients: [-113.0, 0.0, 0.0, 59.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, 2, -1, 1],
        coefficients: [-106.0, 0.0, 0.0, 61.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, -2, 2, 2, 2],
        coefficients: [-129.0, 0.0, 1.0, 55.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 0, 0, 2, 1],
        coefficients: [-114.0, 0.0, 0.0, 57.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [4, 0, 2, -2, 2],
        coefficients: [113.0, 0.0, -1.0, -49.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 0, 0, -2, 2],
        coefficients: [-102.0, 0.0, 0.0, 44.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 2, 0, 0, 1],
        coefficients: [-94.0, 0.0, 0.0, 51.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 0, 0, -4, 1],
        coefficients: [-100.0, 0.0, -1.0, 56.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 2, 2, -2, 1],
        coefficients: [87.0, 0.0, 0.0, -47.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-3, 0, 0, 4, 0],
        coefficients: [161.0, 0.0, 0.0, -1.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 1, 2, 0, 1],
        coefficients: [96.0, 0.0, 0.0, -50.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, -1, 0, 4, 0],
        coefficients: [151.0, 0.0, -1.0, -5.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, -2, 2, 2, 2],
        coefficients: [-104.0, 0.0, 0.0, 44.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, -1, 2, 4, 2],
        coefficients: [-110.0, 0.0, 0.0, 48.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, -1, 2, 2, 1],
        coefficients: [-100.0, 0.0, 1.0, 50.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, 1, 0, 2, 0],
        coefficients: [92.0, 0.0, -5.0, 12.0, 0.0, -2.0],
    },
    SeriesTerm {
        argument: [-2, 1, 2, 0, 1],
        coefficients: [82.0, 0.0, 0.0, -45.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 1, 0, -2, 1],
        coefficients: [82.0, 0.0, 0.0, -45.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-3, 0, 2, 0, 1],
        coefficients: [-78.0, 0.0, 0.0, 41.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, 0, 2, -2, 1],
        coefficients: [-77.0, 0.0, 0.0, 43.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 1, 0, 2, 2],
        coefficients: [2.0, 0.0, 0.0, 54.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, -1, 2, -1, 2],
        coefficients: [94.0, 0.0, 0.0, -40.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 0, 4, -2, 2],
        coefficients: [-93.0, 0.0, 0.0, 40.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, -2, 2, 0, 2],
        coefficients: [-83.0, 0.0, 10.0, 40.0, 0.0, -2.0],
    },
    SeriesTerm {
        argument: [-1, 0, 2, 1, 2],
        coefficients: [83.0, 0.0, 0.0, -36.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 0, 0, 0, 2],
        coefficients: [-91.0, 0.0, 0.0, 39.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, 2, 0, 3],
        coefficients: [128.0, 0.0, 0.0, -1.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, 0, 4, 0, 2],
        coefficients: [-79.0, 0.0, 0.0, 34.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 0, -2, 0, 1],
        coefficients: [-83.0, 0.0, 0.0, 47.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 1, 2, 2, 1],
        coefficients: [84.0, 0.0, 0.0, -44.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [3, 0, 0, 0, 1],
        coefficients: [83.0, 0.0, 0.0, -43.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 0, 2, 3, 2],
        coefficients: [91.0, 0.0, 0.0, -39.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, -1, 2, 0, 1],
        coefficients: [-77.0, 0.0, 0.0, 39.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 1, 2, 2, 1],
        coefficients: [84.0, 0.0, 0.0, -43.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, -1, 2, 4, 2],
        coefficients: [-92.0, 0.0, 1.0, 39.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, -1, 2, 2, 2],
        coefficients: [-92.0, 0.0, 1.0, 39.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 2, -2, 2, 0],
        coefficients: [-94.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, -1, 2, -1, 1],
        coefficients: [68.0, 0.0, 0.0, -36.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, -2, 0, 0, 1],
        coefficients: [-61.0, 0.0, 0.0, 32.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 0, 2, -4, 2],
        coefficients: [71.0, 0.0, 0.0, -31.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, -1, 0, -2, 1],
        coefficients: [62.0, 0.0, 0.0, -34.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, -1, 2, 0, 1],
        coefficients: [-63.0, 0.0, 0.0, 33.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, -1, 2, -2, 2],
        coefficients: [-73.0, 0.0, 0.0, 32.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, -1, 0, 4, 0],
        coefficients: [115.0, 0.0, 0.0, -2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 0, 0, 3, 0],
        coefficients: [-103.0, 0.0, 0.0, 2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, -1, 2, 2, 2],
        coefficients: [63.0, 0.0, 0.0, -28.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 2, 2, 0, 2],
        coefficients: [74.0, 0.0, 0.0, -32.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 1, 0, 2, 0],
        coefficients: [-103.0, 0.0, -3.0, 3.0, 0.0, -1.0],
    },
    SeriesTerm {
        argument: [2, 0, 2, -1, 2],
        coefficients: [-69.0, 0.0, 0.0, 30.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 0, 2, 1, 1],
        coefficients: [57.0, 0.0, 0.0, -29.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [4, 0, 0, 0, 0],
        coefficients: [94.0, 0.0, 0.0, -4.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 1, 2, 0, 1],
        coefficients: [64.0, 0.0, 0.0, -33.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [3, -1, 2, 0, 2],
        coefficients: [-63.0, 0.0, 0.0, 26.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, 2, 0, 2, 1],
        coefficients: [-38.0, 0.0, 0.0, 20.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 0, 2, -3, 1],
        coefficients: [-43.0, 0.0, 0.0, 24.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 1, 2, -4, 1],
        coefficients: [-45.0, 0.0, 0.0, 23.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, -1, 2, -2, 1],
        coefficients: [47.0, 0.0, 0.0, -24.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, -1, 0, -1, 1],
        coefficients: [-48.0, 0.0, 0.0, 25.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, -1, 0, -2, 1],
        coefficients: [45.0, 0.0, 0.0, -26.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, 0, 0, 0, 2],
        coefficients: [56.0, 0.0, 0.0, -25.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, 0, -2, 2, 0],
        coefficients: [88.0, 0.0, 0.0, 2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 0, -2, 4, 0],
        coefficients: [-75.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, -2, 0, 0, 0],
        coefficients: [85.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 1, 0, 1, 1],
        coefficients: [49.0, 0.0, 0.0, -26.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 2, 0, 2, 0],
        coefficients: [-74.0, 0.0, -3.0, -1.0, 0.0, -1.0],
    },
    SeriesTerm {
        argument: [1, -1, 2, -2, 1],
        coefficients: [-39.0, 0.0, 0.0, 21.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 2, 2, -2, 2],
        coefficients: [45.0, 0.0, 0.0, -20.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, -1, 2, -2, 2],
        coefficients: [51.0, 0.0, 0.0, -22.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 0, 2, -1, 1],
        coefficients: [-40.0, 0.0, 0.0, 21.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 1, 2, -2, 1],
        coefficients: [41.0, 0.0, 0.0, -21.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, 0, 0, -2, 1],
        coefficients: [-42.0, 0.0, 0.0, 24.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, -2, 2, 0, 2],
        coefficients: [-51.0, 0.0, 0.0, 22.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 1, 2, 1, 1],
        coefficients: [-42.0, 0.0, 0.0, 22.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 0, 4, -2, 1],
        coefficients: [39.0, 0.0, 0.0, -21.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, 0, 4, 2, 2],
        coefficients: [46.0, 0.0, 0.0, -18.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 1, 2, 1, 2],
        coefficients: [-53.0, 0.0, 0.0, 22.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 0, 0, 4, 0],
        coefficients: [82.0, 0.0, 0.0, -4.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 0, 2, 2, 0],
        coefficients: [81.0, 0.0, -1.0, -4.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 0, 2, 1, 2],
        coefficients: [47.0, 0.0, 0.0, -19.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [3, 1, 2, 0, 2],
        coefficients: [53.0, 0.0, 0.0, -23.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [4, 0, 2, 0, 1],
        coefficients: [-45.0, 0.0, 0.0, 22.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, -1, 2, 0, 0],
        coefficients: [-44.0, 0.0, 0.0, -2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 1, -2, 2, 1],
        coefficients: [-33.0, 0.0, 0.0, 16.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 0, -2, 1, 0],
        coefficients: [-61.0, 0.0, 0.0, 1.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, -1, -2, 2, 1],
        coefficients: [28.0, 0.0, 0.0, -15.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, -1, 0, -2, 1],
        coefficients: [-38.0, 0.0, 0.0, 19.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 0, 2, -1, 2],
        coefficients: [-33.0, 0.0, 0.0, 21.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 0, 2, -3, 2],
        coefficients: [-60.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 1, 2, -2, 3],
        coefficients: [48.0, 0.0, 0.0, -10.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, 2, -3, 1],
        coefficients: [27.0, 0.0, 0.0, -14.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 0, -2, 2, 1],
        coefficients: [38.0, 0.0, 0.0, -20.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, 2, -4, 2],
        coefficients: [31.0, 0.0, 0.0, -13.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, 1, 0, 0, 1],
        coefficients: [-29.0, 0.0, 0.0, 15.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 0, 0, -1, 1],
        coefficients: [28.0, 0.0, 0.0, -15.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 0, 2, -4, 2],
        coefficients: [-32.0, 0.0, 0.0, 15.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, 4, -4, 4],
        coefficients: [45.0, 0.0, 0.0, -8.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, 4, -4, 2],
        coefficients: [-44.0, 0.0, 0.0, 19.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, -2, 0, 2, 1],
        coefficients: [28.0, 0.0, 0.0, -15.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, 0, 0, 3, 0],
        coefficients: [-51.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 0, -2, 2, 1],
        coefficients: [-36.0, 0.0, 0.0, 20.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-3, 0, 2, 2, 2],
        coefficients: [44.0, 0.0, 0.0, -19.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-3, 0, 2, 2, 1],
        coefficients: [26.0, 0.0, 0.0, -14.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, 0, 2, 2, 0],
        coefficients: [-60.0, 0.0, 0.0, 2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, -1, 0, 0, 1],
        coefficients: [35.0, 0.0, 0.0, -18.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, 1, 2, 2, 2],
        coefficients: [-27.0, 0.0, 0.0, 11.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 1, 0, 1, 0],
        coefficients: [47.0, 0.0, 0.0, -1.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 1, 4, -2, 2],
        coefficients: [36.0, 0.0, 0.0, -15.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 1, 0, -2, 1],
        coefficients: [-36.0, 0.0, 0.0, 20.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, 0, -4, 1],
        coefficients: [-35.0, 0.0, 0.0, 19.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, -1, 0, 2, 1],
        coefficients: [-37.0, 0.0, 0.0, 19.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 1, 0, 2, 1],
        coefficients: [32.0, 0.0, 0.0, -16.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 2, 2, 2, 2],
        coefficients: [35.0, 0.0, 0.0, -14.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [3, 1, 2, -2, 2],
        coefficients: [32.0, 0.0, 0.0, -13.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, -1, 0, 4, 0],
        coefficients: [65.0, 0.0, 0.0, -2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, -1, 0, 2, 0],
        coefficients: [47.0, 0.0, 0.0, -1.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, 4, 0, 1],
        coefficients: [32.0, 0.0, 0.0, -16.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 0, 4, -2, 2],
        coefficients: [37.0, 0.0, 0.0, -16.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, -1, 2, 4, 1],
        coefficients: [-30.0, 0.0, 0.0, 15.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 0, 0, 4, 1],
        coefficients: [-32.0, 0.0, 0.0, 16.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, -2, 2, 2, 2],
        coefficients: [-31.0, 0.0, 0.0, 13.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, 2, 3, 2],
        coefficients: [37.0, 0.0, 0.0, -16.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 1, 2, 4, 2],
        coefficients: [31.0, 0.0, 0.0, -13.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [3, 0, 0, 2, 0],
        coefficients: [49.0, 0.0, 0.0, -2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 0, 4, 2, 2],
        coefficients: [32.0, 0.0, 0.0, -13.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 1, 2, 2, 1],
        coefficients: [23.0, 0.0, 0.0, -12.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, 0, 2, 6, 2],
        coefficients: [-43.0, 0.0, 0.0, 18.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 1, 2, 2, 2],
        coefficients: [26.0, 0.0, 0.0, -11.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 0, 2, 6, 2],
        coefficients: [-32.0, 0.0, 0.0, 14.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 0, 2, 4, 1],
        coefficients: [-29.0, 0.0, 0.0, 14.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 0, 2, 4, 2],
        coefficients: [-27.0, 0.0, 0.0, 12.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 1, -2, 1, 0],
        coefficients: [30.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-3, 1, 2, 1, 2],
        coefficients: [-11.0, 0.0, 0.0, 5.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 0, -2, 0, 2],
        coefficients: [-21.0, 0.0, 0.0, 10.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 0, 0, 1, 2],
        coefficients: [-34.0, 0.0, 0.0, 15.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-4, 0, 2, 2, 1],
        coefficients: [-10.0, 0.0, 0.0, 6.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, -1, 0, 1, 0],
        coefficients: [-36.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, -2, 2, 2],
        coefficients: [-9.0, 0.0, 0.0, 4.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 0, 0, -1, 2],
        coefficients: [-12.0, 0.0, 0.0, 5.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, -1, 2, -2, 3],
        coefficients: [-21.0, 0.0, 0.0, 5.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, 1, 2, 0, 0],
        coefficients: [-29.0, 0.0, 0.0, -1.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, 2, -2, 4],
        coefficients: [-15.0, 0.0, 0.0, 3.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, -2, 0, 2, 0],
        coefficients: [-20.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, 0, -2, 4, 0],
        coefficients: [28.0, 0.0, 0.0, 0.0, 0.0, -2.0],
    },
    SeriesTerm {
        argument: [0, -2, -2, 2, 0],
        coefficients: [17.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 2, 0, -2, 1],
        coefficients: [-22.0, 0.0, 0.0, 12.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [3, 0, 0, -4, 1],
        coefficients: [-14.0, 0.0, 0.0, 7.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 1, 2, -2, 2],
        coefficients: [24.0, 0.0, 0.0, -11.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, -1, 2, -4, 1],
        coefficients: [11.0, 0.0, 0.0, -6.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 1, 0, -2, 2],
        coefficients: [14.0, 0.0, 0.0, -6.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-3, 0, 2, 0, 0],
        coefficients: [24.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-3, 0, 2, 0, 2],
        coefficients: [18.0, 0.0, 0.0, -8.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, 0, 0, 1, 0],
        coefficients: [-38.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, -2, 1, 0],
        coefficients: [-31.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-3, 0, 0, 2, 1],
        coefficients: [-16.0, 0.0, 0.0, 8.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, -1, -2, 2, 0],
        coefficients: [29.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 1, 2, -4, 1],
        coefficients: [-18.0, 0.0, 0.0, 10.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 1, 0, -4, 1],
        coefficients: [-10.0, 0.0, 0.0, 5.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 2, 0, -2, 1],
        coefficients: [-17.0, 0.0, 0.0, 10.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 0, 0, -3, 1],
        coefficients: [9.0, 0.0, 0.0, -4.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, 0, 2, -2, 2],
        coefficients: [16.0, 0.0, 0.0, -6.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, -1, 0, 0, 1],
        coefficients: [22.0, 0.0, 0.0, -12.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-4, 0, 0, 2, 0],
        coefficients: [20.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 1, 0, -4, 1],
        coefficients: [-13.0, 0.0, 0.0, 6.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 0, 2, -4, 1],
        coefficients: [-17.0, 0.0, 0.0, 9.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, 4, -4, 1],
        coefficients: [-14.0, 0.0, 0.0, 8.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 3, 2, -2, 2],
        coefficients: [0.0, 0.0, 0.0, -7.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-3, -1, 0, 4, 0],
        coefficients: [14.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-3, 0, 0, 4, 1],
        coefficients: [19.0, 0.0, 0.0, -10.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, -1, -2, 2, 0],
        coefficients: [-34.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, -1, 0, 2, 2],
        coefficients: [-20.0, 0.0, 0.0, 8.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, -2, 0, 0, 1],
        coefficients: [9.0, 0.0, 0.0, -5.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, -1, 0, 0, 2],
        coefficients: [-18.0, 0.0, 0.0, 7.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, 0, 1, 2],
        coefficients: [13.0, 0.0, 0.0, -6.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, -1, 2, 0, 0],
        coefficients: [17.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, -2, 2, -2, 2],
        coefficients: [-12.0, 0.0, 0.0, 5.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, -1, 2, -1, 1],
        coefficients: [15.0, 0.0, 0.0, -8.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 0, 2, 0, 3],
        coefficients: [-11.0, 0.0, 0.0, 3.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 1, 0, 0, 2],
        coefficients: [13.0, 0.0, 0.0, -5.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 1, 2, 0, 0],
        coefficients: [-18.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 2, 0, 0, 0],
        coefficients: [-35.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 2, 2, 0, 2],
        coefficients: [9.0, 0.0, 0.0, -4.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 0, 4, -2, 1],
        coefficients: [-19.0, 0.0, 0.0, 10.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [3, 0, 2, -4, 2],
        coefficients: [-26.0, 0.0, 0.0, 11.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 2, 2, -2, 1],
        coefficients: [8.0, 0.0, 0.0, -4.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 0, 4, -4, 2],
        coefficients: [-10.0, 0.0, 0.0, 4.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, -1, 0, 4, 1],
        coefficients: [10.0, 0.0, 0.0, -6.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, -1, 0, 2, 2],
        coefficients: [-21.0, 0.0, 0.0, 9.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, 1, 0, 4, 0],
        coefficients: [-15.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, -1, 2, 2, 1],
        coefficients: [9.0, 0.0, 0.0, -5.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 0, -2, 2, 0],
        coefficients: [-29.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 0, 0, 1, 1],
        coefficients: [-19.0, 0.0, 0.0, 10.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 1, 0, 2, 2],
        coefficients: [12.0, 0.0, 0.0, -5.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, -1, 2, -1, 2],
        coefficients: [22.0, 0.0, 0.0, -9.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, 0, 4, 0, 1],
        coefficients: [-10.0, 0.0, 0.0, 5.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 1, 0, 0, 1],
        coefficients: [-20.0, 0.0, 0.0, 11.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 1, 2, 0, 0],
        coefficients: [-20.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, -1, 4, -2, 2],
        coefficients: [-17.0, 0.0, 0.0, 7.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, 4, -2, 4],
        coefficients: [15.0, 0.0, 0.0, -3.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 2, 2, 0, 1],
        coefficients: [8.0, 0.0, 0.0, -4.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-3, 0, 0, 6, 0],
        coefficients: [14.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, -1, 0, 4, 1],
        coefficients: [-12.0, 0.0, 0.0, 6.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, -2, 0, 2, 0],
        coefficients: [25.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 0, 0, 4, 2],
        coefficients: [-13.0, 0.0, 0.0, 6.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, -2, 2, 2, 1],
        coefficients: [-14.0, 0.0, 0.0, 8.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 0, 0, -2, 2],
        coefficients: [13.0, 0.0, 0.0, -5.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 0, -2, -2, 1],
        coefficients: [-17.0, 0.0, 0.0, 9.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, -2, -2, 1],
        coefficients: [-12.0, 0.0, 0.0, 6.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, 0, -2, 0, 1],
        coefficients: [-10.0, 0.0, 0.0, 5.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, 0, 3, 1],
        coefficients: [10.0, 0.0, 0.0, -6.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, 0, 3, 0],
        coefficients: [-15.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 1, 0, 4, 0],
        coefficients: [-22.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, -1, 2, 2, 0],
        coefficients: [28.0, 0.0, 0.0, -1.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, 0, 2, 3, 2],
        coefficients: [15.0, 0.0, 0.0, -7.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 0, 0, 2, 2],
        coefficients: [23.0, 0.0, 0.0, -10.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, -1, 2, 1, 2],
        coefficients: [12.0, 0.0, 0.0, -5.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [3, -1, 0, 0, 0],
        coefficients: [29.0, 0.0, 0.0, -1.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 0, 0, 1, 0],
        coefficients: [-25.0, 0.0, 0.0, 1.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, -1, 2, 0, 0],
        coefficients: [22.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, 2, 1, 0],
        coefficients: [-18.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 0, 2, 0, 3],
        coefficients: [15.0, 0.0, 0.0, 3.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [3, 1, 0, 0, 0],
        coefficients: [-23.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [3, -1, 2, -2, 2],
        coefficients: [12.0, 0.0, 0.0, -5.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 0, 2, -1, 1],
        coefficients: [-8.0, 0.0, 0.0, 4.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 1, 2, 0, 0],
        coefficients: [-19.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, 4, -1, 2],
        coefficients: [-10.0, 0.0, 0.0, 4.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 2, 2, 0, 2],
        coefficients: [21.0, 0.0, 0.0, -9.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, 0, 0, 6, 0],
        coefficients: [23.0, 0.0, 0.0, -1.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, -1, 0, 4, 1],
        coefficients: [-16.0, 0.0, 0.0, 8.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, -1, 2, 4, 1],
        coefficients: [-19.0, 0.0, 0.0, 9.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, -2, 2, 2, 1],
        coefficients: [-22.0, 0.0, 0.0, 10.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, -1, 2, 2, 0],
        coefficients: [27.0, 0.0, 0.0, -1.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 0, 2, 3, 1],
        coefficients: [16.0, 0.0, 0.0, -8.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, 1, 2, 4, 2],
        coefficients: [19.0, 0.0, 0.0, -8.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 0, 0, 2, 2],
        coefficients: [9.0, 0.0, 0.0, -4.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, -2, 2, 0, 2],
        coefficients: [-9.0, 0.0, 0.0, 4.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 1, 2, 3, 2],
        coefficients: [-9.0, 0.0, 0.0, 4.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [3, 0, 2, -1, 2],
        coefficients: [-8.0, 0.0, 0.0, 4.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [4, 0, 2, -2, 1],
        coefficients: [18.0, 0.0, 0.0, -9.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 0, 0, 6, 0],
        coefficients: [16.0, 0.0, 0.0, -1.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, -2, 2, 4, 2],
        coefficients: [-10.0, 0.0, 0.0, 4.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-3, 0, 2, 6, 2],
        coefficients: [-23.0, 0.0, 0.0, 9.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 0, 2, 4, 0],
        coefficients: [16.0, 0.0, 0.0, -1.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [3, 0, 0, 2, 1],
        coefficients: [-12.0, 0.0, 0.0, 6.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [3, -1, 2, 0, 1],
        coefficients: [-8.0, 0.0, 0.0, 4.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [3, 0, 2, 0, 0],
        coefficients: [30.0, 0.0, 0.0, -2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 0, 4, 0, 2],
        coefficients: [24.0, 0.0, 0.0, -10.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [5, 0, 2, -2, 2],
        coefficients: [10.0, 0.0, 0.0, -4.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, -1, 2, 4, 1],
        coefficients: [-16.0, 0.0, 0.0, 7.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, -1, 2, 2, 1],
        coefficients: [-16.0, 0.0, 0.0, 7.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 1, 2, 4, 2],
        coefficients: [17.0, 0.0, 0.0, -7.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, -1, 2, 4, 2],
        coefficients: [-24.0, 0.0, 0.0, 10.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [3, -1, 2, 2, 2],
        coefficients: [-12.0, 0.0, 0.0, 5.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [3, 0, 2, 2, 1],
        coefficients: [-24.0, 0.0, 0.0, 11.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [5, 0, 2, 0, 2],
        coefficients: [-23.0, 0.0, 0.0, 9.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, 2, 6, 2],
        coefficients: [-13.0, 0.0, 0.0, 5.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [4, 0, 2, 2, 2],
        coefficients: [-15.0, 0.0, 0.0, 7.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, -1, 1, -1, 1],
        coefficients: [0.0, 0.0, -1988.0, 0.0, 0.0, -1679.0],
    },
    SeriesTerm {
        argument: [-1, 0, 1, 0, 3],
        coefficients: [0.0, 0.0, -63.0, 0.0, 0.0, -27.0],
    },
    SeriesTerm {
        argument: [0, -2, 2, -2, 3],
        coefficients: [-4.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 0, -1, 0, 1],
        coefficients: [0.0, 0.0, 5.0, 0.0, 0.0, 4.0],
    },
    SeriesTerm {
        argument: [2, -2, 0, -2, 1],
        coefficients: [5.0, 0.0, 0.0, -3.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 0, 1, 0, 2],
        coefficients: [0.0, 0.0, 364.0, 0.0, 0.0, 176.0],
    },
    SeriesTerm {
        argument: [-1, 0, 1, 0, 1],
        coefficients: [0.0, 0.0, -1044.0, 0.0, 0.0, -891.0],
    },
    SeriesTerm {
        argument: [-1, -1, 2, -1, 2],
        coefficients: [-3.0, 0.0, 0.0, 1.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, 2, 0, 2, 2],
        coefficients: [4.0, 0.0, 0.0, -2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 0, 1, 0, 0],
        coefficients: [0.0, 0.0, 330.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-4, 1, 2, 2, 2],
        coefficients: [5.0, 0.0, 0.0, -2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-3, 0, 2, 1, 1],
        coefficients: [3.0, 0.0, 0.0, -2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, -1, 2, 0, 2],
        coefficients: [-3.0, 0.0, 0.0, 1.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 0, -2, 1, 1],
        coefficients: [-5.0, 0.0, 0.0, 2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, -1, -2, 0, 1],
        coefficients: [3.0, 0.0, 0.0, -1.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-4, 0, 2, 2, 0],
        coefficients: [3.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-3, 1, 0, 3, 0],
        coefficients: [3.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 0, -1, 2, 0],
        coefficients: [0.0, 0.0, 5.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, -2, 0, 0, 2],
        coefficients: [0.0, 0.0, 0.0, 1.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, -2, 0, 0, 2],
        coefficients: [4.0, 0.0, 0.0, -2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-3, 0, 0, 3, 0],
        coefficients: [6.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, -1, 0, 2, 2],
        coefficients: [5.0, 0.0, 0.0, -2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 0, -2, 3, 0],
        coefficients: [-7.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-4, 0, 0, 4, 0],
        coefficients: [-12.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 1, -2, 0, 1],
        coefficients: [5.0, 0.0, 0.0, -3.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, -1, 0, -2, 2],
        coefficients: [3.0, 0.0, 0.0, -1.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, 1, -1, 0],
        coefficients: [-5.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 2, 0, 1, 0],
        coefficients: [3.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, 1, 2, 0, 2],
        coefficients: [-7.0, 0.0, 0.0, 3.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 1, 0, -1, 1],
        coefficients: [7.0, 0.0, 0.0, -4.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 0, 1, -2, 1],
        coefficients: [0.0, 0.0, -12.0, 0.0, 0.0, -10.0],
    },
    SeriesTerm {
        argument: [0, 2, 0, 0, 2],
        coefficients: [4.0, 0.0, 0.0, -2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, -1, 2, -3, 1],
        coefficients: [3.0, 0.0, 0.0, -2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 1, 2, -1, 1],
        coefficients: [-3.0, 0.0, 0.0, 2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, 0, 4, -2, 2],
        coefficients: [-7.0, 0.0, 0.0, 3.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, 0, 4, -2, 1],
        coefficients: [-4.0, 0.0, 0.0, 2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, -2, 0, 2, 1],
        coefficients: [-3.0, 0.0, 0.0, 1.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, 0, -2, 4, 0],
        coefficients: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 2, 2, -4, 1],
        coefficients: [-3.0, 0.0, 0.0, 1.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 1, 2, -4, 2],
        coefficients: [7.0, 0.0, 0.0, -3.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 2, 2, -2, 1],
        coefficients: [-4.0, 0.0, 0.0, 2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 0, 0, -3, 1],
        coefficients: [4.0, 0.0, 0.0, -2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 2, 0, 0, 1],
        coefficients: [-5.0, 0.0, 0.0, 3.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, 0, -2, 0],
        coefficients: [5.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, -1, 2, -2, 2],
        coefficients: [-5.0, 0.0, 0.0, 2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 1, 0, 0, 2],
        coefficients: [5.0, 0.0, 0.0, -2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, 0, -1, 2],
        coefficients: [-8.0, 0.0, 0.0, 3.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, 1, 0, 1, 0],
        coefficients: [9.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, -2, 0, -2, 1],
        coefficients: [6.0, 0.0, 0.0, -3.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 0, -2, 0, 2],
        coefficients: [-5.0, 0.0, 0.0, 2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-3, 1, 0, 2, 0],
        coefficients: [3.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 1, -2, 2, 0],
        coefficients: [-7.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, -1, 0, 0, 2],
        coefficients: [-3.0, 0.0, 0.0, 1.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-3, 0, 0, 2, 0],
        coefficients: [5.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-3, -1, 0, 2, 0],
        coefficients: [3.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 0, 2, -6, 1],
        coefficients: [-3.0, 0.0, 0.0, 2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 1, 2, -4, 2],
        coefficients: [4.0, 0.0, 0.0, -2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 0, 0, -4, 2],
        coefficients: [3.0, 0.0, 0.0, -1.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, 1, 2, -2, 1],
        coefficients: [-5.0, 0.0, 0.0, 2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, -1, 2, -4, 1],
        coefficients: [4.0, 0.0, 0.0, -2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 1, 0, -2, 2],
        coefficients: [9.0, 0.0, 0.0, -3.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 0, 0, -2, 0],
        coefficients: [4.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 0, -2, -2, 1],
        coefficients: [4.0, 0.0, 0.0, -2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-4, 0, 2, 0, 1],
        coefficients: [-3.0, 0.0, 0.0, 2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, -1, 0, -1, 1],
        coefficients: [-4.0, 0.0, 0.0, 2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, -2, 0, 2],
        coefficients: [9.0, 0.0, 0.0, -3.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-3, 0, 0, 1, 0],
        coefficients: [-4.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 0, -2, 1, 0],
        coefficients: [-4.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, 0, -2, 2, 1],
        coefficients: [3.0, 0.0, 0.0, -2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, -4, 2, 0],
        coefficients: [8.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, -1, -2, 2, 0],
        coefficients: [3.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 0, 2, -6, 1],
        coefficients: [-3.0, 0.0, 0.0, 2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 0, 2, -4, 2],
        coefficients: [3.0, 0.0, 0.0, -1.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 0, 0, -4, 2],
        coefficients: [3.0, 0.0, 0.0, -1.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 1, 2, -4, 2],
        coefficients: [-3.0, 0.0, 0.0, 1.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 1, 2, -4, 1],
        coefficients: [6.0, 0.0, 0.0, -3.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 1, 4, -4, 4],
        coefficients: [3.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 1, 4, -4, 2],
        coefficients: [-3.0, 0.0, 0.0, 1.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, -1, -2, 4, 0],
        coefficients: [-7.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, -3, 0, 2, 0],
        coefficients: [9.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 0, -2, 4, 1],
        coefficients: [-3.0, 0.0, 0.0, 2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, -1, 0, 3, 0],
        coefficients: [-3.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, -2, 3, 0],
        coefficients: [-4.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, 0, 0, 3, 1],
        coefficients: [-5.0, 0.0, 0.0, 3.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, -1, 0, 1, 0],
        coefficients: [-13.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-3, 0, 2, 2, 0],
        coefficients: [-7.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 1, -2, 2, 0],
        coefficients: [10.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 1, 0, 2, 2],
        coefficients: [3.0, 0.0, 0.0, -1.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, -2, 2, -2, 1],
        coefficients: [10.0, 0.0, 13.0, 6.0, 0.0, -5.0],
    },
    SeriesTerm {
        argument: [0, 0, 1, 0, 2],
        coefficients: [0.0, 0.0, 30.0, 0.0, 0.0, 14.0],
    },
    SeriesTerm {
        argument: [0, 0, 1, 0, 1],
        coefficients: [0.0, 0.0, -162.0, 0.0, 0.0, -138.0],
    },
    SeriesTerm {
        argument: [0, 0, 1, 0, 0],
        coefficients: [0.0, 0.0, 75.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 2, 0, 2, 1],
        coefficients: [-7.0, 0.0, 0.0, 4.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, 2, 0, 2],
        coefficients: [-4.0, 0.0, 0.0, 2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, 0, 2, 0, 2],
        coefficients: [4.0, 0.0, 0.0, -2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 0, 0, -1, 1],
        coefficients: [5.0, 0.0, 0.0, -2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [3, 0, 0, -2, 1],
        coefficients: [5.0, 0.0, 0.0, -3.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 0, 2, -2, 3],
        coefficients: [-3.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 2, 0, 0, 1],
        coefficients: [-3.0, 0.0, 0.0, 2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 0, 2, -3, 2],
        coefficients: [-4.0, 0.0, 0.0, 2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 1, 4, -2, 2],
        coefficients: [-5.0, 0.0, 0.0, 2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, -2, 0, 4, 0],
        coefficients: [6.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, -3, 0, 2, 0],
        coefficients: [9.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, -2, 4, 0],
        coefficients: [5.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, -1, 0, 3, 0],
        coefficients: [-7.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, 0, 0, 4, 2],
        coefficients: [-3.0, 0.0, 0.0, 1.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 0, 0, 3, 1],
        coefficients: [-4.0, 0.0, 0.0, 2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, -2, 0, 0, 0],
        coefficients: [7.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, -1, 0, 1, 0],
        coefficients: [-4.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 0, 0, 2, 0],
        coefficients: [4.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, -2, 2, 0, 1],
        coefficients: [-6.0, 0.0, -3.0, 3.0, 0.0, 1.0],
    },
    SeriesTerm {
        argument: [-1, 0, 1, 2, 1],
        coefficients: [0.0, 0.0, -3.0, 0.0, 0.0, -2.0],
    },
    SeriesTerm {
        argument: [-1, 1, 0, 3, 0],
        coefficients: [11.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, -1, 2, 1, 2],
        coefficients: [3.0, 0.0, 0.0, -1.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, -1, 2, 0, 0],
        coefficients: [11.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, 1, 2, 2, 1],
        coefficients: [-3.0, 0.0, 0.0, 2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, -2, 2, -2, 2],
        coefficients: [-1.0, 0.0, 3.0, 3.0, 0.0, -1.0],
    },
    SeriesTerm {
        argument: [1, 1, 0, 1, 1],
        coefficients: [4.0, 0.0, 0.0, -2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 0, 1, 0, 1],
        coefficients: [0.0, 0.0, -13.0, 0.0, 0.0, -11.0],
    },
    SeriesTerm {
        argument: [1, 0, 1, 0, 0],
        coefficients: [3.0, 0.0, 6.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 2, 0, 2, 0],
        coefficients: [-7.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, -1, 2, -2, 1],
        coefficients: [5.0, 0.0, 0.0, -3.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, -1, 4, -2, 1],
        coefficients: [-3.0, 0.0, 0.0, 1.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, 4, -2, 3],
        coefficients: [3.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 1, 4, -2, 1],
        coefficients: [5.0, 0.0, 0.0, -3.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [4, 0, 2, -4, 2],
        coefficients: [-7.0, 0.0, 0.0, 3.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 2, 2, -2, 2],
        coefficients: [8.0, 0.0, 0.0, -3.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 0, 4, -4, 2],
        coefficients: [-4.0, 0.0, 0.0, 2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, -2, 0, 4, 0],
        coefficients: [11.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, -3, 2, 2, 2],
        coefficients: [-3.0, 0.0, 0.0, 1.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-3, 0, 2, 4, 2],
        coefficients: [3.0, 0.0, 0.0, -1.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-3, 0, 2, -2, 1],
        coefficients: [-4.0, 0.0, 0.0, 2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, -1, 0, -2, 1],
        coefficients: [8.0, 0.0, 0.0, -4.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-3, 0, 0, 0, 2],
        coefficients: [3.0, 0.0, 0.0, -1.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-3, 0, -2, 2, 0],
        coefficients: [11.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 1, 0, -4, 1],
        coefficients: [-6.0, 0.0, 0.0, 3.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, 1, 0, -2, 1],
        coefficients: [-4.0, 0.0, 0.0, 2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-4, 0, 0, 0, 1],
        coefficients: [-8.0, 0.0, 0.0, 4.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 0, 0, -4, 1],
        coefficients: [-7.0, 0.0, 0.0, 3.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-3, 0, 0, -2, 1],
        coefficients: [-4.0, 0.0, 0.0, 2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, 0, 3, 2],
        coefficients: [3.0, 0.0, 0.0, -1.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 1, 0, 4, 1],
        coefficients: [6.0, 0.0, 0.0, -3.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, -2, 2, 0, 1],
        coefficients: [-6.0, 0.0, 0.0, 3.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 1, 0, 3, 0],
        coefficients: [6.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 0, 2, 2, 3],
        coefficients: [6.0, 0.0, 0.0, -1.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, 2, 2, 2],
        coefficients: [5.0, 0.0, 0.0, -2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, 0, 2, 2, 2],
        coefficients: [-5.0, 0.0, 0.0, 2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 1, 2, 2, 0],
        coefficients: [-4.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [3, 0, 0, 0, 2],
        coefficients: [-4.0, 0.0, 0.0, 2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 1, 0, 1, 0],
        coefficients: [4.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, -1, 2, -1, 2],
        coefficients: [6.0, 0.0, 0.0, -3.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, 2, 0, 1],
        coefficients: [-4.0, 0.0, 0.0, 2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, 3, 0, 3],
        coefficients: [0.0, 0.0, -26.0, 0.0, 0.0, -11.0],
    },
    SeriesTerm {
        argument: [0, 0, 3, 0, 2],
        coefficients: [0.0, 0.0, -10.0, 0.0, 0.0, -5.0],
    },
    SeriesTerm {
        argument: [-1, 2, 2, 2, 1],
        coefficients: [5.0, 0.0, 0.0, -3.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 0, 4, 0, 0],
        coefficients: [-13.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 2, 2, 0, 1],
        coefficients: [3.0, 0.0, 0.0, -2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [3, 1, 2, -2, 1],
        coefficients: [4.0, 0.0, 0.0, -2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 1, 4, -2, 2],
        coefficients: [7.0, 0.0, 0.0, -3.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, -1, 0, 6, 0],
        coefficients: [4.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, -2, 0, 4, 0],
        coefficients: [5.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, 0, 0, 6, 1],
        coefficients: [-3.0, 0.0, 0.0, 2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, -2, 2, 4, 2],
        coefficients: [-6.0, 0.0, 0.0, 2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, -3, 2, 2, 2],
        coefficients: [-5.0, 0.0, 0.0, 2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, 0, 4, 2],
        coefficients: [-7.0, 0.0, 0.0, 3.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, -1, 2, 3, 2],
        coefficients: [5.0, 0.0, 0.0, -2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, 0, 2, 4, 0],
        coefficients: [13.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, -1, 0, 2, 1],
        coefficients: [-4.0, 0.0, 0.0, 2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 0, 0, 3, 0],
        coefficients: [-3.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 1, 0, 4, 1],
        coefficients: [5.0, 0.0, 0.0, -2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 1, 0, 4, 0],
        coefficients: [-11.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, -1, 2, 1, 2],
        coefficients: [5.0, 0.0, 0.0, -2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, 2, 2, 3],
        coefficients: [4.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 0, 2, 2, 2],
        coefficients: [4.0, 0.0, 0.0, -2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 0, 2, 2, 2],
        coefficients: [-4.0, 0.0, 0.0, 2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, 0, 4, 2, 1],
        coefficients: [6.0, 0.0, 0.0, -3.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 1, 0, 2, 1],
        coefficients: [3.0, 0.0, 0.0, -2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 1, 0, 2, 0],
        coefficients: [-12.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, -1, 2, 0, 0],
        coefficients: [4.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 0, 2, 1, 0],
        coefficients: [-3.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 1, 2, 2, 0],
        coefficients: [-4.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 0, 2, 0, 3],
        coefficients: [3.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [3, 0, 2, 0, 2],
        coefficients: [3.0, 0.0, 0.0, -1.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 0, 2, 0, 2],
        coefficients: [-3.0, 0.0, 0.0, 1.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 0, 3, 0, 3],
        coefficients: [0.0, 0.0, -5.0, 0.0, 0.0, -2.0],
    },
    SeriesTerm {
        argument: [1, 1, 2, 1, 1],
        coefficients: [-7.0, 0.0, 0.0, 4.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 2, 2, 2, 2],
        coefficients: [6.0, 0.0, 0.0, -3.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 1, 2, 0, 0],
        coefficients: [-3.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 0, 4, -2, 1],
        coefficients: [5.0, 0.0, 0.0, -3.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [4, 1, 2, -2, 2],
        coefficients: [3.0, 0.0, 0.0, -1.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, -1, 0, 6, 0],
        coefficients: [3.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-3, -1, 2, 6, 2],
        coefficients: [-3.0, 0.0, 0.0, 1.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 0, 0, 6, 1],
        coefficients: [-5.0, 0.0, 0.0, 3.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-3, 0, 2, 6, 1],
        coefficients: [-3.0, 0.0, 0.0, 2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, -1, 0, 4, 1],
        coefficients: [-3.0, 0.0, 0.0, 2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, -1, 0, 4, 0],
        coefficients: [12.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, 0, 2, 5, 2],
        coefficients: [3.0, 0.0, 0.0, -1.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, -2, 2, 2, 1],
        coefficients: [-4.0, 0.0, 0.0, 2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [3, -1, 0, 2, 0],
        coefficients: [4.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, -1, 2, 2, 0],
        coefficients: [6.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, 2, 3, 1],
        coefficients: [5.0, 0.0, 0.0, -3.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 1, 2, 4, 1],
        coefficients: [4.0, 0.0, 0.0, -2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 1, 2, 3, 2],
        coefficients: [-6.0, 0.0, 0.0, 3.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 0, 4, 2, 1],
        coefficients: [4.0, 0.0, 0.0, -2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 0, 2, 1, 1],
        coefficients: [6.0, 0.0, 0.0, -3.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [5, 0, 0, 0, 0],
        coefficients: [6.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 1, 2, 1, 2],
        coefficients: [-6.0, 0.0, 0.0, 3.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 0, 4, 0, 1],
        coefficients: [3.0, 0.0, 0.0, -2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [3, 1, 2, 0, 1],
        coefficients: [7.0, 0.0, 0.0, -4.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [3, 0, 4, -2, 2],
        coefficients: [4.0, 0.0, 0.0, -2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, -1, 2, 6, 2],
        coefficients: [-5.0, 0.0, 0.0, 2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, 0, 6, 0],
        coefficients: [5.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, -2, 2, 4, 2],
        coefficients: [-6.0, 0.0, 0.0, 3.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-2, 0, 2, 6, 1],
        coefficients: [-6.0, 0.0, 0.0, 3.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 0, 0, 4, 1],
        coefficients: [-4.0, 0.0, 0.0, 2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 0, 0, 4, 0],
        coefficients: [10.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, -2, 2, 2, 2],
        coefficients: [-4.0, 0.0, 0.0, 2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, 2, 4, 0],
        coefficients: [7.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 0, 2, 3, 2],
        coefficients: [7.0, 0.0, 0.0, -3.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [4, 0, 0, 2, 0],
        coefficients: [4.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 0, 2, 2, 0],
        coefficients: [11.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [0, 0, 4, 2, 2],
        coefficients: [5.0, 0.0, 0.0, -2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [4, -1, 2, 0, 2],
        coefficients: [-6.0, 0.0, 0.0, 2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [3, 0, 2, 1, 2],
        coefficients: [4.0, 0.0, 0.0, -2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 1, 2, 2, 1],
        coefficients: [3.0, 0.0, 0.0, -2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [4, 1, 2, 0, 2],
        coefficients: [5.0, 0.0, 0.0, -2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, -1, 2, 6, 2],
        coefficients: [-4.0, 0.0, 0.0, 2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [-1, 0, 2, 6, 1],
        coefficients: [-4.0, 0.0, 0.0, 2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, -1, 2, 4, 1],
        coefficients: [-3.0, 0.0, 0.0, 2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [1, 1, 2, 4, 2],
        coefficients: [4.0, 0.0, 0.0, -2.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [3, 1, 2, 2, 2],
        coefficients: [3.0, 0.0, 0.0, -1.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [5, 0, 2, 0, 1],
        coefficients: [-3.0, 0.0, 0.0, 1.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, -1, 2, 4, 2],
        coefficients: [-3.0, 0.0, 0.0, 1.0, 0.0, 0.0],
    },
    SeriesTerm {
        argument: [2, 0, 2, 4, 1],
        coefficients: [-3.0, 0.0, 0.0, 2.0, 0.0, 0.0],
    },
];

/// The planetary terms of the series.
///
/// The multipliers apply to the arguments `l`, `F`, `D`, `Om`, the mean
/// longitudes of Mercury through Uranus, the mean longitude of Neptune,
/// and the general accumulated precession, in that order. The
/// coefficients are the longitude sine, the longitude cosine, the
/// obliquity sine, and the obliquity cosine, in units of 0.1
/// microarcsecond.
pub static PLANETARY: [PlanetaryTerm; 687] = [
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 8, -16, 4, 5, 0, 0, 0],
        coefficients: [1440.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -8, 16, -4, -5, 0, 0, 2],
        coefficients: [56.0, -117.0, -42.0, -40.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 8, -16, 4, 5, 0, 0, 2],
        coefficients: [125.0, -43.0, 0.0, -54.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 2, 2],
        coefficients: [0.0, 5.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -4, 8, -1, -5, 0, 0, 2],
        coefficients: [3.0, -7.0, -3.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 4, -8, 3, 0, 0, 0, 1],
        coefficients: [3.0, 0.0, 0.0, -2.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 1, 0, 0, 3, -8, 3, 0, 0, 0, 0],
        coefficients: [-114.0, 0.0, 0.0, 61.0],
    },
    PlanetaryTerm {
        argument: [-1, 0, 0, 0, 0, 10, -3, 0, 0, 0, 0, 0, 0],
        coefficients: [-219.0, 89.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 0, 0, -2, 6, -3, 0, 2],
        coefficients: [-3.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 4, -8, 3, 0, 0, 0, 0],
        coefficients: [-462.0, 1604.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 1, 0, 0, -5, 8, -3, 0, 0, 0, 0],
        coefficients: [99.0, 0.0, 0.0, -53.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -4, 8, -3, 0, 0, 0, 1],
        coefficients: [-3.0, 0.0, 0.0, 2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 4, -8, 1, 5, 0, 0, 2],
        coefficients: [0.0, 6.0, 2.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -5, 6, 4, 0, 0, 0, 0, 2],
        coefficients: [3.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 0, 0, 2, -5, 0, 0, 2],
        coefficients: [-12.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 0, 0, 2, -5, 0, 0, 1],
        coefficients: [14.0, -218.0, 117.0, 8.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 1, 0, 0, -1, 0, 2, -5, 0, 0, 0],
        coefficients: [31.0, -481.0, -257.0, -17.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 0, 0, 2, -5, 0, 0, 0],
        coefficients: [-491.0, 128.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 1, 0, 0, -1, 0, -2, 5, 0, 0, 0],
        coefficients: [-3084.0, 5123.0, 2735.0, 1647.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 0, 0, -2, 5, 0, 0, 1],
        coefficients: [-1444.0, 2409.0, -1286.0, -771.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 0, 0, -2, 5, 0, 0, 2],
        coefficients: [11.0, -24.0, -11.0, -9.0],
    },
    PlanetaryTerm {
        argument: [2, -1, -1, 0, 0, 0, 3, -7, 0, 0, 0, 0, 0],
        coefficients: [26.0, -9.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [1, 0, -2, 0, 0, 19, -21, 3, 0, 0, 0, 0, 0],
        coefficients: [103.0, -60.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 1, 0, 2, -4, 0, -3, 0, 0, 0, 0],
        coefficients: [0.0, -13.0, -7.0, 0.0],
    },
    PlanetaryTerm {
        argument: [1, 0, -1, 1, 0, 0, -1, 0, 2, 0, 0, 0, 0],
        coefficients: [-26.0, -29.0, -16.0, 14.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 1, 0, 0, -1, 0, -4, 10, 0, 0, 0],
        coefficients: [9.0, -27.0, -14.0, -5.0],
    },
    PlanetaryTerm {
        argument: [-2, 0, 2, 1, 0, 0, 2, 0, 0, -5, 0, 0, 0],
        coefficients: [12.0, 0.0, 0.0, -6.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 3, -7, 4, 0, 0, 0, 0, 0],
        coefficients: [-7.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, -1, 1, 0, 0, 0, 1, 0, 1, -1, 0, 0, 0],
        coefficients: [0.0, 24.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [-2, 0, 2, 1, 0, 0, 2, 0, -2, 0, 0, 0, 0],
        coefficients: [284.0, 0.0, 0.0, -151.0],
    },
    PlanetaryTerm {
        argument: [-1, 0, 0, 0, 0, 18, -16, 0, 0, 0, 0, 0, 0],
        coefficients: [226.0, 101.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [-2, 1, 1, 2, 0, 0, 1, 0, -2, 0, 0, 0, 0],
        coefficients: [0.0, -8.0, -2.0, 0.0],
    },
    PlanetaryTerm {
        argument: [-1, 1, -1, 1, 0, 18, -17, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, -6.0, -3.0, 0.0],
    },
    PlanetaryTerm {
        argument: [-1, 0, 1, 1, 0, 0, 2, -2, 0, 0, 0, 0, 0],
        coefficients: [5.0, 0.0, 0.0, -3.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -8, 13, 0, 0, 0, 0, 0, 2],
        coefficients: [-41.0, 175.0, 76.0, 17.0],
    },
    PlanetaryTerm {
        argument: [0, 2, -2, 2, 0, -8, 11, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, 15.0, 6.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -8, 13, 0, 0, 0, 0, 0, 1],
        coefficients: [425.0, 212.0, -133.0, 269.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 1, 0, -8, 12, 0, 0, 0, 0, 0, 0],
        coefficients: [1200.0, 598.0, 319.0, -641.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 8, -13, 0, 0, 0, 0, 0, 0],
        coefficients: [235.0, 334.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 1, 0, 8, -14, 0, 0, 0, 0, 0, 0],
        coefficients: [11.0, -12.0, -7.0, -6.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 8, -13, 0, 0, 0, 0, 0, 1],
        coefficients: [5.0, -6.0, 3.0, 3.0],
    },
    PlanetaryTerm {
        argument: [-2, 0, 2, 1, 0, 0, 2, 0, -4, 5, 0, 0, 0],
        coefficients: [-5.0, 0.0, 0.0, 3.0],
    },
    PlanetaryTerm {
        argument: [-2, 0, 2, 2, 0, 3, -3, 0, 0, 0, 0, 0, 0],
        coefficients: [6.0, 0.0, 0.0, -3.0],
    },
    PlanetaryTerm {
        argument: [-2, 0, 2, 0, 0, 0, 2, 0, -3, 1, 0, 0, 0],
        coefficients: [15.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 1, 0, 3, -5, 0, 2, 0, 0, 0, 0],
        coefficients: [13.0, 0.0, 0.0, -7.0],
    },
    PlanetaryTerm {
        argument: [-2, 0, 2, 0, 0, 0, 2, 0, -4, 3, 0, 0, 0],
        coefficients: [-6.0, -9.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, -1, 1, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0],
        coefficients: [266.0, -78.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 1, 0, 0, -1, 2, 0, 0, 0, 0, 0],
        coefficients: [-460.0, -435.0, -232.0, 246.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 2, 0, 0, -2, 2, 0, 0, 0, 0, 0],
        coefficients: [0.0, 15.0, 7.0, 0.0],
    },
    PlanetaryTerm {
        argument: [-1, 1, 0, 1, 0, 3, -5, 0, 0, 0, 0, 0, 0],
        coefficients: [-3.0, 0.0, 0.0, 2.0],
    },
    PlanetaryTerm {
        argument: [-1, 0, 1, 0, 0, 3, -4, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, 131.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [-2, 0, 2, 0, 0, 0, 2, 0, -2, -2, 0, 0, 0],
        coefficients: [4.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [-2, 2, 0, 2, 0, 0, -5, 9, 0, 0, 0, 0, 0],
        coefficients: [0.0, 3.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 1, 0, 0, -1, 0, 0, 0, -1, 0, 0],
        coefficients: [0.0, 4.0, 2.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0],
        coefficients: [0.0, 3.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 1, 0, 0, -1, 0, 0, 0, 0, 2, 0],
        coefficients: [-17.0, -19.0, -10.0, 9.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 1],
        coefficients: [-9.0, -11.0, 6.0, -5.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 2],
        coefficients: [-6.0, 0.0, 0.0, 3.0],
    },
    PlanetaryTerm {
        argument: [-1, 0, 1, 0, 0, 0, 3, -4, 0, 0, 0, 0, 0],
        coefficients: [-16.0, 8.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, -1, 1, 0, 0, 0, 1, 0, 0, 2, 0, 0, 0],
        coefficients: [0.0, 3.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 2, 0, 0, -1, 0, 0, 2, 0, 0, 0],
        coefficients: [11.0, 24.0, 11.0, -5.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 1, 0, 0, -9, 17, 0, 0, 0, 0, 0],
        coefficients: [-3.0, -4.0, -2.0, 1.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 2, 0, -3, 5, 0, 0, 0, 0, 0, 0],
        coefficients: [3.0, 0.0, 0.0, -1.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 1, 0, 0, -1, 0, -1, 2, 0, 0, 0],
        coefficients: [0.0, -8.0, -4.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 0, 0, 1, -2, 0, 0, 0],
        coefficients: [0.0, 3.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [1, 0, -2, 0, 0, 17, -16, 0, -2, 0, 0, 0, 0],
        coefficients: [0.0, 5.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 1, 0, 0, -1, 0, 1, -3, 0, 0, 0],
        coefficients: [0.0, 3.0, 2.0, 0.0],
    },
    PlanetaryTerm {
        argument: [-2, 0, 2, 1, 0, 0, 5, -6, 0, 0, 0, 0, 0],
        coefficients: [-6.0, 4.0, 2.0, 3.0],
    },
    PlanetaryTerm {
        argument: [0, -2, 2, 0, 0, 0, 9, -13, 0, 0, 0, 0, 0],
        coefficients: [-3.0, -5.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 2, 0, 0, -1, 0, 0, 1, 0, 0, 0],
        coefficients: [-5.0, 0.0, 0.0, 2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0],
        coefficients: [4.0, 24.0, 13.0, -2.0],
    },
    PlanetaryTerm {
        argument: [0, -1, 1, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0],
        coefficients: [-42.0, 20.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, -2, 2, 0, 0, 5, -6, 0, 0, 0, 0, 0, 0],
        coefficients: [-10.0, 233.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, -1, 1, 1, 0, 5, -7, 0, 0, 0, 0, 0, 0],
        coefficients: [-3.0, 0.0, 0.0, 1.0],
    },
    PlanetaryTerm {
        argument: [-2, 0, 2, 0, 0, 6, -8, 0, 0, 0, 0, 0, 0],
        coefficients: [78.0, -18.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [2, 1, -3, 1, 0, -6, 7, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, 3.0, 1.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 2, 0, 0, 0, 0, 1, 0, 0, 0, 0],
        coefficients: [0.0, -3.0, -1.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, -1, 1, 1, 0, 0, 1, 0, 1, 0, 0, 0, 0],
        coefficients: [0.0, -4.0, -2.0, 1.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 1, 0, 0, -1, 0, 0, 0, 2, 0, 0],
        coefficients: [0.0, -8.0, -4.0, -1.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 1],
        coefficients: [0.0, -5.0, 3.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 2],
        coefficients: [-7.0, 0.0, 0.0, 3.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -8, 15, 0, 0, 0, 0, 2],
        coefficients: [-14.0, 8.0, 3.0, 6.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -8, 15, 0, 0, 0, 0, 1],
        coefficients: [0.0, 8.0, -4.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 1, 0, 0, -9, 15, 0, 0, 0, 0, 0],
        coefficients: [0.0, 19.0, 10.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 8, -15, 0, 0, 0, 0, 0],
        coefficients: [45.0, -22.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [1, -1, -1, 0, 0, 0, 8, -15, 0, 0, 0, 0, 0],
        coefficients: [-3.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [2, 0, -2, 0, 0, 2, -5, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, -3.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [-2, 0, 2, 0, 0, 0, 2, 0, -5, 5, 0, 0, 0],
        coefficients: [0.0, 3.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [2, 0, -2, 1, 0, 0, -6, 8, 0, 0, 0, 0, 0],
        coefficients: [3.0, 5.0, 3.0, -2.0],
    },
    PlanetaryTerm {
        argument: [2, 0, -2, 1, 0, 0, -2, 0, 3, 0, 0, 0, 0],
        coefficients: [89.0, -16.0, -9.0, -48.0],
    },
    PlanetaryTerm {
        argument: [-2, 1, 1, 0, 0, 0, 1, 0, -3, 0, 0, 0, 0],
        coefficients: [0.0, 3.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [-2, 1, 1, 1, 0, 0, 1, 0, -3, 0, 0, 0, 0],
        coefficients: [-3.0, 7.0, 4.0, 2.0],
    },
    PlanetaryTerm {
        argument: [-2, 0, 2, 0, 0, 0, 2, 0, -3, 0, 0, 0, 0],
        coefficients: [-349.0, -62.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [-2, 0, 2, 0, 0, 0, 6, -8, 0, 0, 0, 0, 0],
        coefficients: [-15.0, 22.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [-2, 0, 2, 0, 0, 0, 2, 0, -1, -5, 0, 0, 0],
        coefficients: [-3.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [-1, 0, 1, 0, 0, 0, 1, 0, -1, 0, 0, 0, 0],
        coefficients: [-53.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [-1, 1, 1, 1, 0, -20, 20, 0, 0, 0, 0, 0, 0],
        coefficients: [5.0, 0.0, 0.0, -3.0],
    },
    PlanetaryTerm {
        argument: [1, 0, -2, 0, 0, 20, -21, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, -8.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 1, 0, 0, 8, -15, 0, 0, 0, 0, 0],
        coefficients: [15.0, -7.0, -4.0, -8.0],
    },
    PlanetaryTerm {
        argument: [0, 2, -2, 1, 0, 0, -10, 15, 0, 0, 0, 0, 0],
        coefficients: [-3.0, 0.0, 0.0, 1.0],
    },
    PlanetaryTerm {
        argument: [0, -1, 1, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0],
        coefficients: [-21.0, -78.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0],
        coefficients: [20.0, -70.0, -37.0, -11.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 2, 0, 0, -1, 0, 1, 0, 0, 0, 0],
        coefficients: [0.0, 6.0, 3.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 1, 0, 0, -1, 0, -2, 4, 0, 0, 0],
        coefficients: [5.0, 3.0, 2.0, -2.0],
    },
    PlanetaryTerm {
        argument: [2, 0, -2, 1, 0, -6, 8, 0, 0, 0, 0, 0, 0],
        coefficients: [-17.0, -4.0, -2.0, 9.0],
    },
    PlanetaryTerm {
        argument: [0, -2, 2, 1, 0, 5, -6, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, 6.0, 3.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 1],
        coefficients: [32.0, 15.0, -8.0, 17.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 1, 0, 0, -1, 0, 0, -1, 0, 0, 0],
        coefficients: [174.0, 84.0, 45.0, -93.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0],
        coefficients: [11.0, 56.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 1, 0, 0, -1, 0, 0, 1, 0, 0, 0],
        coefficients: [-66.0, -12.0, -6.0, 35.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1],
        coefficients: [47.0, 8.0, 4.0, -25.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 2],
        coefficients: [0.0, 8.0, 4.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 2, -2, 1, 0, 0, -9, 13, 0, 0, 0, 0, 0],
        coefficients: [10.0, -22.0, -12.0, -5.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 1, 0, 0, 7, -13, 0, 0, 0, 0, 0],
        coefficients: [-3.0, 0.0, 0.0, 2.0],
    },
    PlanetaryTerm {
        argument: [-2, 0, 2, 0, 0, 0, 5, -6, 0, 0, 0, 0, 0],
        coefficients: [-24.0, 12.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 9, -17, 0, 0, 0, 0, 0],
        coefficients: [5.0, -6.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -9, 17, 0, 0, 0, 0, 2],
        coefficients: [3.0, 0.0, 0.0, -2.0],
    },
    PlanetaryTerm {
        argument: [1, 0, -1, 1, 0, 0, -3, 4, 0, 0, 0, 0, 0],
        coefficients: [4.0, 3.0, 1.0, -2.0],
    },
    PlanetaryTerm {
        argument: [1, 0, -1, 1, 0, -3, 4, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, 29.0, 15.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 2, 0, 0, -1, 2, 0, 0, 0, 0, 0],
        coefficients: [-5.0, -4.0, -2.0, 2.0],
    },
    PlanetaryTerm {
        argument: [0, -1, 1, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0],
        coefficients: [8.0, -3.0, -1.0, -5.0],
    },
    PlanetaryTerm {
        argument: [0, -2, 2, 0, 1, 0, -2, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, -3.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 3, -5, 0, 2, 0, 0, 0, 0],
        coefficients: [10.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [-2, 0, 2, 1, 0, 0, 2, 0, -3, 1, 0, 0, 0],
        coefficients: [3.0, 0.0, 0.0, -2.0],
    },
    PlanetaryTerm {
        argument: [-2, 0, 2, 1, 0, 3, -3, 0, 0, 0, 0, 0, 0],
        coefficients: [-5.0, 0.0, 0.0, 3.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 1, 0, 8, -13, 0, 0, 0, 0, 0, 0],
        coefficients: [46.0, 66.0, 35.0, -25.0],
    },
    PlanetaryTerm {
        argument: [0, -1, 1, 0, 0, 8, -12, 0, 0, 0, 0, 0, 0],
        coefficients: [-14.0, 7.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 2, -2, 1, 0, -8, 11, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, 3.0, 2.0, 0.0],
    },
    PlanetaryTerm {
        argument: [-1, 0, 1, 0, 0, 0, 2, -2, 0, 0, 0, 0, 0],
        coefficients: [-5.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [-1, 0, 0, 1, 0, 18, -16, 0, 0, 0, 0, 0, 0],
        coefficients: [-68.0, -34.0, -18.0, 36.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 1, 0, 0, -1, 0, -1, 1, 0, 0, 0],
        coefficients: [0.0, 14.0, 7.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 1, 0, 3, -7, 4, 0, 0, 0, 0, 0],
        coefficients: [10.0, -6.0, -3.0, -5.0],
    },
    PlanetaryTerm {
        argument: [-2, 1, 1, 1, 0, 0, -3, 7, 0, 0, 0, 0, 0],
        coefficients: [-5.0, -4.0, -2.0, 3.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 2, 0, 0, -1, 0, -2, 5, 0, 0, 0],
        coefficients: [-3.0, 5.0, 2.0, 1.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 1, 0, 0, 0, 0, -2, 5, 0, 0, 0],
        coefficients: [76.0, 17.0, 9.0, -41.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 1, 0, 0, -4, 8, -3, 0, 0, 0, 0],
        coefficients: [84.0, 298.0, 159.0, -45.0],
    },
    PlanetaryTerm {
        argument: [1, 0, 0, 1, 0, -10, 3, 0, 0, 0, 0, 0, 0],
        coefficients: [3.0, 0.0, 0.0, -1.0],
    },
    PlanetaryTerm {
        argument: [0, 2, -2, 1, 0, 0, -2, 0, 0, 0, 0, 0, 0],
        coefficients: [-3.0, 0.0, 0.0, 2.0],
    },
    PlanetaryTerm {
        argument: [-1, 0, 0, 1, 0, 10, -3, 0, 0, 0, 0, 0, 0],
        coefficients: [-3.0, 0.0, 0.0, 1.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 1, 0, 0, 4, -8, 3, 0, 0, 0, 0],
        coefficients: [-82.0, 292.0, 156.0, 44.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 1, 0, 0, 0, 0, 2, -5, 0, 0, 0],
        coefficients: [-73.0, 17.0, 9.0, 39.0],
    },
    PlanetaryTerm {
        argument: [0, -1, 1, 0, 0, 0, 1, 0, 2, -5, 0, 0, 0],
        coefficients: [-9.0, -16.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [2, -1, -1, 1, 0, 0, 3, -7, 0, 0, 0, 0, 0],
        coefficients: [3.0, 0.0, -1.0, -2.0],
    },
    PlanetaryTerm {
        argument: [-2, 0, 2, 0, 0, 0, 2, 0, 0, -5, 0, 0, 0],
        coefficients: [-3.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 1, 0, -3, 7, -4, 0, 0, 0, 0, 0],
        coefficients: [-9.0, -5.0, -3.0, 5.0],
    },
    PlanetaryTerm {
        argument: [-2, 0, 2, 0, 0, 0, 2, 0, -2, 0, 0, 0, 0],
        coefficients: [-439.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [1, 0, 0, 1, 0, -18, 16, 0, 0, 0, 0, 0, 0],
        coefficients: [57.0, -28.0, -15.0, -30.0],
    },
    PlanetaryTerm {
        argument: [-2, 1, 1, 1, 0, 0, 1, 0, -2, 0, 0, 0, 0],
        coefficients: [0.0, -6.0, -3.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 2, 0, -8, 12, 0, 0, 0, 0, 0, 0],
        coefficients: [-4.0, 0.0, 0.0, 2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 1, 0, -8, 13, 0, 0, 0, 0, 0, 0],
        coefficients: [-40.0, 57.0, 30.0, 21.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 1, -2, 0, 0, 0, 0, 1],
        coefficients: [23.0, 7.0, 3.0, -13.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 1, 0, 0, 0, -2, 0, 0, 0, 0, 0],
        coefficients: [273.0, 80.0, 43.0, -146.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 1, -2, 0, 0, 0, 0, 0],
        coefficients: [-449.0, 430.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 1, 0, 0, -2, 2, 0, 0, 0, 0, 0],
        coefficients: [-8.0, -47.0, -25.0, 4.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -1, 2, 0, 0, 0, 0, 1],
        coefficients: [6.0, 47.0, 25.0, -3.0],
    },
    PlanetaryTerm {
        argument: [-1, 0, 1, 1, 0, 3, -4, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, 23.0, 13.0, 0.0],
    },
    PlanetaryTerm {
        argument: [-1, 0, 1, 1, 0, 0, 3, -4, 0, 0, 0, 0, 0],
        coefficients: [-3.0, 0.0, 0.0, 2.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 1, 0, 0, -1, 0, 0, -2, 0, 0, 0],
        coefficients: [3.0, -4.0, -2.0, -2.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 1, 0, 0, -1, 0, 0, 2, 0, 0, 0],
        coefficients: [-48.0, -110.0, -59.0, 26.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 1],
        coefficients: [51.0, 114.0, 61.0, -27.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 2],
        coefficients: [-133.0, 0.0, 0.0, 57.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 0, 0, 3, -6, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, 4.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 1, 0, -3, 5, 0, 0, 0, 0, 0, 0],
        coefficients: [-21.0, -6.0, -3.0, 11.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 2, 0, -3, 4, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, -3.0, -1.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 1, 0, 0, -2, 4, 0, 0, 0, 0, 0],
        coefficients: [-11.0, -21.0, -11.0, 6.0],
    },
    PlanetaryTerm {
        argument: [0, 2, -2, 1, 0, -5, 6, 0, 0, 0, 0, 0, 0],
        coefficients: [-18.0, -436.0, -233.0, 9.0],
    },
    PlanetaryTerm {
        argument: [0, -1, 1, 0, 0, 5, -7, 0, 0, 0, 0, 0, 0],
        coefficients: [35.0, -7.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 1, 0, 5, -8, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, 5.0, 3.0, 0.0],
    },
    PlanetaryTerm {
        argument: [-2, 0, 2, 1, 0, 6, -8, 0, 0, 0, 0, 0, 0],
        coefficients: [11.0, -3.0, -1.0, -6.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 1, 0, 0, -8, 15, 0, 0, 0, 0, 0],
        coefficients: [-5.0, -3.0, -1.0, 3.0],
    },
    PlanetaryTerm {
        argument: [-2, 0, 2, 1, 0, 0, 2, 0, -3, 0, 0, 0, 0],
        coefficients: [-53.0, -9.0, -5.0, 28.0],
    },
    PlanetaryTerm {
        argument: [-2, 0, 2, 1, 0, 0, 6, -8, 0, 0, 0, 0, 0],
        coefficients: [0.0, 3.0, 2.0, 1.0],
    },
    PlanetaryTerm {
        argument: [1, 0, -1, 1, 0, 0, -1, 0, 1, 0, 0, 0, 0],
        coefficients: [4.0, 0.0, 0.0, -2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 0, 0, 3, -5, 0, 0, 0],
        coefficients: [0.0, -4.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 1, 0, 0, -1, 0, -1, 0, 0, 0, 0],
        coefficients: [-50.0, 194.0, 103.0, 27.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 1],
        coefficients: [-13.0, 52.0, 28.0, 7.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0],
        coefficients: [-91.0, 248.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1],
        coefficients: [6.0, 49.0, 26.0, -3.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 1, 0, 0, -1, 0, 1, 0, 0, 0, 0],
        coefficients: [-6.0, -47.0, -25.0, 3.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1],
        coefficients: [0.0, 5.0, 3.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2],
        coefficients: [52.0, 23.0, 10.0, -23.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 2, 0, 0, -1, 0, 0, -1, 0, 0, 0],
        coefficients: [-3.0, 0.0, 0.0, 1.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 1, 0, 0, 0, 0, 0, -1, 0, 0, 0],
        coefficients: [0.0, 5.0, 3.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, -1, 1, 0, 0, 0, 1, 0, 0, -1, 0, 0, 0],
        coefficients: [-4.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -7, 13, 0, 0, 0, 0, 2],
        coefficients: [-4.0, 8.0, 3.0, 2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 7, -13, 0, 0, 0, 0, 0],
        coefficients: [10.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [2, 0, -2, 1, 0, 0, -5, 6, 0, 0, 0, 0, 0],
        coefficients: [3.0, 0.0, 0.0, -2.0],
    },
    PlanetaryTerm {
        argument: [0, 2, -2, 1, 0, 0, -8, 11, 0, 0, 0, 0, 0],
        coefficients: [0.0, 8.0, 4.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 2, -2, 1, -1, 0, 2, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, 8.0, 4.0, 1.0],
    },
    PlanetaryTerm {
        argument: [-2, 0, 2, 0, 0, 0, 4, -4, 0, 0, 0, 0, 0],
        coefficients: [-4.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 0, 0, 2, -2, 0, 0, 0],
        coefficients: [-4.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 1, 0, 0, -1, 0, 0, 3, 0, 0, 0],
        coefficients: [-8.0, 4.0, 2.0, 4.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 1],
        coefficients: [8.0, -4.0, -2.0, -4.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 2],
        coefficients: [0.0, 15.0, 7.0, 0.0],
    },
    PlanetaryTerm {
        argument: [-2, 0, 2, 0, 0, 3, -3, 0, 0, 0, 0, 0, 0],
        coefficients: [-138.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 2, 0, 0, -4, 8, -3, 0, 0, 0, 0],
        coefficients: [0.0, -7.0, -3.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 2, 0, 0, 4, -8, 3, 0, 0, 0, 0],
        coefficients: [0.0, -7.0, -3.0, 0.0],
    },
    PlanetaryTerm {
        argument: [2, 0, -2, 1, 0, 0, -2, 0, 2, 0, 0, 0, 0],
        coefficients: [54.0, 0.0, 0.0, -29.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 2, 0, 0, -1, 0, 2, 0, 0, 0, 0],
        coefficients: [0.0, 10.0, 4.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 2, 0, 0, 0, -2, 0, 0, 0, 0, 0],
        coefficients: [-7.0, 0.0, 0.0, 3.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 1, 0, 0, 1, -2, 0, 0, 0, 0, 0],
        coefficients: [-37.0, 35.0, 19.0, 20.0],
    },
    PlanetaryTerm {
        argument: [0, -1, 1, 0, 0, 0, 2, -2, 0, 0, 0, 0, 0],
        coefficients: [0.0, 4.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, -1, 1, 0, 0, 0, 1, 0, 0, -2, 0, 0, 0],
        coefficients: [-4.0, 9.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 2, -2, 1, 0, 0, -2, 0, 0, 2, 0, 0, 0],
        coefficients: [8.0, 0.0, 0.0, -4.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 1, 0, 3, -6, 0, 0, 0, 0, 0, 0],
        coefficients: [-9.0, -14.0, -8.0, 5.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 3, -5, 0, 0, 0, 0, 0, 1],
        coefficients: [-3.0, -9.0, -5.0, 3.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 3, -5, 0, 0, 0, 0, 0, 0],
        coefficients: [-145.0, 47.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 1, 0, -3, 4, 0, 0, 0, 0, 0, 0],
        coefficients: [-10.0, 40.0, 21.0, 5.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -3, 5, 0, 0, 0, 0, 0, 1],
        coefficients: [11.0, -49.0, -26.0, -7.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -3, 5, 0, 0, 0, 0, 0, 2],
        coefficients: [-2150.0, 0.0, 0.0, 932.0],
    },
    PlanetaryTerm {
        argument: [0, 2, -2, 2, 0, -3, 3, 0, 0, 0, 0, 0, 0],
        coefficients: [-12.0, 0.0, 0.0, 5.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -3, 5, 0, 0, 0, 0, 0, 2],
        coefficients: [85.0, 0.0, 0.0, -37.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 2, -4, 0, 0, 0, 0, 1],
        coefficients: [4.0, 0.0, 0.0, -2.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 1, 0, 0, 1, -4, 0, 0, 0, 0, 0],
        coefficients: [3.0, 0.0, 0.0, -2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 2, -4, 0, 0, 0, 0, 0],
        coefficients: [-86.0, 153.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -2, 4, 0, 0, 0, 0, 1],
        coefficients: [-6.0, 9.0, 5.0, 3.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 1, 0, 0, -3, 4, 0, 0, 0, 0, 0],
        coefficients: [9.0, -13.0, -7.0, -5.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -2, 4, 0, 0, 0, 0, 1],
        coefficients: [-8.0, 12.0, 6.0, 4.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -2, 4, 0, 0, 0, 0, 2],
        coefficients: [-51.0, 0.0, 0.0, 22.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -5, 8, 0, 0, 0, 0, 0, 2],
        coefficients: [-11.0, -268.0, -116.0, 5.0],
    },
    PlanetaryTerm {
        argument: [0, 2, -2, 2, 0, -5, 6, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, 12.0, 5.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -5, 8, 0, 0, 0, 0, 0, 2],
        coefficients: [0.0, 7.0, 3.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -5, 8, 0, 0, 0, 0, 0, 1],
        coefficients: [31.0, 6.0, 3.0, -17.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 1, 0, -5, 7, 0, 0, 0, 0, 0, 0],
        coefficients: [140.0, 27.0, 14.0, -75.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -5, 8, 0, 0, 0, 0, 0, 1],
        coefficients: [57.0, 11.0, 6.0, -30.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 5, -8, 0, 0, 0, 0, 0, 0],
        coefficients: [-14.0, -39.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 2, 0, 0, -1, 0, -1, 0, 0, 0, 0],
        coefficients: [0.0, -6.0, -2.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 1, 0, 0, 0, 0, -1, 0, 0, 0, 0],
        coefficients: [4.0, 15.0, 8.0, -2.0],
    },
    PlanetaryTerm {
        argument: [0, -1, 1, 0, 0, 0, 1, 0, -1, 0, 0, 0, 0],
        coefficients: [0.0, 4.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 2, -2, 1, 0, 0, -2, 0, 1, 0, 0, 0, 0],
        coefficients: [-3.0, 0.0, 0.0, 1.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -6, 11, 0, 0, 0, 0, 2],
        coefficients: [0.0, 11.0, 5.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 6, -11, 0, 0, 0, 0, 0],
        coefficients: [9.0, 6.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, -1, 0, 4, 0, 0, 0, 0, 0, 2],
        coefficients: [-4.0, 10.0, 4.0, 2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 1, 0, -4, 0, 0, 0, 0, 0, 0],
        coefficients: [5.0, 3.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [2, 0, -2, 1, 0, -3, 3, 0, 0, 0, 0, 0, 0],
        coefficients: [16.0, 0.0, 0.0, -9.0],
    },
    PlanetaryTerm {
        argument: [-2, 0, 2, 0, 0, 0, 2, 0, 0, -2, 0, 0, 0],
        coefficients: [-3.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 2, -2, 1, 0, 0, -7, 9, 0, 0, 0, 0, 0],
        coefficients: [0.0, 3.0, 2.0, -1.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 0, 0, 4, -5, 0, 0, 2],
        coefficients: [7.0, 0.0, 0.0, -3.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0],
        coefficients: [-25.0, 22.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 1],
        coefficients: [42.0, 223.0, 119.0, -22.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 1, 0, 0, -1, 0, 2, 0, 0, 0, 0],
        coefficients: [-27.0, -143.0, -77.0, 14.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 1],
        coefficients: [9.0, 49.0, 26.0, -5.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 2],
        coefficients: [-1166.0, 0.0, 0.0, 505.0],
    },
    PlanetaryTerm {
        argument: [0, 2, -2, 2, 0, 0, -2, 0, 2, 0, 0, 0, 0],
        coefficients: [-5.0, 0.0, 0.0, 2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 2],
        coefficients: [-6.0, 0.0, 0.0, 3.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 1, 0, 3, -5, 0, 0, 0, 0, 0, 0],
        coefficients: [-8.0, 0.0, 1.0, 4.0],
    },
    PlanetaryTerm {
        argument: [0, -1, 1, 0, 0, 3, -4, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, -4.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 2, -2, 1, 0, -3, 3, 0, 0, 0, 0, 0, 0],
        coefficients: [117.0, 0.0, 0.0, -63.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 1, 0, 0, 2, -4, 0, 0, 0, 0, 0],
        coefficients: [-4.0, 8.0, 4.0, 2.0],
    },
    PlanetaryTerm {
        argument: [0, 2, -2, 1, 0, 0, -4, 4, 0, 0, 0, 0, 0],
        coefficients: [3.0, 0.0, 0.0, -2.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 2, 0, -5, 7, 0, 0, 0, 0, 0, 0],
        coefficients: [-5.0, 0.0, 0.0, 2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 3, -6, 0, 0, 0, 0, 0],
        coefficients: [0.0, 31.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -3, 6, 0, 0, 0, 0, 1],
        coefficients: [-5.0, 0.0, 1.0, 3.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 1, 0, 0, -4, 6, 0, 0, 0, 0, 0],
        coefficients: [4.0, 0.0, 0.0, -2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -3, 6, 0, 0, 0, 0, 1],
        coefficients: [-4.0, 0.0, 0.0, 2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -3, 6, 0, 0, 0, 0, 2],
        coefficients: [-24.0, -13.0, -6.0, 10.0],
    },
    PlanetaryTerm {
        argument: [0, -1, 1, 0, 0, 2, -2, 0, 0, 0, 0, 0, 0],
        coefficients: [3.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 1, 0, 2, -3, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, -32.0, -17.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -5, 9, 0, 0, 0, 0, 2],
        coefficients: [8.0, 12.0, 5.0, -3.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -5, 9, 0, 0, 0, 0, 1],
        coefficients: [3.0, 0.0, 0.0, -1.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 5, -9, 0, 0, 0, 0, 0],
        coefficients: [7.0, 13.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, -1, 1, 0, 0, 0, 1, 0, -2, 0, 0, 0, 0],
        coefficients: [-3.0, 16.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 2, -2, 1, 0, 0, -2, 0, 2, 0, 0, 0, 0],
        coefficients: [50.0, 0.0, 0.0, -27.0],
    },
    PlanetaryTerm {
        argument: [-2, 1, 1, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, -5.0, -3.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, -2, 2, 0, 0, 3, -3, 0, 0, 0, 0, 0, 0],
        coefficients: [13.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -6, 10, 0, 0, 0, 0, 0, 1],
        coefficients: [0.0, 5.0, 3.0, 1.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -6, 10, 0, 0, 0, 0, 0, 2],
        coefficients: [24.0, 5.0, 2.0, -11.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -2, 3, 0, 0, 0, 0, 0, 2],
        coefficients: [5.0, -11.0, -5.0, -2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -2, 3, 0, 0, 0, 0, 0, 1],
        coefficients: [30.0, -3.0, -2.0, -16.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 1, 0, -2, 2, 0, 0, 0, 0, 0, 0],
        coefficients: [18.0, 0.0, 0.0, -9.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 2, -3, 0, 0, 0, 0, 0, 0],
        coefficients: [8.0, 614.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 2, -3, 0, 0, 0, 0, 0, 1],
        coefficients: [3.0, -3.0, -1.0, -2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 1],
        coefficients: [6.0, 17.0, 9.0, -3.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 1, 0, 0, -1, 0, 3, 0, 0, 0, 0],
        coefficients: [-3.0, -9.0, -5.0, 2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 1],
        coefficients: [0.0, 6.0, 3.0, -1.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 2],
        coefficients: [-127.0, 21.0, 9.0, 55.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 4, -8, 0, 0, 0, 0, 0],
        coefficients: [3.0, 5.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -4, 8, 0, 0, 0, 0, 2],
        coefficients: [-6.0, -10.0, -4.0, 3.0],
    },
    PlanetaryTerm {
        argument: [0, -2, 2, 0, 0, 0, 2, 0, -2, 0, 0, 0, 0],
        coefficients: [5.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -4, 7, 0, 0, 0, 0, 2],
        coefficients: [16.0, 9.0, 4.0, -7.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -4, 7, 0, 0, 0, 0, 1],
        coefficients: [3.0, 0.0, 0.0, -2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 4, -7, 0, 0, 0, 0, 0],
        coefficients: [0.0, 22.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 1, 0, -2, 3, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, 19.0, 10.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 2, -2, 1, 0, 0, -2, 0, 3, 0, 0, 0, 0],
        coefficients: [7.0, 0.0, 0.0, -4.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -5, 10, 0, 0, 0, 0, 2],
        coefficients: [0.0, -5.0, -2.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 1, 0, -1, 2, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, 3.0, 1.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 2],
        coefficients: [-9.0, 3.0, 1.0, 4.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -3, 5, 0, 0, 0, 0, 2],
        coefficients: [17.0, 0.0, 0.0, -7.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -3, 5, 0, 0, 0, 0, 1],
        coefficients: [0.0, -3.0, -2.0, -1.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 3, -5, 0, 0, 0, 0, 0],
        coefficients: [-20.0, 34.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 1, -2, 0, 0, 0, 0, 0, 1],
        coefficients: [-10.0, 0.0, 1.0, 5.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 1, 0, 1, -3, 0, 0, 0, 0, 0, 0],
        coefficients: [-4.0, 0.0, 0.0, 2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 1, -2, 0, 0, 0, 0, 0, 0],
        coefficients: [22.0, -87.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -1, 2, 0, 0, 0, 0, 0, 1],
        coefficients: [-4.0, 0.0, 0.0, 2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -1, 2, 0, 0, 0, 0, 0, 2],
        coefficients: [-3.0, -6.0, -2.0, 1.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -7, 11, 0, 0, 0, 0, 0, 2],
        coefficients: [-16.0, -3.0, -1.0, 7.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -7, 11, 0, 0, 0, 0, 0, 1],
        coefficients: [0.0, -3.0, -2.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, -2, 2, 0, 0, 4, -4, 0, 0, 0, 0, 0, 0],
        coefficients: [4.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 2, -3, 0, 0, 0, 0, 0],
        coefficients: [-68.0, 39.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 2, -2, 1, 0, -4, 4, 0, 0, 0, 0, 0, 0],
        coefficients: [27.0, 0.0, 0.0, -14.0],
    },
    PlanetaryTerm {
        argument: [0, -1, 1, 0, 0, 4, -5, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, -4.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 1, -1, 0, 0, 0, 0, 0],
        coefficients: [-25.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -4, 7, 0, 0, 0, 0, 0, 1],
        coefficients: [-12.0, -3.0, -2.0, 6.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 1, 0, -4, 6, 0, 0, 0, 0, 0, 0],
        coefficients: [3.0, 0.0, 0.0, -1.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -4, 7, 0, 0, 0, 0, 0, 2],
        coefficients: [3.0, 66.0, 29.0, -1.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -4, 6, 0, 0, 0, 0, 0, 2],
        coefficients: [490.0, 0.0, 0.0, -213.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -4, 6, 0, 0, 0, 0, 0, 1],
        coefficients: [-22.0, 93.0, 49.0, 12.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 1, 0, -4, 5, 0, 0, 0, 0, 0, 0],
        coefficients: [-7.0, 28.0, 15.0, 4.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -4, 6, 0, 0, 0, 0, 0, 1],
        coefficients: [-3.0, 13.0, 7.0, 2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 4, -6, 0, 0, 0, 0, 0, 0],
        coefficients: [-46.0, 14.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [-2, 0, 2, 0, 0, 2, -2, 0, 0, 0, 0, 0, 0],
        coefficients: [-5.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0],
        coefficients: [2.0, 1.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, -1, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, -3.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 1, 0, 1, -1, 0, 0, 0, 0, 0, 0],
        coefficients: [-28.0, 0.0, 0.0, 15.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -1, 0, 5, 0, 0, 0, 2],
        coefficients: [5.0, 0.0, 0.0, -2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 1, -3, 0, 0, 0, 0, 0],
        coefficients: [0.0, 3.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -1, 3, 0, 0, 0, 0, 2],
        coefficients: [-11.0, 0.0, 0.0, 5.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -7, 12, 0, 0, 0, 0, 2],
        coefficients: [0.0, 3.0, 1.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -1, 1, 0, 0, 0, 0, 0, 2],
        coefficients: [-3.0, 0.0, 0.0, 1.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -1, 1, 0, 0, 0, 0, 0, 1],
        coefficients: [25.0, 106.0, 57.0, -13.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 1, 0, -1, 0, 0, 0, 0, 0, 0, 0],
        coefficients: [5.0, 21.0, 11.0, -3.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 1, -1, 0, 0, 0, 0, 0, 0],
        coefficients: [1485.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 1, -1, 0, 0, 0, 0, 0, 1],
        coefficients: [-7.0, -32.0, -17.0, 4.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 1, 0, 1, -2, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, 5.0, 3.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -2, 5, 0, 0, 0, 0, 2],
        coefficients: [-6.0, -3.0, -2.0, 3.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -1, 0, 4, 0, 0, 0, 2],
        coefficients: [30.0, -6.0, -2.0, -13.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 1, 0, -4, 0, 0, 0, 0],
        coefficients: [-4.0, 4.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 1, 0, -1, 1, 0, 0, 0, 0, 0, 0],
        coefficients: [-19.0, 0.0, 0.0, 10.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -6, 10, 0, 0, 0, 0, 2],
        coefficients: [0.0, 4.0, 2.0, -1.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -6, 10, 0, 0, 0, 0, 0],
        coefficients: [0.0, 3.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 2, -2, 1, 0, 0, -3, 0, 3, 0, 0, 0, 0],
        coefficients: [4.0, 0.0, 0.0, -2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -3, 7, 0, 0, 0, 0, 2],
        coefficients: [0.0, -3.0, -1.0, 0.0],
    },
    PlanetaryTerm {
        argument: [-2, 0, 2, 0, 0, 4, -4, 0, 0, 0, 0, 0, 0],
        coefficients: [-3.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -5, 8, 0, 0, 0, 0, 2],
        coefficients: [5.0, 3.0, 1.0, -2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 5, -8, 0, 0, 0, 0, 0],
        coefficients: [0.0, 11.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -1, 0, 3, 0, 0, 0, 2],
        coefficients: [118.0, 0.0, 0.0, -52.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -1, 0, 3, 0, 0, 0, 1],
        coefficients: [0.0, -5.0, -3.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 1, 0, -3, 0, 0, 0, 0],
        coefficients: [-28.0, 36.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 2, -4, 0, 0, 0, 0, 0, 0],
        coefficients: [5.0, -5.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -2, 4, 0, 0, 0, 0, 0, 1],
        coefficients: [14.0, -59.0, -31.0, -8.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 1, 0, -2, 3, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, 9.0, 5.0, 1.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -2, 4, 0, 0, 0, 0, 0, 2],
        coefficients: [-458.0, 0.0, 0.0, 198.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -6, 9, 0, 0, 0, 0, 0, 2],
        coefficients: [0.0, -45.0, -20.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -6, 9, 0, 0, 0, 0, 0, 1],
        coefficients: [9.0, 0.0, 0.0, -5.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 6, -9, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, -3.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 1, 0, 0, 1, 0, -2, 0, 0, 0, 0],
        coefficients: [0.0, -4.0, -2.0, -1.0],
    },
    PlanetaryTerm {
        argument: [0, 2, -2, 1, 0, -2, 2, 0, 0, 0, 0, 0, 0],
        coefficients: [11.0, 0.0, 0.0, -6.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -4, 6, 0, 0, 0, 0, 2],
        coefficients: [6.0, 0.0, 0.0, -2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 4, -6, 0, 0, 0, 0, 0],
        coefficients: [-16.0, 23.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 1, 0, 3, -4, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, -4.0, -2.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -1, 0, 2, 0, 0, 0, 2],
        coefficients: [-5.0, 0.0, 0.0, 2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 1, 0, -2, 0, 0, 0, 0],
        coefficients: [-166.0, 269.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 1, 0, 0, 1, 0, -1, 0, 0, 0, 0],
        coefficients: [15.0, 0.0, 0.0, -8.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -5, 9, 0, 0, 0, 0, 0, 2],
        coefficients: [10.0, 0.0, 0.0, -4.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 3, -4, 0, 0, 0, 0, 0],
        coefficients: [-78.0, 45.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -3, 4, 0, 0, 0, 0, 0, 2],
        coefficients: [0.0, -5.0, -2.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -3, 4, 0, 0, 0, 0, 0, 1],
        coefficients: [7.0, 0.0, 0.0, -4.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 3, -4, 0, 0, 0, 0, 0, 0],
        coefficients: [-5.0, 328.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 3, -4, 0, 0, 0, 0, 0, 1],
        coefficients: [3.0, 0.0, 0.0, -2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 1, 0, 0, 2, -2, 0, 0, 0, 0, 0],
        coefficients: [5.0, 0.0, 0.0, -2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 1, 0, 0, -1, 0, 2, 0, 0, 0, 0],
        coefficients: [0.0, 3.0, 1.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 1, 0, 0, -3, 0, 0, 0],
        coefficients: [-3.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 1, 0, 1, -5, 0, 0, 0],
        coefficients: [-3.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -1, 0, 1, 0, 0, 0, 1],
        coefficients: [0.0, -4.0, -2.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 1, 0, -1, 0, 0, 0, 0],
        coefficients: [-1223.0, -26.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 1, 0, -1, 0, 0, 0, 1],
        coefficients: [0.0, 7.0, 3.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 1, 0, -3, 5, 0, 0, 0],
        coefficients: [3.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 1, 0, -3, 4, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, 3.0, 2.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 1, 0, 0, -2, 0, 0, 0],
        coefficients: [-6.0, 20.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 2, -2, 0, 0, 0, 0, 0],
        coefficients: [-368.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 1, 0, 0, -1, 0, 0, 0],
        coefficients: [-75.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 1, 0, 0, -1, 0, 1, 0, 0, 0, 0],
        coefficients: [11.0, 0.0, 0.0, -6.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 1, 0, 0, -2, 2, 0, 0, 0, 0, 0],
        coefficients: [3.0, 0.0, 0.0, -2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -8, 14, 0, 0, 0, 0, 0, 2],
        coefficients: [-3.0, 0.0, 0.0, 1.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 1, 0, 2, -5, 0, 0, 0],
        coefficients: [-13.0, -30.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 5, -8, 3, 0, 0, 0, 0],
        coefficients: [21.0, 3.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 5, -8, 3, 0, 0, 0, 2],
        coefficients: [-3.0, 0.0, 0.0, 1.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 1],
        coefficients: [-4.0, 0.0, 0.0, 2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0],
        coefficients: [8.0, -27.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 3, -8, 3, 0, 0, 0, 0],
        coefficients: [-19.0, -11.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -3, 8, -3, 0, 0, 0, 2],
        coefficients: [-4.0, 0.0, 0.0, 2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 1, 0, -2, 5, 0, 0, 2],
        coefficients: [0.0, 5.0, 2.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -8, 12, 0, 0, 0, 0, 0, 2],
        coefficients: [-6.0, 0.0, 0.0, 2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -8, 12, 0, 0, 0, 0, 0, 0],
        coefficients: [-8.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 1, 0, 1, -2, 0, 0, 0],
        coefficients: [-1.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 2],
        coefficients: [-14.0, 0.0, 0.0, 6.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0],
        coefficients: [6.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 2],
        coefficients: [-74.0, 0.0, 0.0, 32.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 1, 0, 0, 2, 0, 0, 2],
        coefficients: [0.0, -3.0, -1.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 2, -2, 1, 0, -5, 5, 0, 0, 0, 0, 0, 0],
        coefficients: [4.0, 0.0, 0.0, -2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0],
        coefficients: [8.0, 11.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 1],
        coefficients: [0.0, 3.0, 2.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 2],
        coefficients: [-262.0, 0.0, 0.0, 114.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 3, -6, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, -4.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -3, 6, 0, 0, 0, 0, 0, 1],
        coefficients: [-7.0, 0.0, 0.0, 4.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -3, 6, 0, 0, 0, 0, 0, 2],
        coefficients: [0.0, -27.0, -12.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -1, 4, 0, 0, 0, 0, 2],
        coefficients: [-19.0, -8.0, -4.0, 8.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -5, 7, 0, 0, 0, 0, 0, 2],
        coefficients: [202.0, 0.0, 0.0, -87.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -5, 7, 0, 0, 0, 0, 0, 1],
        coefficients: [-8.0, 35.0, 19.0, 5.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 1, 0, -5, 6, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, 4.0, 2.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 5, -7, 0, 0, 0, 0, 0, 0],
        coefficients: [16.0, -5.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 2, -2, 1, 0, 0, -1, 0, 1, 0, 0, 0, 0],
        coefficients: [5.0, 0.0, 0.0, -3.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -1, 0, 1, 0, 0, 0, 0],
        coefficients: [0.0, -3.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, -1, 0, 3, 0, 0, 0, 0, 0, 2],
        coefficients: [1.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 1, 0, 2, 0, 0, 0, 2],
        coefficients: [-35.0, -48.0, -21.0, 15.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -2, 6, 0, 0, 0, 0, 2],
        coefficients: [-3.0, -5.0, -2.0, 1.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 1, 0, 2, -2, 0, 0, 0, 0, 0, 0],
        coefficients: [6.0, 0.0, 0.0, -3.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -6, 9, 0, 0, 0, 0, 2],
        coefficients: [3.0, 0.0, 0.0, -1.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 6, -9, 0, 0, 0, 0, 0],
        coefficients: [0.0, -5.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -2, 2, 0, 0, 0, 0, 0, 1],
        coefficients: [12.0, 55.0, 29.0, -6.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 1, 0, -2, 1, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, 5.0, 3.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 2, -2, 0, 0, 0, 0, 0, 0],
        coefficients: [-598.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 2, -2, 0, 0, 0, 0, 0, 1],
        coefficients: [-3.0, -13.0, -7.0, 1.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 1, 0, 3, 0, 0, 0, 2],
        coefficients: [-5.0, -7.0, -3.0, 2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -5, 7, 0, 0, 0, 0, 2],
        coefficients: [3.0, 0.0, 0.0, -1.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 5, -7, 0, 0, 0, 0, 0],
        coefficients: [5.0, -7.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 1, 0, -2, 2, 0, 0, 0, 0, 0, 0],
        coefficients: [4.0, 0.0, 0.0, -2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 4, -5, 0, 0, 0, 0, 0],
        coefficients: [16.0, -6.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 1, -3, 0, 0, 0, 0, 0, 0],
        coefficients: [8.0, -3.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -1, 3, 0, 0, 0, 0, 0, 1],
        coefficients: [8.0, -31.0, -16.0, -4.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 1, 0, -1, 2, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, 3.0, 1.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -1, 3, 0, 0, 0, 0, 0, 2],
        coefficients: [113.0, 0.0, 0.0, -49.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -7, 10, 0, 0, 0, 0, 0, 2],
        coefficients: [0.0, -24.0, -10.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -7, 10, 0, 0, 0, 0, 0, 1],
        coefficients: [4.0, 0.0, 0.0, -2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 3, -3, 0, 0, 0, 0, 0],
        coefficients: [27.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -4, 8, 0, 0, 0, 0, 0, 2],
        coefficients: [-3.0, 0.0, 0.0, 1.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -4, 5, 0, 0, 0, 0, 0, 2],
        coefficients: [0.0, -4.0, -2.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -4, 5, 0, 0, 0, 0, 0, 1],
        coefficients: [5.0, 0.0, 0.0, -2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 4, -5, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, -3.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 2],
        coefficients: [-13.0, 0.0, 0.0, 6.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -2, 0, 5, 0, 0, 0, 2],
        coefficients: [5.0, 0.0, 0.0, -2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 2],
        coefficients: [-18.0, -10.0, -4.0, 8.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0],
        coefficients: [-4.0, -28.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 2],
        coefficients: [-5.0, 6.0, 3.0, 2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -9, 13, 0, 0, 0, 0, 0, 2],
        coefficients: [-3.0, 0.0, 0.0, 1.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -1, 5, 0, 0, 0, 0, 2],
        coefficients: [-5.0, -9.0, -4.0, 2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -2, 0, 4, 0, 0, 0, 2],
        coefficients: [17.0, 0.0, 0.0, -7.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 2, 0, -4, 0, 0, 0, 0],
        coefficients: [11.0, 4.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -2, 7, 0, 0, 0, 0, 2],
        coefficients: [0.0, -6.0, -2.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 2, 0, -3, 0, 0, 0, 0],
        coefficients: [83.0, 15.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -2, 5, 0, 0, 0, 0, 0, 1],
        coefficients: [-4.0, 0.0, 0.0, 2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -2, 5, 0, 0, 0, 0, 0, 2],
        coefficients: [0.0, -114.0, -49.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -6, 8, 0, 0, 0, 0, 0, 2],
        coefficients: [117.0, 0.0, 0.0, -51.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -6, 8, 0, 0, 0, 0, 0, 1],
        coefficients: [-5.0, 19.0, 10.0, 2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 6, -8, 0, 0, 0, 0, 0, 0],
        coefficients: [-3.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 1, 0, 0, 2, 0, -2, 0, 0, 0, 0],
        coefficients: [-3.0, 0.0, 0.0, 2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -3, 9, 0, 0, 0, 0, 2],
        coefficients: [0.0, -3.0, -1.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 5, -6, 0, 0, 0, 0, 0],
        coefficients: [3.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 5, -6, 0, 0, 0, 0, 2],
        coefficients: [0.0, -6.0, -2.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 2, 0, -2, 0, 0, 0, 0],
        coefficients: [393.0, 3.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 2, 0, -2, 0, 0, 0, 1],
        coefficients: [-4.0, 21.0, 11.0, 2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 2, 0, -2, 0, 0, 0, 2],
        coefficients: [-6.0, 0.0, -1.0, 3.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -5, 10, 0, 0, 0, 0, 0, 2],
        coefficients: [-3.0, 8.0, 4.0, 1.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 4, -4, 0, 0, 0, 0, 0],
        coefficients: [8.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 4, -4, 0, 0, 0, 0, 2],
        coefficients: [18.0, -29.0, -13.0, -8.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -3, 3, 0, 0, 0, 0, 0, 1],
        coefficients: [8.0, 34.0, 18.0, -4.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 3, -3, 0, 0, 0, 0, 0, 0],
        coefficients: [89.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 3, -3, 0, 0, 0, 0, 0, 1],
        coefficients: [3.0, 12.0, 6.0, -1.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 3, -3, 0, 0, 0, 0, 0, 2],
        coefficients: [54.0, -15.0, -7.0, -24.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 2, 0, 0, -3, 0, 0, 0],
        coefficients: [0.0, 3.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -5, 13, 0, 0, 0, 0, 2],
        coefficients: [3.0, 0.0, 0.0, -1.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 2, 0, -1, 0, 0, 0, 0],
        coefficients: [0.0, 35.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 2, 0, -1, 0, 0, 0, 2],
        coefficients: [-154.0, -30.0, -13.0, 67.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 2, 0, 0, -2, 0, 0, 0],
        coefficients: [15.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 2, 0, 0, -2, 0, 0, 1],
        coefficients: [0.0, 4.0, 2.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 3, -2, 0, 0, 0, 0, 0],
        coefficients: [0.0, 9.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 3, -2, 0, 0, 0, 0, 2],
        coefficients: [80.0, -71.0, -31.0, -35.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 2, 0, 0, -1, 0, 0, 2],
        coefficients: [0.0, -20.0, -9.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -6, 15, 0, 0, 0, 0, 2],
        coefficients: [11.0, 5.0, 2.0, -5.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -8, 15, 0, 0, 0, 0, 0, 2],
        coefficients: [61.0, -96.0, -42.0, -27.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -3, 9, -4, 0, 0, 0, 0, 2],
        coefficients: [14.0, 9.0, 4.0, -6.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 2, 0, 2, -5, 0, 0, 2],
        coefficients: [-11.0, -6.0, -3.0, 5.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -2, 8, -1, -5, 0, 0, 2],
        coefficients: [0.0, -3.0, -1.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 6, -8, 3, 0, 0, 0, 2],
        coefficients: [123.0, -415.0, -180.0, -53.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, 0.0, 0.0, -35.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0],
        coefficients: [-5.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 1],
        coefficients: [7.0, -32.0, -17.0, -4.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, -9.0, -5.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 1],
        coefficients: [0.0, -4.0, 2.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 2],
        coefficients: [-89.0, 0.0, 0.0, 38.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -6, 16, -4, -5, 0, 0, 2],
        coefficients: [0.0, -86.0, -19.0, -6.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -2, 8, -3, 0, 0, 0, 2],
        coefficients: [0.0, 0.0, -19.0, 6.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -2, 8, -3, 0, 0, 0, 2],
        coefficients: [-123.0, -416.0, -180.0, 53.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 6, -8, 1, 5, 0, 0, 2],
        coefficients: [0.0, -3.0, -1.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 2, 0, -2, 5, 0, 0, 2],
        coefficients: [12.0, -6.0, -3.0, -5.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 3, -5, 4, 0, 0, 0, 0, 2],
        coefficients: [-13.0, 9.0, 4.0, 6.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -8, 11, 0, 0, 0, 0, 0, 2],
        coefficients: [0.0, -15.0, -7.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -8, 11, 0, 0, 0, 0, 0, 1],
        coefficients: [3.0, 0.0, 0.0, -1.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -8, 11, 0, 0, 0, 0, 0, 2],
        coefficients: [-62.0, -97.0, -42.0, 27.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 11, 0, 0, 0, 0, 0, 2],
        coefficients: [-11.0, 5.0, 2.0, 5.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 2, 0, 0, 1, 0, 0, 2],
        coefficients: [0.0, -19.0, -8.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 3, -3, 0, 2, 0, 0, 0, 2],
        coefficients: [-3.0, 0.0, 0.0, 1.0],
    },
    PlanetaryTerm {
        argument: [0, 2, -2, 1, 0, 0, 4, -8, 3, 0, 0, 0, 0],
        coefficients: [0.0, 4.0, 2.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 1, -1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, 3.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 2, -2, 1, 0, 0, -4, 8, -3, 0, 0, 0, 0],
        coefficients: [0.0, 4.0, 2.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 1, 2, 0, 0, 0, 0, 2],
        coefficients: [-85.0, -70.0, -31.0, 37.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 2, 0, 1, 0, 0, 0, 2],
        coefficients: [163.0, -12.0, -5.0, -72.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -3, 7, 0, 0, 0, 0, 0, 2],
        coefficients: [-63.0, -16.0, -7.0, 28.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 2],
        coefficients: [-21.0, -32.0, -14.0, 9.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -5, 6, 0, 0, 0, 0, 0, 2],
        coefficients: [0.0, -3.0, -1.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -5, 6, 0, 0, 0, 0, 0, 1],
        coefficients: [3.0, 0.0, 0.0, -2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 5, -6, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, 8.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 5, -6, 0, 0, 0, 0, 0, 2],
        coefficients: [3.0, 10.0, 4.0, -1.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 2, 0, 2, 0, 0, 0, 2],
        coefficients: [3.0, 0.0, 0.0, -1.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -1, 6, 0, 0, 0, 0, 2],
        coefficients: [0.0, -7.0, -3.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 7, -9, 0, 0, 0, 0, 2],
        coefficients: [0.0, -4.0, -2.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 2, -1, 0, 0, 0, 0, 0, 0],
        coefficients: [6.0, 19.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 2, -1, 0, 0, 0, 0, 0, 2],
        coefficients: [5.0, -173.0, -75.0, -2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 6, -7, 0, 0, 0, 0, 2],
        coefficients: [0.0, -7.0, -3.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 5, -5, 0, 0, 0, 0, 2],
        coefficients: [7.0, -12.0, -5.0, -3.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -1, 4, 0, 0, 0, 0, 0, 1],
        coefficients: [-3.0, 0.0, 0.0, 2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -1, 4, 0, 0, 0, 0, 0, 2],
        coefficients: [3.0, -4.0, -2.0, -1.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -7, 9, 0, 0, 0, 0, 0, 2],
        coefficients: [74.0, 0.0, 0.0, -32.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -7, 9, 0, 0, 0, 0, 0, 1],
        coefficients: [-3.0, 12.0, 6.0, 2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 4, -3, 0, 0, 0, 0, 2],
        coefficients: [26.0, -14.0, -6.0, -11.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 3, -1, 0, 0, 0, 0, 2],
        coefficients: [19.0, 0.0, 0.0, -8.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -4, 4, 0, 0, 0, 0, 0, 1],
        coefficients: [6.0, 24.0, 13.0, -3.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 4, -4, 0, 0, 0, 0, 0, 0],
        coefficients: [83.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 4, -4, 0, 0, 0, 0, 0, 1],
        coefficients: [0.0, -10.0, -5.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 4, -4, 0, 0, 0, 0, 0, 2],
        coefficients: [11.0, -3.0, -1.0, -5.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 2, 1, 0, 0, 0, 0, 2],
        coefficients: [3.0, 0.0, 1.0, -1.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -3, 0, 5, 0, 0, 0, 2],
        coefficients: [3.0, 0.0, 0.0, -1.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0],
        coefficients: [-4.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 1],
        coefficients: [5.0, -23.0, -12.0, -3.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 2],
        coefficients: [-339.0, 0.0, 0.0, 147.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -9, 12, 0, 0, 0, 0, 0, 2],
        coefficients: [0.0, -10.0, -5.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 3, 0, -4, 0, 0, 0, 0],
        coefficients: [5.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 2, -2, 1, 0, 1, -1, 0, 0, 0, 0, 0, 0],
        coefficients: [3.0, 0.0, 0.0, -1.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 7, -8, 0, 0, 0, 0, 2],
        coefficients: [0.0, -4.0, -2.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 3, 0, -3, 0, 0, 0, 0],
        coefficients: [18.0, -3.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 3, 0, -3, 0, 0, 0, 2],
        coefficients: [9.0, -11.0, -5.0, -4.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -2, 6, 0, 0, 0, 0, 0, 2],
        coefficients: [-8.0, 0.0, 0.0, 4.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -6, 7, 0, 0, 0, 0, 0, 1],
        coefficients: [3.0, 0.0, 0.0, -1.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 6, -7, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, 9.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 6, -6, 0, 0, 0, 0, 2],
        coefficients: [6.0, -9.0, -4.0, -2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 3, 0, -2, 0, 0, 0, 0],
        coefficients: [-4.0, -12.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 3, 0, -2, 0, 0, 0, 2],
        coefficients: [67.0, -91.0, -39.0, -29.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 5, -4, 0, 0, 0, 0, 2],
        coefficients: [30.0, -18.0, -8.0, -13.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 3, -2, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 3, -2, 0, 0, 0, 0, 0, 2],
        coefficients: [0.0, -114.0, -50.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 3, 0, -1, 0, 0, 0, 2],
        coefficients: [0.0, 0.0, 0.0, 23.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 3, 0, -1, 0, 0, 0, 2],
        coefficients: [517.0, 16.0, 7.0, -224.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 3, 0, 0, -2, 0, 0, 2],
        coefficients: [0.0, -7.0, -3.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 4, -2, 0, 0, 0, 0, 2],
        coefficients: [143.0, -3.0, -1.0, -62.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 3, 0, 0, -1, 0, 0, 2],
        coefficients: [29.0, 0.0, 0.0, -13.0],
    },
    PlanetaryTerm {
        argument: [0, 2, -2, 1, 0, 0, 1, 0, -1, 0, 0, 0, 0],
        coefficients: [-4.0, 0.0, 0.0, 2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -8, 16, 0, 0, 0, 0, 0, 2],
        coefficients: [-6.0, 0.0, 0.0, 3.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 3, 0, 2, -5, 0, 0, 2],
        coefficients: [5.0, 12.0, 5.0, -2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 7, -8, 3, 0, 0, 0, 2],
        coefficients: [-25.0, 0.0, 0.0, 11.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -5, 16, -4, -5, 0, 0, 2],
        coefficients: [-3.0, 0.0, 0.0, 1.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 2],
        coefficients: [0.0, 4.0, 2.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, -1, 8, -3, 0, 0, 0, 2],
        coefficients: [-22.0, 12.0, 5.0, 10.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -8, 10, 0, 0, 0, 0, 0, 2],
        coefficients: [50.0, 0.0, 0.0, -22.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -8, 10, 0, 0, 0, 0, 0, 1],
        coefficients: [0.0, 7.0, 4.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -8, 10, 0, 0, 0, 0, 0, 2],
        coefficients: [0.0, 3.0, 1.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 2, 2, 0, 0, 0, 0, 2],
        coefficients: [-4.0, 4.0, 2.0, 2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 3, 0, 1, 0, 0, 0, 2],
        coefficients: [-5.0, -11.0, -5.0, 2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -3, 8, 0, 0, 0, 0, 0, 2],
        coefficients: [0.0, 4.0, 2.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -5, 5, 0, 0, 0, 0, 0, 1],
        coefficients: [4.0, 17.0, 9.0, -2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 5, -5, 0, 0, 0, 0, 0, 0],
        coefficients: [59.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 5, -5, 0, 0, 0, 0, 0, 1],
        coefficients: [0.0, -4.0, -2.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 5, -5, 0, 0, 0, 0, 0, 2],
        coefficients: [-8.0, 0.0, 0.0, 4.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0],
        coefficients: [-3.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 1],
        coefficients: [4.0, -15.0, -8.0, -2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 2],
        coefficients: [370.0, -8.0, 0.0, -160.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 7, -7, 0, 0, 0, 0, 2],
        coefficients: [0.0, 0.0, -3.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 7, -7, 0, 0, 0, 0, 2],
        coefficients: [0.0, 3.0, 1.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 6, -5, 0, 0, 0, 0, 2],
        coefficients: [-6.0, 3.0, 1.0, 3.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 7, -8, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, 6.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 5, -3, 0, 0, 0, 0, 2],
        coefficients: [-10.0, 0.0, 0.0, 4.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 4, -3, 0, 0, 0, 0, 0, 2],
        coefficients: [0.0, 9.0, 4.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 1, 2, 0, 0, 0, 0, 0, 2],
        coefficients: [4.0, 17.0, 7.0, -2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -9, 11, 0, 0, 0, 0, 0, 2],
        coefficients: [34.0, 0.0, 0.0, -15.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -9, 11, 0, 0, 0, 0, 0, 1],
        coefficients: [0.0, 5.0, 3.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 4, 0, -4, 0, 0, 0, 2],
        coefficients: [-5.0, 0.0, 0.0, 2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 4, 0, -3, 0, 0, 0, 2],
        coefficients: [-37.0, -7.0, -3.0, 16.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -6, 6, 0, 0, 0, 0, 0, 1],
        coefficients: [3.0, 13.0, 7.0, -2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 6, -6, 0, 0, 0, 0, 0, 0],
        coefficients: [40.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 6, -6, 0, 0, 0, 0, 0, 1],
        coefficients: [0.0, -3.0, -2.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 4, 0, -2, 0, 0, 0, 2],
        coefficients: [-184.0, -3.0, -1.0, 80.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 6, -4, 0, 0, 0, 0, 2],
        coefficients: [-3.0, 0.0, 0.0, 1.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 3, -1, 0, 0, 0, 0, 0, 0],
        coefficients: [-3.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 3, -1, 0, 0, 0, 0, 0, 1],
        coefficients: [0.0, -10.0, -6.0, -1.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 3, -1, 0, 0, 0, 0, 0, 2],
        coefficients: [31.0, -6.0, 0.0, -13.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 4, 0, -1, 0, 0, 0, 2],
        coefficients: [-3.0, -32.0, -14.0, 1.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 4, 0, 0, -2, 0, 0, 2],
        coefficients: [-7.0, 0.0, 0.0, 3.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 5, -2, 0, 0, 0, 0, 2],
        coefficients: [0.0, -8.0, -4.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0],
        coefficients: [3.0, -4.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 8, -9, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, 4.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 5, -4, 0, 0, 0, 0, 0, 2],
        coefficients: [0.0, 3.0, 1.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 2, 1, 0, 0, 0, 0, 0, 2],
        coefficients: [19.0, -23.0, -10.0, 2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 2, 1, 0, 0, 0, 0, 0, 1],
        coefficients: [0.0, 0.0, 0.0, -10.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 2, 1, 0, 0, 0, 0, 0, 1],
        coefficients: [0.0, 3.0, 2.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -7, 7, 0, 0, 0, 0, 0, 1],
        coefficients: [0.0, 9.0, 5.0, -1.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 7, -7, 0, 0, 0, 0, 0, 0],
        coefficients: [28.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 4, -2, 0, 0, 0, 0, 0, 1],
        coefficients: [0.0, -7.0, -4.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 4, -2, 0, 0, 0, 0, 0, 2],
        coefficients: [8.0, -4.0, 0.0, -4.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 4, -2, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, 0.0, -2.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 4, -2, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, 3.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 5, 0, -4, 0, 0, 0, 2],
        coefficients: [-3.0, 0.0, 0.0, 1.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 5, 0, -3, 0, 0, 0, 2],
        coefficients: [-9.0, 0.0, 1.0, 4.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 5, 0, -2, 0, 0, 0, 2],
        coefficients: [3.0, 12.0, 5.0, -1.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 2],
        coefficients: [17.0, -3.0, -1.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -8, 8, 0, 0, 0, 0, 0, 1],
        coefficients: [0.0, 7.0, 4.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 8, -8, 0, 0, 0, 0, 0, 0],
        coefficients: [19.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 5, -3, 0, 0, 0, 0, 0, 1],
        coefficients: [0.0, -5.0, -3.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 5, -3, 0, 0, 0, 0, 0, 2],
        coefficients: [14.0, -3.0, 0.0, -1.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -9, 9, 0, 0, 0, 0, 0, 1],
        coefficients: [0.0, 0.0, -1.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -9, 9, 0, 0, 0, 0, 0, 1],
        coefficients: [0.0, 0.0, 0.0, -5.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, -9, 9, 0, 0, 0, 0, 0, 1],
        coefficients: [0.0, 5.0, 3.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 9, -9, 0, 0, 0, 0, 0, 0],
        coefficients: [13.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 6, -4, 0, 0, 0, 0, 0, 1],
        coefficients: [0.0, -3.0, -2.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 6, 0, 0, 0, 0, 0, 2],
        coefficients: [2.0, 9.0, 4.0, 3.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, 0.0, 0.0, -4.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0],
        coefficients: [8.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 6, 0, 0, 0, 0, 0, 1],
        coefficients: [0.0, 4.0, 2.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 6, 0, 0, 0, 0, 0, 2],
        coefficients: [6.0, 0.0, 0.0, -3.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0],
        coefficients: [6.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 6, 0, 0, 0, 0, 0, 1],
        coefficients: [0.0, 3.0, 1.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 6, 0, 0, 0, 0, 0, 2],
        coefficients: [5.0, 0.0, 0.0, -2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2],
        coefficients: [3.0, 0.0, 0.0, -1.0],
    },
    PlanetaryTerm {
        argument: [1, 0, -2, 0, 0, 0, 2, 0, -2, 0, 0, 0, 0],
        coefficients: [-3.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [1, 0, -2, 0, 0, 2, -2, 0, 0, 0, 0, 0, 0],
        coefficients: [6.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [1, 0, -2, 0, 0, 0, 1, 0, -1, 0, 0, 0, 0],
        coefficients: [7.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [1, 0, -2, 0, 0, 1, -1, 0, 0, 0, 0, 0, 0],
        coefficients: [-4.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [-1, 0, 0, 0, 0, 3, -3, 0, 0, 0, 0, 0, 0],
        coefficients: [4.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [-1, 0, 0, 0, 0, 0, 2, 0, -2, 0, 0, 0, 0],
        coefficients: [6.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [-1, 0, 2, 0, 0, 0, 4, -8, 3, 0, 0, 0, 0],
        coefficients: [0.0, -4.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [1, 0, -2, 0, 0, 0, 4, -8, 3, 0, 0, 0, 0],
        coefficients: [0.0, -4.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [-2, 0, 2, 0, 0, 0, 4, -8, 3, 0, 0, 0, 0],
        coefficients: [5.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [-1, 0, 0, 0, 0, 0, 2, 0, -3, 0, 0, 0, 0],
        coefficients: [-3.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [-1, 0, 0, 0, 0, 0, 1, 0, -1, 0, 0, 0, 0],
        coefficients: [4.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [-1, 0, 0, 0, 0, 1, -1, 0, 0, 0, 0, 0, 0],
        coefficients: [-5.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [-1, 0, 2, 0, 0, 2, -2, 0, 0, 0, 0, 0, 0],
        coefficients: [4.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [1, -1, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, 3.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [-1, 0, 2, 0, 0, 0, 2, 0, -3, 0, 0, 0, 0],
        coefficients: [13.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [-2, 0, 0, 0, 0, 0, 2, 0, -3, 0, 0, 0, 0],
        coefficients: [21.0, 11.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [1, 0, 0, 0, 0, 0, 4, -8, 3, 0, 0, 0, 0],
        coefficients: [0.0, -5.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [-1, 1, -1, 1, 0, 0, -1, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, -5.0, -2.0, 0.0],
    },
    PlanetaryTerm {
        argument: [1, 1, -1, 1, 0, 0, -1, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, 5.0, 3.0, 0.0],
    },
    PlanetaryTerm {
        argument: [-1, 0, 0, 0, 0, 0, 4, -8, 3, 0, 0, 0, 0],
        coefficients: [0.0, -5.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [-1, 0, 2, 1, 0, 0, 2, 0, -2, 0, 0, 0, 0],
        coefficients: [-3.0, 0.0, 0.0, 2.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 0, 0, 0, 0, 2, 0, -2, 0, 0, 0, 0],
        coefficients: [20.0, 10.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [-1, 0, 2, 0, 0, 0, 2, 0, -2, 0, 0, 0, 0],
        coefficients: [-34.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [-1, 0, 2, 0, 0, 3, -3, 0, 0, 0, 0, 0, 0],
        coefficients: [-19.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [1, 0, -2, 1, 0, 0, -2, 0, 2, 0, 0, 0, 0],
        coefficients: [3.0, 0.0, 0.0, -2.0],
    },
    PlanetaryTerm {
        argument: [1, 2, -2, 2, 0, -3, 3, 0, 0, 0, 0, 0, 0],
        coefficients: [-3.0, 0.0, 0.0, 1.0],
    },
    PlanetaryTerm {
        argument: [1, 2, -2, 2, 0, 0, -2, 0, 2, 0, 0, 0, 0],
        coefficients: [-6.0, 0.0, 0.0, 3.0],
    },
    PlanetaryTerm {
        argument: [1, 0, 0, 0, 0, 1, -1, 0, 0, 0, 0, 0, 0],
        coefficients: [-4.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [1, 0, 0, 0, 0, 0, 1, 0, -1, 0, 0, 0, 0],
        coefficients: [3.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, -2, 0, 0, 2, -2, 0, 0, 0, 0, 0, 0],
        coefficients: [3.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, -2, 0, 0, 0, 1, 0, -1, 0, 0, 0, 0],
        coefficients: [4.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 2, 0, 2, 0, -2, 2, 0, 0, 0, 0, 0, 0],
        coefficients: [3.0, 0.0, 0.0, -1.0],
    },
    PlanetaryTerm {
        argument: [0, 2, 0, 2, 0, 0, -1, 0, 1, 0, 0, 0, 0],
        coefficients: [6.0, 0.0, 0.0, -3.0],
    },
    PlanetaryTerm {
        argument: [0, 2, 0, 2, 0, -1, 1, 0, 0, 0, 0, 0, 0],
        coefficients: [-8.0, 0.0, 0.0, 3.0],
    },
    PlanetaryTerm {
        argument: [0, 2, 0, 2, 0, -2, 3, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, 3.0, 1.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 0, 2, 0, 0, 0, 2, 0, -2, 0, 0, 0, 0],
        coefficients: [-3.0, 0.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 1, 1, 2, 0, 0, 1, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, -3.0, -2.0, 0.0],
    },
    PlanetaryTerm {
        argument: [1, 2, 0, 2, 0, 0, 1, 0, 0, 0, 0, 0, 0],
        coefficients: [126.0, -63.0, -27.0, -55.0],
    },
    PlanetaryTerm {
        argument: [-1, 2, 0, 2, 0, 10, -3, 0, 0, 0, 0, 0, 0],
        coefficients: [-5.0, 0.0, 1.0, 2.0],
    },
    PlanetaryTerm {
        argument: [0, 1, 1, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0],
        coefficients: [-3.0, 28.0, 15.0, 2.0],
    },
    PlanetaryTerm {
        argument: [1, 2, 0, 2, 0, 0, 1, 0, 0, 0, 0, 0, 0],
        coefficients: [5.0, 0.0, 1.0, -2.0],
    },
    PlanetaryTerm {
        argument: [0, 2, 0, 2, 0, 0, 4, -8, 3, 0, 0, 0, 0],
        coefficients: [0.0, 9.0, 4.0, 1.0],
    },
    PlanetaryTerm {
        argument: [0, 2, 0, 2, 0, 0, -4, 8, -3, 0, 0, 0, 0],
        coefficients: [0.0, 9.0, 4.0, -1.0],
    },
    PlanetaryTerm {
        argument: [-1, 2, 0, 2, 0, 0, -4, 8, -3, 0, 0, 0, 0],
        coefficients: [-126.0, -63.0, -27.0, 55.0],
    },
    PlanetaryTerm {
        argument: [2, 2, -2, 2, 0, 0, -2, 0, 3, 0, 0, 0, 0],
        coefficients: [3.0, 0.0, 0.0, -1.0],
    },
    PlanetaryTerm {
        argument: [1, 2, 0, 1, 0, 0, -2, 0, 3, 0, 0, 0, 0],
        coefficients: [21.0, -11.0, -6.0, -11.0],
    },
    PlanetaryTerm {
        argument: [0, 1, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, -4.0, 0.0, 0.0],
    },
    PlanetaryTerm {
        argument: [-1, 2, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0],
        coefficients: [-21.0, -11.0, -6.0, 11.0],
    },
    PlanetaryTerm {
        argument: [-2, 2, 2, 2, 0, 0, 2, 0, -2, 0, 0, 0, 0],
        coefficients: [-3.0, 0.0, 0.0, 1.0],
    },
    PlanetaryTerm {
        argument: [0, 2, 0, 2, 0, 2, -3, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, 3.0, 1.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 2, 0, 2, 0, 1, -1, 0, 0, 0, 0, 0, 0],
        coefficients: [8.0, 0.0, 0.0, -4.0],
    },
    PlanetaryTerm {
        argument: [0, 2, 0, 2, 0, 0, 1, 0, -1, 0, 0, 0, 0],
        coefficients: [-6.0, 0.0, 0.0, 3.0],
    },
    PlanetaryTerm {
        argument: [0, 2, 0, 2, 0, 2, -2, 0, 0, 0, 0, 0, 0],
        coefficients: [-3.0, 0.0, 0.0, 1.0],
    },
    PlanetaryTerm {
        argument: [-1, 2, 2, 2, 0, 0, -1, 0, 1, 0, 0, 0, 0],
        coefficients: [3.0, 0.0, 0.0, -1.0],
    },
    PlanetaryTerm {
        argument: [1, 2, 0, 2, 0, -1, 1, 0, 0, 0, 0, 0, 0],
        coefficients: [-3.0, 0.0, 0.0, 1.0],
    },
    PlanetaryTerm {
        argument: [-1, 2, 2, 2, 0, 0, 2, 0, -3, 0, 0, 0, 0],
        coefficients: [-5.0, 0.0, 0.0, 2.0],
    },
    PlanetaryTerm {
        argument: [2, 2, 0, 2, 0, 0, 2, 0, -3, 0, 0, 0, 0],
        coefficients: [24.0, -12.0, -5.0, -11.0],
    },
    PlanetaryTerm {
        argument: [1, 2, 0, 2, 0, 0, -4, 8, -3, 0, 0, 0, 0],
        coefficients: [0.0, 3.0, 1.0, 0.0],
    },
    PlanetaryTerm {
        argument: [1, 2, 0, 2, 0, 0, 4, -8, 3, 0, 0, 0, 0],
        coefficients: [0.0, 3.0, 1.0, 0.0],
    },
    PlanetaryTerm {
        argument: [1, 1, 1, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0],
        coefficients: [0.0, 3.0, 2.0, 0.0],
    },
    PlanetaryTerm {
        argument: [0, 2, 0, 2, 0, 0, 1, 0, 0, 0, 0, 0, 0],
        coefficients: [-24.0, -12.0, -5.0, 10.0],
    },
    PlanetaryTerm {
        argument: [2, 2, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0],
        coefficients: [4.0, 0.0, -1.0, -2.0],
    },
    PlanetaryTerm {
        argument: [-1, 2, 2, 2, 0, 0, 2, 0, -2, 0, 0, 0, 0],
        coefficients: [13.0, 0.0, 0.0, -6.0],
    },
    PlanetaryTerm {
        argument: [-1, 2, 2, 2, 0, 3, -3, 0, 0, 0, 0, 0, 0],
        coefficients: [7.0, 0.0, 0.0, -3.0],
    },
    PlanetaryTerm {
        argument: [1, 2, 0, 2, 0, 1, -1, 0, 0, 0, 0, 0, 0],
        coefficients: [3.0, 0.0, 0.0, -1.0],
    },
    PlanetaryTerm {
        argument: [0, 2, 2, 2, 0, 0, 2, 0, -2, 0, 0, 0, 0],
        coefficients: [3.0, 0.0, 0.0, -1.0],
    },
];
