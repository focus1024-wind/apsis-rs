//! Structured fields of a two-line element set.
//!
//! 两行根数集的结构化字段。

use crate::checksum::verify_line_checksum;
use crate::field::{ElementError, ElementErrorKind, Result};
use apsis_time::{Epoch, TimeScale};

/// The security classification of an element set.
///
/// 根数集的安全密级。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Classification {
    /// Unclassified.
    ///
    /// 未分类。
    Unclassified,
    /// Classified as secret by the originating authority.
    ///
    /// 由发布机构列为机密。
    Secret,
    /// Classified at some level the parser does not recognise.
    ///
    /// 列为解析器无法识别的密级。
    Other(char),
}

impl Classification {
    /// Returns the character used in the element set.
    ///
    /// 返回根数集中所用的字符。
    #[must_use]
    pub const fn as_char(self) -> char {
        match self {
            Self::Unclassified => 'U',
            Self::Secret => 'S',
            Self::Other(character) => character,
        }
    }
}

/// A parsed two-line element set.
///
/// The fields are the mean elements and identification data exactly as
/// written in the element set. Units are the ones the format uses, named in
/// each field's documentation. Converting them to a state vector is the
/// concern of `apsis-sgp4`.
///
/// 已解析的两线根数集。
///
/// 各字段即根数集中原样写下的平均根数与标识信息。单位沿用格式所用者，在各字段
/// 文档中注明。把它们转换为状态矢量是 `apsis-sgp4` 的职责。
#[derive(Debug, Clone, PartialEq)]
pub struct TwoLineElement {
    /// Satellite name from the optional first line, if one was present.
    ///
    /// 可选首行中的卫星名称，若存在。
    pub name: Option<String>,
    /// Satellite catalog number.
    ///
    /// 卫星编目号。
    pub satellite_number: u32,
    /// Classification of the element set.
    ///
    /// 根数集的密级。
    pub classification: Classification,
    /// International designator, such as `98067A`.
    ///
    /// 国际designator，如 `98067A`。
    pub international_designator: String,
    /// Epoch of the mean elements, on the UTC scale.
    ///
    /// The format stores the epoch as a two digit year and a fraction of a
    /// day. The two digit year is expanded with the convention that years
    /// `57` through `99` are in the twentieth century and years `00`
    /// through `56` are in the twenty-first, which places the boundary
    /// before the start of the satellite catalogue.
    ///
    /// 平均根数的历元，位于 UTC 标度。
    ///
    /// 格式以两位数年份加一日的小数部分存储历元。两位数年份按如下约定展开：
    /// `57` 至 `99` 属二十世纪，`00` 至 `56` 属二十一世纪，该分界位于卫星编目
    /// 开始之前。
    pub epoch: Epoch,
    /// First derivative of mean motion divided by two, in revolutions per
    /// day squared.
    ///
    /// The format stores half of the derivative, which is why the field is
    /// documented as `ndot / 2`.
    ///
    /// 平运动的一阶导数除以二，单位为每日每二次方圈。
    ///
    /// 格式存储该导数的一半，故本字段记作 `ndot / 2`。
    pub mean_motion_dot: f64,
    /// Second derivative of mean motion divided by six, in revolutions per
    /// day cubed.
    ///
    /// The format stores one sixth of the derivative, which is why the
    /// field is documented as `nddot / 6`.
    ///
    /// 平运动的二阶导数除以六，单位为每日每三次方圈。
    ///
    /// 格式存储该导数的六分之一，故本字段记作 `nddot / 6`。
    pub mean_motion_double_dot: f64,
    /// The `BSTAR` drag term, in units of inverse Earth radii.
    ///
    /// The term is a free parameter of the SGP4 drag model. Its units are
    /// the reciprocal of the reference distance used by the model, which is
    /// the Earth equatorial radius in kilometers.
    ///
    /// `BSTAR` 阻力项，单位为地球半径的倒数。
    ///
    /// 该项是 SGP4 阻力模型的自由参数。其单位是模型所用参考距离的倒数，该距离
    /// 即以千米表示的地球赤道半径。
    pub bstar: f64,
    /// The element set number, which increments when a satellite's elements
    /// are refreshed.
    ///
    /// 根数集编号，在卫星根数更新时递增。
    pub element_set_number: u32,
    /// Orbital inclination, in degrees.
    ///
    /// 轨道倾角，单位为度。
    pub inclination: f64,
    /// Right ascension of the ascending node, in degrees.
    ///
    /// 升交点赤经，单位为度。
    pub right_ascension: f64,
    /// Orbital eccentricity, dimensionless.
    ///
    /// 轨道偏心率，无量纲。
    pub eccentricity: f64,
    /// Argument of perigee, in degrees.
    ///
    /// 近地点幅角，单位为度。
    pub argument_of_perigee: f64,
    /// Mean anomaly, in degrees.
    ///
    /// 平近点角，单位为度。
    pub mean_anomaly: f64,
    /// Mean motion, in revolutions per day.
    ///
    /// 平运动，单位为每日圈数。
    pub mean_motion: f64,
    /// Revolution number at epoch.
    ///
    /// 历元处的转数。
    pub revolution_number: u32,
}

