use rspin_core::{Axis, Unit};

use crate::{
    Abs1D, Abs2D, BaselineMethod, Crop1D, Crop2D, ExponentialApodization, ExponentialApodization2D,
    Fft1D, Fft2D, FftDirection, GaussianApodization, GaussianApodization2D, Magnitude,
    Normalize2DMaxAbs, Normalize2DVolume, NormalizeArea, NormalizeMaxAbs, Offset2D,
    OffsetIntensity, PhaseCorrection, Resample1D, Resample2D, Scale2D, ScaleIntensity, Shift2DAxes,
    ShiftAxis, SineBellApodization, SineBellApodization2D, SubtractBaseline, ZeroFill, ZeroFill2D,
};

#[test]
fn one_dimensional_step_builders_are_chainable() -> anyhow::Result<()> {
    let target_axis = Axis::linear("1H", Unit::Ppm, 0.0, 1.0, 4)?;

    assert_eq!(Abs1D::new(), Abs1D);
    assert_eq!(ScaleIntensity::new(2.0), ScaleIntensity { factor: 2.0 });
    assert_eq!(OffsetIntensity::new(-0.5), OffsetIntensity { offset: -0.5 });
    assert_eq!(NormalizeMaxAbs::new(), NormalizeMaxAbs);
    assert_eq!(
        NormalizeArea::new(10.0).with_absolute_intensity(true),
        NormalizeArea::absolute(10.0)
    );
    assert_eq!(ShiftAxis::new(0.03), ShiftAxis { delta: 0.03 });
    assert_eq!(ZeroFill::new(16), ZeroFill { target_len: 16 });
    assert_eq!(Crop1D::new(7.0, 0.0), Crop1D { from: 7.0, to: 0.0 });
    assert_eq!(
        Resample1D::new(target_axis.clone()).with_outside_value(-1.0),
        Resample1D {
            target_axis,
            outside_value: -1.0,
        }
    );
    assert_eq!(
        ExponentialApodization::new(0.3, 0.001),
        ExponentialApodization {
            line_broadening_hz: 0.3,
            dwell_time_s: 0.001,
        }
    );
    assert_eq!(
        GaussianApodization::new(0.4, 0.001),
        GaussianApodization {
            gaussian_broadening_hz: 0.4,
            dwell_time_s: 0.001,
        }
    );
    assert_eq!(
        SineBellApodization::new(10.0, 170.0, 1.5),
        SineBellApodization {
            start_angle_deg: 10.0,
            end_angle_deg: 170.0,
            exponent: 1.5,
        }
    );
    assert_eq!(Magnitude::new(), Magnitude);
    assert_eq!(Fft1D::forward(), Fft1D::new(FftDirection::Forward));
    assert_eq!(Fft1D::inverse(), Fft1D::new(FftDirection::Inverse));
    assert_eq!(
        PhaseCorrection::new()
            .zero_order(-12.0)
            .first_order(3.0)
            .pivot_fraction(0.25),
        PhaseCorrection::from_degrees(-12.0, 3.0, 0.25)
    );
    assert_eq!(
        SubtractBaseline::default().with_method(BaselineMethod::Constant { value: 1.0 }),
        SubtractBaseline::new(BaselineMethod::Constant { value: 1.0 })
    );

    Ok(())
}

#[test]
fn two_dimensional_step_builders_are_chainable() -> anyhow::Result<()> {
    let target_x = Axis::linear("1H", Unit::Ppm, -1.0, 1.0, 5)?;
    let target_y = Axis::linear("13C", Unit::Ppm, 10.0, 20.0, 4)?;

    assert_eq!(Abs2D::new(), Abs2D);
    assert_eq!(Scale2D::new(0.5), Scale2D { factor: 0.5 });
    assert_eq!(Offset2D::new(-0.25), Offset2D { offset: -0.25 });
    assert_eq!(Normalize2DMaxAbs::new(), Normalize2DMaxAbs);
    assert_eq!(
        Normalize2DVolume::new(3.0).with_absolute_intensity(true),
        Normalize2DVolume::absolute(3.0)
    );
    assert_eq!(
        ZeroFill2D::new(8, 6),
        ZeroFill2D {
            target_x_len: 8,
            target_y_len: 6,
        }
    );
    assert_eq!(
        Crop2D::new(8.0, 0.0, 120.0, 30.0),
        Crop2D {
            x_from: 8.0,
            x_to: 0.0,
            y_from: 120.0,
            y_to: 30.0,
        }
    );
    assert_eq!(
        Resample2D::new(target_x.clone(), target_y.clone()).with_outside_value(-1.0),
        Resample2D {
            target_x,
            target_y,
            outside_value: -1.0,
        }
    );
    assert_eq!(
        ExponentialApodization2D::new(0.3, 0.6, 0.001, 0.002),
        ExponentialApodization2D {
            x_line_broadening_hz: 0.3,
            y_line_broadening_hz: 0.6,
            x_dwell_time_s: 0.001,
            y_dwell_time_s: 0.002,
        }
    );
    assert_eq!(
        GaussianApodization2D::new(0.4, 0.8, 0.001, 0.002),
        GaussianApodization2D {
            x_gaussian_broadening_hz: 0.4,
            y_gaussian_broadening_hz: 0.8,
            x_dwell_time_s: 0.001,
            y_dwell_time_s: 0.002,
        }
    );
    assert_eq!(
        SineBellApodization2D::new(10.0, 170.0, 1.5, 20.0, 160.0, 2.0),
        SineBellApodization2D {
            x_start_angle_deg: 10.0,
            x_end_angle_deg: 170.0,
            x_exponent: 1.5,
            y_start_angle_deg: 20.0,
            y_end_angle_deg: 160.0,
            y_exponent: 2.0,
        }
    );
    assert_eq!(Fft2D::forward(), Fft2D::new(FftDirection::Forward));
    assert_eq!(Fft2D::inverse(), Fft2D::new(FftDirection::Inverse));
    assert_eq!(
        Shift2DAxes::x(0.1).with_y_delta(-0.2),
        Shift2DAxes::new(0.1, -0.2)
    );
    assert_eq!(
        Shift2DAxes::y(0.3).with_x_delta(-0.4),
        Shift2DAxes::new(-0.4, 0.3)
    );

    Ok(())
}
