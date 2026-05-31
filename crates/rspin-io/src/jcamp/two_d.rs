//! JCAMP-DX two-dimensional spectrum reader.

use std::str::FromStr;

use rspin_core::{Axis, Metadata, Nucleus, RSpinError, Result, Spectrum2D, Unit};

use crate::SpectrumReader;

use super::{JcampDxVersion, parse_jcamp_dx_version};

/// Reader for a focused JCAMP-DX 2D NTUPLES/page subset.
#[derive(Clone, Copy, Debug, Default)]
pub struct JcampDx2D;

impl SpectrumReader for JcampDx2D {
    type Output = Spectrum2D;

    fn read_str(&self, input: &str) -> Result<Self::Output> {
        read_jcamp_dx_2d(input)
    }
}

#[derive(Default)]
struct RawJcamp2D {
    version: Option<JcampDxVersion>,
    title: Option<String>,
    x_unit: Unit,
    y_unit: Unit,
    first_x: Option<f64>,
    last_x: Option<f64>,
    first_y: Option<f64>,
    last_y: Option<f64>,
    x_points: Option<usize>,
    y_points: Option<usize>,
    x_factor: Option<f64>,
    y_axis_factor: Option<f64>,
    z_factor: Option<f64>,
    nucleus: Option<Nucleus>,
    frequency_mhz: Option<f64>,
    solvent: Option<String>,
    temperature_k: Option<f64>,
    origin: Option<String>,
    pages: Vec<Jcamp2DPage>,
    saw_page_label: bool,
}

#[derive(Clone, Debug, Default)]
struct Jcamp2DPage {
    y_value: Option<f64>,
    channel: Channel2D,
    values: Vec<f64>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Channel2D {
    #[default]
    Real,
    Imaginary,
}

#[derive(Clone, Copy)]
enum DataBlock2D {
    PageValues(Channel2D),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AxisLayout {
    Declared,
    Swapped,
}

impl AxisLayout {
    fn direct_points(self, raw: &RawJcamp2D) -> Option<usize> {
        match self {
            Self::Declared => raw.x_points,
            Self::Swapped => raw.y_points,
        }
    }

    fn indirect_points(self, raw: &RawJcamp2D) -> Option<usize> {
        match self {
            Self::Declared => raw.y_points,
            Self::Swapped => raw.x_points,
        }
    }

    fn direct_unit(self, raw: &RawJcamp2D) -> Unit {
        match self {
            Self::Declared => raw.x_unit,
            Self::Swapped => raw.y_unit,
        }
    }

    fn indirect_unit(self, raw: &RawJcamp2D) -> Unit {
        match self {
            Self::Declared => raw.y_unit,
            Self::Swapped => raw.x_unit,
        }
    }

    fn direct_factor(self, raw: &RawJcamp2D) -> Option<f64> {
        match self {
            Self::Declared => raw.x_factor,
            Self::Swapped => raw.y_axis_factor,
        }
    }

    fn indirect_factor(self, raw: &RawJcamp2D) -> Option<f64> {
        match self {
            Self::Declared => raw.y_axis_factor,
            Self::Swapped => raw.x_factor,
        }
    }

    fn direct_first(self, raw: &RawJcamp2D) -> Option<f64> {
        match self {
            Self::Declared => raw.first_x,
            Self::Swapped => raw.first_y,
        }
    }

    fn direct_last(self, raw: &RawJcamp2D) -> Option<f64> {
        match self {
            Self::Declared => raw.last_x,
            Self::Swapped => raw.last_y,
        }
    }

    fn indirect_first(self, raw: &RawJcamp2D) -> Option<f64> {
        match self {
            Self::Declared => raw.first_y,
            Self::Swapped => raw.first_x,
        }
    }

