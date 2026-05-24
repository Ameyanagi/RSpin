use std::fmt::Debug;

use rspin_core::{Axis, Unit};
use serde::{Serialize, de::DeserializeOwned};

use crate::{
    Abs1D, Abs2D, AutoPhaseCorrection, AutoPhaseCorrection2D, BaselineMethod, Crop1D, Crop2D,
    ExponentialApodization, ExponentialApodization2D, Fft1D, Fft2D, FftDirection,
    GaussianApodization, GaussianApodization2D, Magnitude, Normalize2DMaxAbs, Normalize2DVolume,
    NormalizeArea, NormalizeMaxAbs, OffsetIntensity, PhaseCorrection, PhaseCorrection2D,
    ProjectionMode, Resample1D, Resample2D, Scale2D, ScaleIntensity, ShiftAxis,
    SineBellApodization, SineBellApodization2D, SubtractBaseline, ZeroFill, ZeroFill2D,
};

#[test]
fn serializes_one_dimensional_steps() -> anyhow::Result<()> {
    let target_axis = Axis::linear("1H", Unit::Ppm, -1.0, 1.0, 5)?;

    round_trip(&Abs1D)?;
    round_trip(&ScaleIntensity { factor: 2.0 })?;
    round_trip(&OffsetIntensity { offset: -0.5 })?;
    round_trip(&NormalizeMaxAbs)?;
    round_trip(&NormalizeArea::absolute(10.0))?;
    round_trip(&ShiftAxis { delta: 0.03 })?;
    round_trip(&ZeroFill { target_len: 16 })?;
    round_trip(&Crop1D { from: 7.0, to: 0.0 })?;
    round_trip(&Resample1D::new(target_axis).with_outside_value(-1.0))?;
    round_trip(&ExponentialApodization {
        line_broadening_hz: 0.3,
        dwell_time_s: 0.001,
    })?;
    round_trip(&GaussianApodization {
        gaussian_broadening_hz: 0.4,
        dwell_time_s: 0.001,
    })?;
    round_trip(&SineBellApodization {
        start_angle_deg: 10.0,
        end_angle_deg: 170.0,
        exponent: 1.5,
    })?;
    round_trip(&Magnitude)?;
    round_trip(&Fft1D {
        direction: FftDirection::Forward,
    })?;
    round_trip(&PhaseCorrection {
        zero_order_deg: -12.0,
        first_order_deg: 3.0,
        pivot_fraction: 0.25,
    })?;
    round_trip(&AutoPhaseCorrection::new().zero_order_range(-15.0, 15.0, 1.0))?;
    round_trip(&SubtractBaseline {
        method: BaselineMethod::WhittakerAsls {
            lambda: 1.0e6,
            p: 0.01,
            max_iter: 25,
            tolerance: 1.0e-4,
        },
    })?;

    Ok(())
}

#[test]
fn serializes_two_dimensional_steps() -> anyhow::Result<()> {
    let target_x = Axis::linear("1H", Unit::Ppm, -1.0, 1.0, 5)?;
    let target_y = Axis::linear("13C", Unit::Ppm, 10.0, 20.0, 4)?;

    round_trip(&Abs2D)?;
    round_trip(&Scale2D { factor: 0.5 })?;
    round_trip(&Normalize2DMaxAbs)?;
    round_trip(&Normalize2DVolume::absolute(3.0))?;
    round_trip(&ZeroFill2D {
        target_x_len: 8,
        target_y_len: 6,
    })?;
    round_trip(&Crop2D {
        x_from: 8.0,
        x_to: 0.0,
        y_from: 120.0,
        y_to: 30.0,
    })?;
    round_trip(&Resample2D::new(target_x, target_y).with_outside_value(-1.0))?;
    round_trip(&ExponentialApodization2D {
        x_line_broadening_hz: 0.3,
        y_line_broadening_hz: 0.6,
        x_dwell_time_s: 0.001,
        y_dwell_time_s: 0.002,
    })?;
    round_trip(&GaussianApodization2D {
        x_gaussian_broadening_hz: 0.4,
        y_gaussian_broadening_hz: 0.8,
        x_dwell_time_s: 0.001,
        y_dwell_time_s: 0.002,
    })?;
    round_trip(&SineBellApodization2D {
        x_start_angle_deg: 10.0,
        x_end_angle_deg: 170.0,
        x_exponent: 1.5,
        y_start_angle_deg: 20.0,
        y_end_angle_deg: 160.0,
        y_exponent: 2.0,
    })?;
    round_trip(&Fft2D {
        direction: FftDirection::Inverse,
    })?;
    round_trip(
        &PhaseCorrection2D::new()
            .x_phase(-5.0, 1.0, 0.3)
            .y_phase(2.0, -1.0, 0.4),
    )?;
    round_trip(
        &AutoPhaseCorrection2D::new()
            .x_zero_order_range(-10.0, 10.0, 2.0)
            .y_zero_order_range(-4.0, 4.0, 1.0),
    )?;

    Ok(())
}

#[test]
fn serializes_projection_mode_as_snake_case() -> anyhow::Result<()> {
    let json = serde_json::to_string(&ProjectionMode::MaxAbs)?;
    assert_eq!(json, "\"max_abs\"");

    let decoded: ProjectionMode = serde_json::from_str(&json)?;
    assert_eq!(decoded, ProjectionMode::MaxAbs);

    Ok(())
}

fn round_trip<T>(value: &T) -> anyhow::Result<()>
where
    T: Serialize + DeserializeOwned + PartialEq + Debug,
{
    let json = serde_json::to_string(value)?;
    let decoded = serde_json::from_str::<T>(&json)?;
    assert_eq!(&decoded, value);
    Ok(())
}