/// Returns the one based inclusive character range of a field, as a slice of
/// characters.
///
/// Ranges are inclusive of both ends, matching the column numbering in the
/// format definition.
///
/// 返回某字段的从一开始、含两端的字符区间，以字符切片表示。
///
/// 区间含两端，与格式定义中的列编号一致。
fn field(line: &str, first: usize, last: usize) -> Option<String> {
    if first < 1 || last < first {
        return None;
    }
    let characters: Vec<char> = line.chars().collect();
    if last > characters.len() {
        return None;
    }
    Some(characters[first - 1..last].iter().collect())
}

/// Reads a field and returns it trimmed of surrounding spaces.
///
/// 读取某字段并去除首尾空格。
fn trimmed_field(line: &str, first: usize, last: usize, line_number: usize) -> Result<String> {
    match field(line, first, last) {
        Some(text) => Ok(text.trim().to_string()),
        None => Err(ElementError::new(
            ElementErrorKind::TooShort,
            line_number,
            Some((first, last)),
        )),
    }
}

/// Parses an unsigned integer field.
///
/// 解析无符号整数字段。
fn unsigned_field(line: &str, first: usize, last: usize, line_number: usize) -> Result<u32> {
    let text = trimmed_field(line, first, last, line_number)?;
    text.parse::<u32>().map_err(|_| {
        ElementError::new(
            ElementErrorKind::InvalidField,
            line_number,
            Some((first, last)),
        )
        .with_detail(format!("expected an unsigned integer, found {text:?}"))
    })
}

/// Parses a floating point field written with an explicit decimal point.
///
/// 解析带显式小数点的浮点字段。
fn float_field(line: &str, first: usize, last: usize, line_number: usize) -> Result<f64> {
    let text = trimmed_field(line, first, last, line_number)?;
    text.parse::<f64>().map_err(|_| {
        ElementError::new(
            ElementErrorKind::InvalidField,
            line_number,
            Some((first, last)),
        )
        .with_detail(format!("expected a number, found {text:?}"))
    })
}