    fn indirect_last(self, raw: &RawJcamp2D) -> Option<f64> {
        match self {
            Self::Declared => raw.last_y,
            Self::Swapped => raw.last_x,
        }
    }
}

/// Reads a two-dimensional spectrum from JCAMP-DX text.
///
/// This parser targets numeric or ASDF-compressed NTUPLES page data where each
/// page contains an `XYDATA`/`DATA TABLE` row sequence for one indirect-axis
/// coordinate. Real and imaginary pages may be encoded as paired `R..R`/`I..I`
/// data tables. It supports `VAR_DIM`, `FACTOR`, `FIRST`, `LAST`, `UNITS`,
/// `PAGE`, and common metadata labels.
///
/// # Errors
///
/// Returns an error when required page data are missing or malformed.
pub fn read_jcamp_dx_2d(input: &str) -> Result<Spectrum2D> {
    let mut raw = RawJcamp2D::default();
    let mut data_block = None;

    for line in input.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if let Some((key, value)) = super::parse_labeled_line(line) {
            let normalized_key = super::normalized_key(key);
            data_block = match normalized_key.as_str() {
                "XYDATA" => {
                    let channel = data_table_channel_2d(value);
                    set_current_page_channel(&mut raw, channel);
                    Some(DataBlock2D::PageValues(channel))
                }
                "DATATABLE" => {
                    let block = data_table_block_2d(value);
                    if let Some(DataBlock2D::PageValues(channel)) = block {
                        set_current_page_channel(&mut raw, channel);
                    }
                    block
                }
                _ => None,
            };
            apply_label_2d(&mut raw, &normalized_key, value)?;
            continue;
        }

        if line.starts_with("##") {
            data_block = None;
            continue;
        }

        if let Some((key, value)) = super::parse_comment_assignment(line) {
            apply_comment_assignment_2d(&mut raw, key, value)?;
            continue;
        }

        if line.starts_with('$') {
            continue;
        }

        match data_block {
            Some(DataBlock2D::PageValues(channel)) => {
                let page = current_page(&mut raw);
                page.channel = channel;
                super::parse_xydata_line(line, &mut page.values)?;
            }
            None => {}
        }
    }

    spectrum_from_raw_2d(&raw)
}

fn apply_label_2d(raw: &mut RawJcamp2D, key: &str, value: &str) -> Result<()> {
    match key {
        "JCAMPDX" => {
            let version = parse_jcamp_dx_version(value)?;
            version.validate_supported_by_current_reader()?;
            raw.version = Some(version);
        }
        "TITLE" => set_text(&mut raw.title, value),
        "ORIGIN" => set_text(&mut raw.origin, value),
        "SOLVENT" | "SOLVENTNAME" => set_text(&mut raw.solvent, value),
        "TEMPERATURE" | "TEMP" => raw.temperature_k = Some(parse_temperature_k(value)?),
        "OBSERVENUCLEUS" => raw.nucleus = Some(parse_nucleus(value)?),
        "OBSERVEFREQUENCY" => {
            raw.frequency_mhz = Some(parse_label_float("OBSERVE FREQUENCY", value)?);
        }
        "XUNITS" => raw.x_unit = parse_unit(value),
        "YUNITS" => raw.y_unit = parse_unit(value),
        "FIRSTX" => raw.first_x = Some(parse_label_float("FIRSTX", value)?),
        "LASTX" => raw.last_x = Some(parse_label_float("LASTX", value)?),
        "FIRSTY" => raw.first_y = Some(parse_label_float("FIRSTY", value)?),
        "LASTY" => raw.last_y = Some(parse_label_float("LASTY", value)?),
        "NPOINTS" => raw.x_points = Some(super::parse_usize("NPOINTS", clean_value(value))?),
        "XFACTOR" => raw.x_factor = Some(parse_label_float("XFACTOR", value)?),
        "YFACTOR" => raw.y_axis_factor = Some(parse_label_float("YFACTOR", value)?),
        "ZFACTOR" => raw.z_factor = Some(parse_label_float("ZFACTOR", value)?),
        "VARDIM" => apply_var_dim(raw, value)?,
        "FACTOR" => apply_factor(raw, value)?,
        "FIRST" => apply_first(raw, value)?,
        "LAST" => apply_last(raw, value)?,
        "UNITS" => apply_units(raw, value),
        "PAGE" => {
            raw.saw_page_label = true;
            raw.pages.push(Jcamp2DPage {
                y_value: parse_page_coordinate(value)?,
                channel: Channel2D::Real,
                values: Vec::new(),
            });
        }
        _ => {}
    }
    Ok(())
}

fn apply_comment_assignment_2d(raw: &mut RawJcamp2D, key: &str, value: &str) -> Result<()> {
    match super::normalized_key(key).as_str() {
        "ORIGIN" => set_text(&mut raw.origin, value),
        "SOLVENT" | "SOLVENTNAME" => set_text(&mut raw.solvent, value),
        "TEMPERATURE" | "TEMP" => raw.temperature_k = Some(parse_temperature_k(value)?),
        _ => {}
    }
    Ok(())
}

fn apply_var_dim(raw: &mut RawJcamp2D, value: &str) -> Result<()> {
    let values = parse_usize_list("VAR_DIM", value)?;
    if let Some(value) = list_value(&values, 0) {
        raw.x_points = Some(value);
    }
    if let Some(value) = list_value(&values, 1) {
        raw.y_points = Some(value);
    }
    Ok(())
}

fn apply_factor(raw: &mut RawJcamp2D, value: &str) -> Result<()> {
    let values = parse_numeric_list("FACTOR", value)?;
    if let Some(value) = list_value(&values, 0) {
        raw.x_factor = Some(value);
    }
    if let Some(value) = list_value(&values, 1) {
        raw.y_axis_factor = Some(value);
    }
    if let Some(value) = list_value(&values, 2) {
        raw.z_factor = Some(value);
    }
    Ok(())
}

fn apply_first(raw: &mut RawJcamp2D, value: &str) -> Result<()> {
    let values = parse_numeric_list("FIRST", value)?;
    if let Some(value) = list_value(&values, 0) {
        raw.first_x = Some(value);
    }
    if let Some(value) = list_value(&values, 1) {
        raw.first_y = Some(value);
    }
    Ok(())
}

fn apply_last(raw: &mut RawJcamp2D, value: &str) -> Result<()> {
    let values = parse_numeric_list("LAST", value)?;
    if let Some(value) = list_value(&values, 0) {
        raw.last_x = Some(value);
    }
    if let Some(value) = list_value(&values, 1) {
        raw.last_y = Some(value);
    }
    Ok(())
}

fn apply_units(raw: &mut RawJcamp2D, value: &str) {
    let values = split_list_values(value);
    if let Some(value) = values.first() {
        raw.x_unit = parse_unit(value);
    }
    if let Some(value) = values.get(1) {
        raw.y_unit = parse_unit(value);
    }
}

fn data_table_block_2d(value: &str) -> Option<DataBlock2D> {
    let upper = value.to_ascii_uppercase();
    if upper.contains("XYDATA") || upper.contains("PEAKS") || upper.contains("PROFILE") {
        Some(DataBlock2D::PageValues(data_table_channel_2d(value)))
    } else {
        None
    }
}

fn data_table_channel_2d(value: &str) -> Channel2D {
    let upper = value.to_ascii_uppercase();
    if upper.contains("I..I") {
        Channel2D::Imaginary
    } else {
        Channel2D::Real
    }
}

fn set_current_page_channel(raw: &mut RawJcamp2D, channel: Channel2D) {
    current_page(raw).channel = channel;
}

fn current_page(raw: &mut RawJcamp2D) -> &mut Jcamp2DPage {
    if raw.pages.is_empty() {
        raw.pages.push(Jcamp2DPage::default());
    }
    let index = raw.pages.len() - 1;
    &mut raw.pages[index]
}

fn spectrum_from_raw_2d(raw: &RawJcamp2D) -> Result<Spectrum2D> {
    if !raw.saw_page_label {
        return Err(RSpinError::Parse {
            format: "JCAMP-DX",
            message: "missing 2D JCAMP-DX PAGE labels".to_owned(),
        });
    }
    if raw.pages.is_empty() {
        return Err(RSpinError::Parse {
            format: "JCAMP-DX",
            message: "missing 2D JCAMP-DX pages".to_owned(),
        });
    }

    let real_page_indices = page_indices(raw, Channel2D::Real);
    let imaginary_page_indices = page_indices(raw, Channel2D::Imaginary);
    let layout = infer_axis_layout(raw, &real_page_indices);
    let width = infer_width(raw, &real_page_indices, layout)?;
    let height = infer_height(raw, real_page_indices.len(), layout)?;
    validate_imaginary_pages(raw, &real_page_indices, &imaginary_page_indices)?;
    let x_axis = x_axis(raw, width, layout)?;
    let y_axis = y_axis(raw, &real_page_indices, height, layout)?;
    let metadata = metadata_from_raw_2d(raw);
    let z_factor = super::option_or(raw.z_factor, 1.0);
    let z = scaled_page_values(
        raw,
        &real_page_indices,
        width,
        z_factor,
        "2D JCAMP-DX intensity",
    )?;
    let imaginary = if imaginary_page_indices.is_empty() {
        None
    } else {
        Some(scaled_page_values(
            raw,
            &imaginary_page_indices,
            width,
            z_factor,
            "2D JCAMP-DX imaginary intensity",
        )?)
    };

    Spectrum2D::new_complex(x_axis, y_axis, z, imaginary, metadata)
}

fn page_indices(raw: &RawJcamp2D, channel: Channel2D) -> Vec<usize> {
    raw.pages
        .iter()
        .enumerate()
        .filter_map(|(index, page)| {
            if page.channel == channel {
                Some(index)
            } else {
                None
            }
        })
        .collect()
}

fn infer_axis_layout(raw: &RawJcamp2D, page_indices: &[usize]) -> AxisLayout {
    let page_count = page_indices.len();
    let page_width = observed_page_width(raw, page_indices);
    if let (Some(first_dim), Some(second_dim), Some(page_width)) =
        (raw.x_points, raw.y_points, page_width)
    {
        let declared_layout_matches = first_dim == page_width && second_dim == page_count;
        let swapped_layout_matches = first_dim == page_count && page_width >= second_dim;
        if swapped_layout_matches && !declared_layout_matches {
            return AxisLayout::Swapped;
        }
    }

    AxisLayout::Declared
}

fn observed_page_width(raw: &RawJcamp2D, page_indices: &[usize]) -> Option<usize> {
    page_indices
        .iter()
        .filter_map(|index| raw.pages.get(*index))
        .find_map(|page| {
            if page.values.is_empty() {
                None
            } else {
                Some(page.values.len())
            }
        })
}

fn infer_width(raw: &RawJcamp2D, page_indices: &[usize], layout: AxisLayout) -> Result<usize> {
    let width = match layout.direct_points(raw) {
        Some(points) => points,
        None => observed_page_width(raw, page_indices).ok_or_else(|| RSpinError::Parse {
            format: "JCAMP-DX",
            message: "missing 2D JCAMP-DX page values".to_owned(),
        })?,
    };
    if width == 0 {
        return Err(RSpinError::Parse {
            format: "JCAMP-DX",
            message: "2D JCAMP-DX width must be positive".to_owned(),
        });
    }
    for (row_index, page_index) in page_indices.iter().copied().enumerate() {
        let Some(page) = raw.pages.get(page_index) else {
            return Err(RSpinError::Parse {
                format: "JCAMP-DX",
                message: "2D JCAMP-DX internal page index is invalid".to_owned(),
            });
        };
        if page.values.len() < width {
            return Err(RSpinError::Parse {
                format: "JCAMP-DX",
                message: format!(
                    "2D JCAMP-DX page {} has {} values but expected at least {width}",
                    row_index + 1,
                    page.values.len()
                ),
            });
        }
    }
    Ok(width)
}

fn infer_height(raw: &RawJcamp2D, real_page_count: usize, layout: AxisLayout) -> Result<usize> {
    let height = real_page_count;
    if height == 0 {
        return Err(RSpinError::Parse {
            format: "JCAMP-DX",
            message: "2D JCAMP-DX requires at least one real page".to_owned(),
        });
    }
    if let Some(declared) = layout.indirect_points(raw)
        && declared != height
    {
        return Err(RSpinError::Parse {
            format: "JCAMP-DX",
            message: format!(
                "2D JCAMP-DX declares {declared} y points but contains {height} pages"
            ),
        });
    }
    Ok(height)
}

fn x_axis(raw: &RawJcamp2D, width: usize, layout: AxisLayout) -> Result<Axis> {
    let factor = super::option_or(layout.direct_factor(raw), 1.0);
    let first = super::scale_value(
        "2D JCAMP-DX first x",
        super::option_or(layout.direct_first(raw), 0.0),
        factor,
    )?;
    let last_default = u32::try_from(width - 1).map_or(0.0, f64::from);
    let last = super::scale_value(
        "2D JCAMP-DX last x",
        super::option_or(layout.direct_last(raw), last_default),
        factor,
    )?;
    Axis::linear("x", layout.direct_unit(raw), first, last, width)
}

fn y_axis(
    raw: &RawJcamp2D,
    page_indices: &[usize],
    height: usize,
    layout: AxisLayout,
) -> Result<Axis> {
    let factor = super::option_or(layout.indirect_factor(raw), 1.0);
    if let Some(values) = page_y_values(raw, page_indices, factor)? {
        return Axis::new("y", layout.indirect_unit(raw), values);
    }

    let first = super::scale_value(
        "2D JCAMP-DX first y",
        super::option_or(layout.indirect_first(raw), 0.0),
        factor,
    )?;
    let last_default = u32::try_from(height - 1).map_or(0.0, f64::from);
    let last = super::scale_value(
        "2D JCAMP-DX last y",
        super::option_or(layout.indirect_last(raw), last_default),
        factor,
    )?;
    Axis::linear("y", layout.indirect_unit(raw), first, last, height)
}

fn page_y_values(
    raw: &RawJcamp2D,
    page_indices: &[usize],
    factor: f64,
) -> Result<Option<Vec<f64>>> {
    let mut values = Vec::with_capacity(page_indices.len());
    for index in page_indices {
        let Some(page) = raw.pages.get(*index) else {
            return Err(RSpinError::Parse {
                format: "JCAMP-DX",
                message: "2D JCAMP-DX internal page index is invalid".to_owned(),
            });
        };
        let Some(value) = page.y_value else {
            return Ok(None);
        };
        values.push(super::scale_value(
            "2D JCAMP-DX page coordinate",
            value,
            factor,
        )?);
    }
    Ok(Some(values))
}

fn validate_imaginary_pages(
    raw: &RawJcamp2D,
    real_page_indices: &[usize],
    imaginary_page_indices: &[usize],
) -> Result<()> {
    if imaginary_page_indices.is_empty() {
        return Ok(());
    }
    if imaginary_page_indices.len() != real_page_indices.len() {
        return Err(RSpinError::Parse {
            format: "JCAMP-DX",
            message: format!(
                "2D JCAMP-DX contains {} real pages but {} imaginary pages",
                real_page_indices.len(),
                imaginary_page_indices.len()
            ),
        });
    }

    for (row_index, (real_index, imaginary_index)) in real_page_indices
        .iter()
        .copied()
        .zip(imaginary_page_indices.iter().copied())
        .enumerate()
    {
        let Some(real_page) = raw.pages.get(real_index) else {
            return Err(RSpinError::Parse {
                format: "JCAMP-DX",
                message: "2D JCAMP-DX internal real page index is invalid".to_owned(),
            });
        };
        let Some(imaginary_page) = raw.pages.get(imaginary_index) else {
            return Err(RSpinError::Parse {
                format: "JCAMP-DX",
                message: "2D JCAMP-DX internal imaginary page index is invalid".to_owned(),
            });
        };
        if let (Some(real_y), Some(imaginary_y)) = (real_page.y_value, imaginary_page.y_value)
            && !close_enough(real_y, imaginary_y)
        {
            return Err(RSpinError::Parse {
                format: "JCAMP-DX",
                message: format!(
                    "2D JCAMP-DX imaginary page {} coordinate does not match real page",
                    row_index + 1
                ),
            });
        }
    }
    Ok(())
}

fn close_enough(left: f64, right: f64) -> bool {
    let tolerance = 1.0e-10 * left.abs().max(right.abs()).max(1.0);
    (left - right).abs() <= tolerance
}

fn scaled_page_values(
    raw: &RawJcamp2D,
    page_indices: &[usize],
    width: usize,
    factor: f64,
    field: &'static str,
) -> Result<Vec<f64>> {
    let mut values = Vec::with_capacity(width * page_indices.len());
    for (row_index, page_index) in page_indices.iter().copied().enumerate() {
        let page = raw.pages.get(page_index).ok_or_else(|| RSpinError::Parse {
            format: "JCAMP-DX",
            message: "2D JCAMP-DX internal page index is invalid".to_owned(),
        })?;
        if page.values.len() < width {
            return Err(RSpinError::Parse {
                format: "JCAMP-DX",
                message: format!(
                    "2D JCAMP-DX page {} has {} values but expected at least {width}",
                    row_index + 1,
                    page.values.len()
                ),
            });
        }
        for value in page.values.iter().copied().take(width) {
            values.push(super::scale_value(field, value, factor)?);
        }
    }
    Ok(values)
}

fn metadata_from_raw_2d(raw: &RawJcamp2D) -> Metadata {
    let mut metadata = Metadata {
        name: raw.title.clone(),
        nucleus: raw.nucleus.clone(),
        frequency_mhz: raw.frequency_mhz,
        solvent: raw.solvent.clone(),
        temperature_k: raw.temperature_k,
        origin: raw.origin.clone(),
        ..Metadata::default()
    };
    if let Some(version) = raw.version.as_ref() {
        metadata
            .properties
            .insert("jcamp_dx.version".to_owned(), version.raw.clone());
    }
    metadata
}

fn parse_page_coordinate(value: &str) -> Result<Option<f64>> {
    let value = clean_value(value);
    if value.is_empty() {
        return Ok(None);
    }

    if value.contains('=') {
        for assignment in value.split([',', ';']) {
            let Some((key, value)) = assignment.split_once('=') else {
                continue;
            };
            match super::normalized_key(key).as_str() {
                "F1" | "F2" | "Y" | "YVALUE" | "INDIRECT" => {
                    return Ok(Some(parse_label_float("PAGE", value)?));
                }
                _ => {}
            }
        }
        return Ok(None);
    }

    let token = numeric_prefix("PAGE", value)?;
    Ok(Some(super::parse_float("PAGE", token)?))
}

fn parse_nucleus(value: &str) -> Result<Nucleus> {
    let value = clean_value(value).trim_start_matches('^');
    Nucleus::from_str(value)
}

fn parse_temperature_k(value: &str) -> Result<f64> {
    let numeric_value = parse_label_float("TEMPERATURE", value)?;
    let normalized = super::normalized_key(clean_value(value));
    let temperature_k = if normalized.contains("KELVIN") || normalized.ends_with('K') {
        numeric_value
    } else if normalized.contains("CELSIUS") || normalized.ends_with('C') || numeric_value < 170.0 {
        numeric_value + 273.15
    } else {
        numeric_value
    };

    if temperature_k.is_finite() {
        Ok(temperature_k)
    } else {
        Err(RSpinError::NonFinite {
            field: "TEMPERATURE",
        })
    }
}

fn parse_label_float(field: &'static str, value: &str) -> Result<f64> {
    let token = numeric_prefix(field, clean_value(value))?;
    super::parse_float(field, token)
}

fn numeric_prefix<'a>(field: &'static str, value: &'a str) -> Result<&'a str> {
    let value = value.trim();
    let end = match value.char_indices().find_map(|(index, character)| {
        if is_numeric_prefix_character(character) {
            None
        } else {
            Some(index)
        }
    }) {
        Some(index) => index,
        None => value.len(),
    };
    let token = value[..end].trim();
    if token.is_empty() {
        return Err(RSpinError::Parse {
            format: "JCAMP-DX",
            message: format!("{field}: expected numeric value"),
        });
    }
    Ok(token)
}

fn parse_numeric_list(field: &'static str, value: &str) -> Result<Vec<f64>> {
    split_list_values(value)
        .into_iter()
        .map(|token| parse_label_float(field, &token))
        .collect()
}

fn parse_usize_list(field: &'static str, value: &str) -> Result<Vec<usize>> {
    split_list_values(value)
        .into_iter()
        .map(|token| super::parse_usize(field, &token))
        .collect()
}

fn split_list_values(value: &str) -> Vec<String> {
    clean_value(value)
        .split(',')
        .map(clean_list_token)
        .filter(|token| !token.is_empty())
        .collect()
}

fn clean_list_token(value: &str) -> String {
    value
        .trim()
        .trim_start_matches('(')
        .trim_end_matches(')')
        .trim()
        .to_owned()
}

fn list_value<T: Copy>(values: &[T], index: usize) -> Option<T> {
    values.iter().copied().nth(index)
}

fn set_text(target: &mut Option<String>, value: &str) {
    let value = clean_value(value);
    if !value.is_empty() {
        *target = Some(value.to_owned());
    }
}

fn clean_value(value: &str) -> &str {
    super::clean_label_value(value)
}

fn parse_unit(value: &str) -> Unit {
    match super::normalized_key(clean_value(value)).as_str() {
        "PPM" => Unit::Ppm,
        "HZ" | "HERTZ" => Unit::Hertz,
        "SECONDS" | "SECOND" | "SEC" | "S" => Unit::Seconds,
        "POINTS" | "POINT" => Unit::Points,
        _ => Unit::Arbitrary,
    }
}

fn is_numeric_prefix_character(character: char) -> bool {
    character.is_ascii_digit()
        || character == '.'
        || character == '+'
        || character == '-'
        || character == 'E'
        || character == 'e'
}