/// Parses an exponent field of the form `smmmmmse` where the first `s` is an
/// optional sign for the mantissa, `e` is a signed single digit exponent, and
/// the mantissa is interpreted with an assumed decimal point before its first
/// digit.
///
/// For example ` 12345-3` is `0.12345e-3` and `-11606-4` is `-0.11606e-4`.
/// An all blank field is zero, which is how the format writes an absent
/// term.
///
/// 解析形如 `smmmmmse` 的指数字段：首个 `s` 是尾数的可选符号，`e` 是带符号的
/// 一位指数，尾数按首位之前带隐含小数点解释。
///
/// 例如 ` 12345-3` 即 `0.12345e-3`，`-11606-4` 即 `-0.11606e-4`。全空白
/// 字段为零，这正是格式表示某项缺省的方式。
fn exponent_field(line: &str, first: usize, last: usize, line_number: usize) -> Result<f64> {
    let text = trimmed_field(line, first, last, line_number)?;
    let columns = Some((first, last));
    if text.is_empty() {
        return Ok(0.0);
    }

    // The last character is the exponent sign and the one before it is the
    // exponent magnitude, so the mantissa is everything before those two.
    // 末位字符是指数符号，其前一位是指数大小，故尾数为这两位之前的全部内容。
    let characters: Vec<char> = text.chars().collect();
    if characters.len() < 3 {
        return Err(
            ElementError::new(ElementErrorKind::InvalidField, line_number, columns)
                .with_detail(format!("exponent field {text:?} is too short")),
        );
    }
    let exponent_sign = match characters[characters.len() - 2] {
        '+' => 1,
        '-' => -1,
        other => {
            return Err(ElementError::new(
                ElementErrorKind::UnexpectedCharacter,
                line_number,
                columns,
            )
            .with_detail(format!("exponent sign is {other:?}, expected + or -")));
        }
    };
    let exponent_magnitude = characters[characters.len() - 1];
    let Some(exponent_magnitude) = exponent_magnitude.to_digit(10) else {
        return Err(
            ElementError::new(ElementErrorKind::UnexpectedCharacter, line_number, columns)
                .with_detail(format!(
                    "exponent is {exponent_magnitude:?}, expected a digit"
                )),
        );
    };

    let mantissa: String = characters[..characters.len() - 2].iter().collect();
    let mantissa = mantissa.replace(' ', "");
    let (mantissa_sign, mantissa) = if let Some(rest) = mantissa.strip_prefix('-') {
        (-1.0, rest.to_string())
    } else if let Some(rest) = mantissa.strip_prefix('+') {
        (1.0, rest.to_string())
    } else {
        (1.0, mantissa)
    };
    let digits: String = mantissa.chars().filter(char::is_ascii_digit).collect();
    if digits.is_empty() {
        return Err(
            ElementError::new(ElementErrorKind::InvalidField, line_number, columns)
                .with_detail(format!("mantissa is {mantissa:?}, expected digits")),
        );
    }
    let value = format!("0.{digits}").parse::<f64>().map_err(|_| {
        ElementError::new(ElementErrorKind::InvalidField, line_number, columns)
            .with_detail(format!("mantissa is {mantissa:?}"))
    })?;
    let exponent = exponent_sign * exponent_magnitude as i32;
    Ok(mantissa_sign * value * 10f64.powi(exponent))
}

/// Parses the eccentricity field, which is written with an assumed decimal
/// point before its first digit.
///
/// 解析偏心率字段，该字段按首位之前带隐含小数点书写。
fn assumed_decimal_field(line: &str, first: usize, last: usize, line_number: usize) -> Result<f64> {
    let text = trimmed_field(line, first, last, line_number)?;
    let columns = Some((first, last));
    if text.is_empty() || !text.chars().all(|character| character.is_ascii_digit()) {
        return Err(
            ElementError::new(ElementErrorKind::InvalidField, line_number, columns)
                .with_detail(format!("expected digits, found {text:?}")),
        );
    }
    format!("0.{text}").parse::<f64>().map_err(|_| {
        ElementError::new(ElementErrorKind::InvalidField, line_number, columns)
            .with_detail(format!("eccentricity is {text:?}"))
    })
}

/// Expands a two digit year using the convention that `57` through `99` are
/// in the twentieth century and `00` through `56` are in the twenty-first.
///
/// The boundary sits before the start of the satellite catalogue, so no
/// catalogue entry is placed a century away from its true date.
///
/// 按如下约定展开两位数年份：`57` 至 `99` 属二十世纪，`00` 至 `56` 属
/// 二十一世纪。
///
/// 该分界位于卫星编目开始之前，故不会有编目条目被放到距真实日期一个世纪之处。
pub const fn expand_two_digit_year(two_digit: u32) -> i32 {
    if two_digit >= 57 {
        1900 + two_digit as i32
    } else {
        2000 + two_digit as i32
    }
}

/// Parses the epoch field, which is a two digit year and a fractional day of
/// year, and resolves it on the UTC scale.
///
/// 解析历元字段，即两位数年份与年内的日小数，并在 UTC 标度上确定其值。
fn epoch_field(line: &str, first: usize, last: usize, line_number: usize) -> Result<Epoch> {
    let text = trimmed_field(line, first, last, line_number)?;
    let columns = Some((first, last));
    let invalid = |detail: String| {
        ElementError::new(ElementErrorKind::InvalidField, line_number, columns).with_detail(detail)
    };

    // The field is a two digit year followed by three digits of day of year
    // and then a fraction of that day.
    // 该字段为两位数年份，后接三位年日与该日的小数部分。
    if text.len() < 5 {
        return Err(invalid(format!("epoch {text:?} is too short")));
    }
    let (year_digits, rest) = text.split_at(2);
    let (day_digits, fraction_digits) = rest.split_at(3);
    if !year_digits.chars().all(|c| c.is_ascii_digit())
        || !day_digits.chars().all(|c| c.is_ascii_digit())
    {
        return Err(invalid(format!("epoch {text:?} is not numeric")));
    }
    let year = expand_two_digit_year(
        year_digits
            .parse::<u32>()
            .map_err(|_| invalid(format!("year {year_digits:?}")))?,
    );
    let day_of_year = day_digits
        .parse::<u32>()
        .map_err(|_| invalid(format!("day of year {day_digits:?}")))?;
    if !(1..=366).contains(&day_of_year) {
        return Err(invalid(format!(
            "day of year {day_of_year} is out of range"
        )));
    }
    let fraction = if fraction_digits.is_empty() {
        0.0
    } else {
        // The field writes the decimal point explicitly, so the fraction
        // begins with a dot. Strip it before forming the value, otherwise
        // the parse would see two decimal points.
        // 该字段显式写出小数点，故小数部分以点开头。构造数值前先去掉它，否则
        // 解析会遇到两个小数点。
        let digits = fraction_digits.strip_prefix('.').unwrap_or(fraction_digits);
        format!("0.{digits}")
            .parse::<f64>()
            .map_err(|_| invalid(format!("day fraction {fraction_digits:?}")))?
    };

    let calendar = calendar_from_day_of_year(year, day_of_year)?;
    let mjd = apsis_time::mjd_from_ymd(calendar.0, calendar.1, calendar.2) + fraction;
    Epoch::from_mjd(mjd, TimeScale::Utc)
        .ok_or_else(|| invalid(format!("epoch {text:?} is outside the leap second table")))
}

/// Converts a year and a day of year, both one based, to a calendar date.
///
/// 把从一开始的年与年日转换为日历日期。
fn calendar_from_day_of_year(year: i32, day_of_year: u32) -> Result<(i32, u32, u32)> {
    let mut remaining = day_of_year;
    let mut month = 1;
    while month <= 12 {
        let length = apsis_time::days_in_month(year, month);
        if remaining <= length {
            return Ok((year, month, remaining));
        }
        remaining -= length;
        month += 1;
    }
    Err(
        ElementError::new(ElementErrorKind::InvalidField, 0, None).with_detail(format!(
            "day of year {day_of_year} does not exist in {year}"
        )),
    )
}

impl TwoLineElement {
    /// Parses a two-line element set from one, two, or three lines.
    ///
    /// Line endings may be `\n`, `\r\n`, or a mix; blank lines and lines
    /// that are entirely whitespace are ignored. A leading line that does
    /// not begin with `1 ` is taken as the satellite name, which is how the
    /// optional three line form is written.
    ///
    /// Each line is checked for the expected line number in column one, for
    /// its length, and for its checksum. The element set number of the two
    /// lines must agree.
    ///
    /// Returns an error describing the first failure and the line on which
    /// it occurred.
    ///
    /// 从一行、两行或三行文本解析两线根数集。
    ///
    /// 行结束符可以是 `\n`、`\r\n` 或二者混用；空行与全为空白字符的行会被
    /// 忽略。不以 `1 ` 开头的首行被取作卫星名称，这正是可选三行形式的写法。
    ///
    /// 每一行都检查第一列的行号、长度与校验和。两行的根数集编号必须一致。
    ///
    /// 返回描述首个失败及其所在行的错误。
    pub fn parse(text: &str) -> Result<Self> {
        let lines: Vec<&str> = text
            .lines()
            .filter(|line| !line.trim().is_empty())
            .collect();

        let (name, first, second) = match lines.len() {
            2 => (None, lines[0], lines[1]),
            3 => (Some(lines[0].trim().to_string()), lines[1], lines[2]),
            count => {
                return Err(ElementError::new(ElementErrorKind::TooShort, 1, None)
                    .with_detail(format!("expected 2 or 3 non-empty lines, found {count}")));
            }
        };

        // The name line, when present, is not part of the checksummed
        // format.
        // 名称行若存在，不属于带校验和的格式。
        let first_number = if name.is_some() { 2 } else { 1 };
        let second_number = first_number + 1;

        verify_line_checksum(first, first_number)?;
        verify_line_checksum(second, second_number)?;

        if !first.starts_with('1') {
            return Err(ElementError::new(
                ElementErrorKind::UnexpectedCharacter,
                first_number,
                Some((1, 1)),
            )
            .with_detail("first data line must begin with 1"));
        }
        if !second.starts_with('2') {
            return Err(ElementError::new(
                ElementErrorKind::UnexpectedCharacter,
                second_number,
                Some((1, 1)),
            )
            .with_detail("second data line must begin with 2"));
        }

        let satellite_number = unsigned_field(first, 3, 7, first_number)?;
        let second_satellite_number = unsigned_field(second, 3, 7, second_number)?;
        if satellite_number != second_satellite_number {
            return Err(ElementError::new(
                ElementErrorKind::MismatchedElementSet,
                second_number,
                Some((3, 7)),
            )
            .with_detail(format!(
                "line {first_number} has {satellite_number}, line {second_number} has {second_satellite_number}"
            )));
        }

        let classification = match field(first, 8, 8).as_deref() {
            Some("U") => Classification::Unclassified,
            Some("S") => Classification::Secret,
            Some(other) => {
                let character = other.chars().next().unwrap_or(' ');
                Classification::Other(character)
            }
            None => {
                return Err(ElementError::new(
                    ElementErrorKind::TooShort,
                    first_number,
                    Some((8, 8)),
                ));
            }
        };

        let international_designator = trimmed_field(first, 10, 17, first_number)?;
        let epoch = epoch_field(first, 19, 32, first_number)?;
        let mean_motion_dot = float_field(first, 34, 43, first_number)?;
        let mean_motion_double_dot = exponent_field(first, 45, 52, first_number)?;
        let bstar = exponent_field(first, 54, 61, first_number)?;
        let element_set_number = unsigned_field(first, 65, 68, first_number)?;

        let inclination = float_field(second, 9, 16, second_number)?;
        let right_ascension = float_field(second, 18, 25, second_number)?;
        let eccentricity = assumed_decimal_field(second, 27, 33, second_number)?;
        let argument_of_perigee = float_field(second, 35, 42, second_number)?;
        let mean_anomaly = float_field(second, 44, 51, second_number)?;
        let mean_motion = float_field(second, 53, 63, second_number)?;
        let revolution_number = unsigned_field(second, 64, 68, second_number)?;

        Ok(Self {
            name,
            satellite_number,
            classification,
            international_designator,
            epoch,
            mean_motion_dot,
            mean_motion_double_dot,
            bstar,
            element_set_number,
            inclination,
            right_ascension,
            eccentricity,
            argument_of_perigee,
            mean_anomaly,
            mean_motion,
            revolution_number,
        })
    }
}
