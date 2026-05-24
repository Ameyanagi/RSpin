//! Chainable exact spin-system simulation workflows.

use rspin_core::{Result, Spectrum1D, Spectrum2D};

use crate::LineShape;

use super::{
    ExactSpectrum2DOptions, ExactSpectrumDecomposition1D, ExactSpectrumDecomposition2D,
    ExactSpectrumOptions, ExactSpinOptions, ExactSpinPair, ExactTransition, SpinHalfSystem,
    decompose_exact_spin_half_1d, decompose_exact_spin_half_2d, exact_spin_half_transitions,
    simulate_exact_spin_half_1d, simulate_exact_spin_half_2d,
};

/// Extension trait for chainable exact spin-system simulation.
pub trait SimulateExactSpinHalf {
    /// Starts a borrowed exact simulation workflow.
    #[must_use]
    fn simulate_exact(&self) -> ExactSpinHalfWorkflow<'_>;
}

impl SimulateExactSpinHalf for SpinHalfSystem {
    fn simulate_exact(&self) -> ExactSpinHalfWorkflow<'_> {
        ExactSpinHalfWorkflow::new(self)
    }
}

/// Extension trait for chainable exact simulation from fallible system inputs.
pub trait SimulateExactSpinHalfResult {
    /// Starts an owned exact simulation workflow from an existing result.
    #[must_use]
    fn simulate_exact(self) -> ExactSpinHalfResultWorkflow;
}

impl SimulateExactSpinHalfResult for Result<SpinHalfSystem> {
    fn simulate_exact(self) -> ExactSpinHalfResultWorkflow {
        ExactSpinHalfResultWorkflow::from_result(self)
    }
}

/// Borrowed exact simulation workflow.
#[derive(Clone, Debug)]
pub struct ExactSpinHalfWorkflow<'a> {
    system: &'a SpinHalfSystem,
    transition_options: ExactSpinOptions,
}

impl<'a> ExactSpinHalfWorkflow<'a> {
    /// Creates an exact simulation workflow for `system`.
    #[must_use]
    pub fn new(system: &'a SpinHalfSystem) -> Self {
        Self {
            system,
            transition_options: ExactSpinOptions::default(),
        }
    }

    /// Replaces exact transition options.
    #[must_use]
    pub fn with_transition_options(mut self, transition_options: ExactSpinOptions) -> Self {
        self.transition_options = transition_options;
        self
    }

    /// Sets the spectrometer frequency in MHz.
    #[must_use]
    pub fn with_spectrometer_mhz(mut self, spectrometer_mhz: f64) -> Self {
        self.transition_options.spectrometer_mhz = spectrometer_mhz;
        self
    }

    /// Sets the transition intensity threshold.
    #[must_use]
    pub fn with_intensity_threshold(mut self, intensity_threshold: f64) -> Self {
        self.transition_options.intensity_threshold = intensity_threshold;
        self
    }

    /// Sets the merge tolerance for transition frequencies in Hz.
    #[must_use]
    pub fn with_frequency_tolerance_hz(mut self, frequency_tolerance_hz: f64) -> Self {
        self.transition_options.frequency_tolerance_hz = frequency_tolerance_hz;
        self
    }

    /// Sets the per-call exact solver spin-count limit.
    #[must_use]
    pub fn with_max_spins(mut self, max_spins: usize) -> Self {
        self.transition_options.max_spins = max_spins;
        self
    }

    /// Adds one zero-based spin index to the detection operator.
    #[must_use]
    pub fn with_detected_spin(mut self, spin_index: usize) -> Self {
        self.transition_options.detected_spins.push(spin_index);
        self
    }

    /// Replaces the detected spin list.
    ///
    /// An empty list detects all spins.
    #[must_use]
    pub fn with_detected_spins(mut self, detected_spins: impl IntoIterator<Item = usize>) -> Self {
        self.transition_options.detected_spins = detected_spins.into_iter().collect();
        self
    }

    /// Simulates exact transition lines.
    ///
    /// # Errors
    ///
    /// Returns an error when the system or transition options are invalid.
    pub fn transitions(self) -> Result<Vec<ExactTransition>> {
        exact_spin_half_transitions(self.system, &self.transition_options)
    }

    /// Starts a chainable one-dimensional spectrum rendering workflow.
    #[must_use]
    pub fn render_1d(self) -> ExactSpinHalfSpectrum1DWorkflow<'a> {
        ExactSpinHalfSpectrum1DWorkflow::new(self.system, self.transition_options)
    }

    /// Starts a chainable two-dimensional spectrum rendering workflow.
    #[must_use]
    pub fn render_2d(self) -> ExactSpinHalfSpectrum2DWorkflow<'a> {
        ExactSpinHalfSpectrum2DWorkflow::new(self.system, self.transition_options)
    }

    /// Simulates a one-dimensional spectrum with default rendering options.
    ///
    /// # Errors
    ///
    /// Returns an error when the system, transition options, or rendering
    /// options are invalid.
    pub fn spectrum_1d(self) -> Result<Spectrum1D> {
        self.render_1d().run()
    }

    /// Simulates a two-dimensional spectrum with default rendering options.
    ///
    /// # Errors
    ///
    /// Returns an error when the system, transition options, spin pairs, or
    /// rendering options are invalid.
    pub fn spectrum_2d(self) -> Result<Spectrum2D> {
        self.render_2d().run()
    }
}

/// Owned exact simulation workflow from a fallible spin-system input.
#[derive(Debug)]
pub struct ExactSpinHalfResultWorkflow {
    system: Result<SpinHalfSystem>,
    transition_options: ExactSpinOptions,
}

impl ExactSpinHalfResultWorkflow {
    /// Creates an exact simulation workflow from an existing result.
    #[must_use]
    pub fn from_result(system: Result<SpinHalfSystem>) -> Self {
        Self {
            system,
            transition_options: ExactSpinOptions::default(),
        }
    }

    /// Replaces exact transition options.
    #[must_use]
    pub fn with_transition_options(mut self, transition_options: ExactSpinOptions) -> Self {
        self.transition_options = transition_options;
        self
    }

    /// Sets the spectrometer frequency in MHz.
    #[must_use]
    pub fn with_spectrometer_mhz(mut self, spectrometer_mhz: f64) -> Self {
        self.transition_options.spectrometer_mhz = spectrometer_mhz;
        self
    }

    /// Sets the transition intensity threshold.
    #[must_use]
    pub fn with_intensity_threshold(mut self, intensity_threshold: f64) -> Self {
        self.transition_options.intensity_threshold = intensity_threshold;
        self
    }

    /// Sets the merge tolerance for transition frequencies in Hz.
    #[must_use]
    pub fn with_frequency_tolerance_hz(mut self, frequency_tolerance_hz: f64) -> Self {
        self.transition_options.frequency_tolerance_hz = frequency_tolerance_hz;
        self
    }

    /// Sets the per-call exact solver spin-count limit.
    #[must_use]
    pub fn with_max_spins(mut self, max_spins: usize) -> Self {
        self.transition_options.max_spins = max_spins;
        self
    }

    /// Adds one zero-based spin index to the detection operator.
    #[must_use]
    pub fn with_detected_spin(mut self, spin_index: usize) -> Self {
        self.transition_options.detected_spins.push(spin_index);
        self
    }

    /// Replaces the detected spin list.
    ///
    /// An empty list detects all spins.
    #[must_use]
    pub fn with_detected_spins(mut self, detected_spins: impl IntoIterator<Item = usize>) -> Self {
        self.transition_options.detected_spins = detected_spins.into_iter().collect();
        self
    }

    /// Simulates exact transition lines.
    ///
    /// # Errors
    ///
    /// Returns the initial system error, or an error from transition simulation.
    pub fn transitions(self) -> Result<Vec<ExactTransition>> {
        let system = self.system?;
        exact_spin_half_transitions(&system, &self.transition_options)
    }

    /// Starts a chainable one-dimensional spectrum rendering workflow.
    #[must_use]
    pub fn render_1d(self) -> ExactSpinHalfSpectrum1DResultWorkflow {
        ExactSpinHalfSpectrum1DResultWorkflow::new(self.system, self.transition_options)
    }

    /// Starts a chainable two-dimensional spectrum rendering workflow.
    #[must_use]
    pub fn render_2d(self) -> ExactSpinHalfSpectrum2DResultWorkflow {
        ExactSpinHalfSpectrum2DResultWorkflow::new(self.system, self.transition_options)
    }

    /// Simulates a one-dimensional spectrum with default rendering options.
    ///
    /// # Errors
    ///
    /// Returns the initial system error, or an error from rendering.
    pub fn spectrum_1d(self) -> Result<Spectrum1D> {
        self.render_1d().run()
    }

    /// Simulates a two-dimensional spectrum with default rendering options.
    ///
    /// # Errors
    ///
    /// Returns the initial system error, or an error from rendering.
    pub fn spectrum_2d(self) -> Result<Spectrum2D> {
        self.render_2d().run()
    }
}

/// Borrowed one-dimensional exact spectrum rendering workflow.
#[derive(Clone, Debug)]
pub struct ExactSpinHalfSpectrum1DWorkflow<'a> {
    system: &'a SpinHalfSystem,
    options: ExactSpectrumOptions,
}

impl<'a> ExactSpinHalfSpectrum1DWorkflow<'a> {
    /// Creates a one-dimensional exact spectrum workflow.
    #[must_use]
    pub fn new(system: &'a SpinHalfSystem, transition_options: ExactSpinOptions) -> Self {
        Self {
            system,
            options: exact_spectrum_1d_options(transition_options),
        }
    }

    /// Replaces all one-dimensional rendering options.
    #[must_use]
    pub fn with_options(mut self, options: ExactSpectrumOptions) -> Self {
        self.options = options;
        self
    }

    /// Sets the output ppm range.
    #[must_use]
    pub fn with_ppm_range(mut self, from_ppm: f64, to_ppm: f64) -> Self {
        self.options.from_ppm = from_ppm;
        self.options.to_ppm = to_ppm;
        self
    }

    /// Sets the number of output points.
    #[must_use]
    pub fn with_points(mut self, points: usize) -> Self {
        self.options.points = points;
        self
    }

    /// Sets the integrated spectrum area.
    #[must_use]
    pub fn with_area(mut self, area: f64) -> Self {
        self.options.area = area;
        self
    }

    /// Sets the full width at half maximum in Hz.
    #[must_use]
    pub fn with_line_width_hz(mut self, line_width_hz: f64) -> Self {
        self.options.line_width_hz = line_width_hz;
        self
    }

    /// Sets the rendered line shape.
    #[must_use]
    pub fn with_line_shape(mut self, line_shape: LineShape) -> Self {
        self.options.line_shape = line_shape;
        self
    }

    /// Replaces exact transition generation options.
    #[must_use]
    pub fn with_transition_options(mut self, transition_options: ExactSpinOptions) -> Self {
        self.options.transition_options = transition_options;
        self
    }

    /// Sets the spectrometer frequency in MHz.
    #[must_use]
    pub fn with_spectrometer_mhz(mut self, spectrometer_mhz: f64) -> Self {
        self.options.transition_options.spectrometer_mhz = spectrometer_mhz;
        self
    }

    /// Simulates the configured one-dimensional spectrum.
    ///
    /// # Errors
    ///
    /// Returns an error when the system, transition options, or rendering
    /// options are invalid.
    pub fn run(self) -> Result<Spectrum1D> {
        simulate_exact_spin_half_1d(self.system, &self.options)
    }

    /// Simulates the configured spectrum and per-transition contributions.
    ///
    /// # Errors
    ///
    /// Returns an error when the system, transition options, or rendering
    /// options are invalid.
    pub fn decompose(self) -> Result<ExactSpectrumDecomposition1D> {
        decompose_exact_spin_half_1d(self.system, &self.options)
    }
}

/// Owned one-dimensional exact spectrum workflow from a fallible spin-system input.
#[derive(Debug)]
pub struct ExactSpinHalfSpectrum1DResultWorkflow {
    system: Result<SpinHalfSystem>,
    options: ExactSpectrumOptions,
}

impl ExactSpinHalfSpectrum1DResultWorkflow {
    /// Creates a one-dimensional exact spectrum workflow from an existing result.
    #[must_use]
    pub fn new(system: Result<SpinHalfSystem>, transition_options: ExactSpinOptions) -> Self {
        Self {
            system,
            options: exact_spectrum_1d_options(transition_options),
        }
    }

    /// Replaces all one-dimensional rendering options.
    #[must_use]
    pub fn with_options(mut self, options: ExactSpectrumOptions) -> Self {
        self.options = options;
        self
    }

    /// Sets the output ppm range.
    #[must_use]
    pub fn with_ppm_range(mut self, from_ppm: f64, to_ppm: f64) -> Self {
        self.options.from_ppm = from_ppm;
        self.options.to_ppm = to_ppm;
        self
    }

    /// Sets the number of output points.
    #[must_use]
    pub fn with_points(mut self, points: usize) -> Self {
        self.options.points = points;
        self
    }

    /// Sets the integrated spectrum area.
    #[must_use]
    pub fn with_area(mut self, area: f64) -> Self {
        self.options.area = area;
        self
    }

    /// Sets the full width at half maximum in Hz.
    #[must_use]
    pub fn with_line_width_hz(mut self, line_width_hz: f64) -> Self {
        self.options.line_width_hz = line_width_hz;
        self
    }

    /// Sets the rendered line shape.
    #[must_use]
    pub fn with_line_shape(mut self, line_shape: LineShape) -> Self {
        self.options.line_shape = line_shape;
        self
    }

    /// Replaces exact transition generation options.
    #[must_use]
    pub fn with_transition_options(mut self, transition_options: ExactSpinOptions) -> Self {
        self.options.transition_options = transition_options;
        self
    }

    /// Sets the spectrometer frequency in MHz.
    #[must_use]
    pub fn with_spectrometer_mhz(mut self, spectrometer_mhz: f64) -> Self {
        self.options.transition_options.spectrometer_mhz = spectrometer_mhz;
        self
    }

    /// Simulates the configured one-dimensional spectrum.
    ///
    /// # Errors
    ///
    /// Returns the initial system error, or an error from rendering.
    pub fn run(self) -> Result<Spectrum1D> {
        let system = self.system?;
        simulate_exact_spin_half_1d(&system, &self.options)
    }

    /// Simulates the configured spectrum and per-transition contributions.
    ///
    /// # Errors
    ///
    /// Returns the initial system error, or an error from rendering.
    pub fn decompose(self) -> Result<ExactSpectrumDecomposition1D> {
        let system = self.system?;
        decompose_exact_spin_half_1d(&system, &self.options)
    }
}

/// Borrowed two-dimensional exact spectrum rendering workflow.
#[derive(Clone, Debug)]
pub struct ExactSpinHalfSpectrum2DWorkflow<'a> {
    system: &'a SpinHalfSystem,
    options: ExactSpectrum2DOptions,
}

impl<'a> ExactSpinHalfSpectrum2DWorkflow<'a> {
    /// Creates a two-dimensional exact spectrum workflow.
    #[must_use]
    pub fn new(system: &'a SpinHalfSystem, transition_options: ExactSpinOptions) -> Self {
        Self {
            system,
            options: exact_spectrum_2d_options(transition_options),
        }
    }

    /// Replaces all two-dimensional rendering options.
    #[must_use]
    pub fn with_options(mut self, options: ExactSpectrum2DOptions) -> Self {
        self.options = options;
        self
    }

    /// Sets the output x-axis ppm range.
    #[must_use]
    pub fn with_x_ppm_range(mut self, from_ppm: f64, to_ppm: f64) -> Self {
        self.options.x_from_ppm = from_ppm;
        self.options.x_to_ppm = to_ppm;
        self
    }

    /// Sets the output y-axis ppm range.
    #[must_use]
    pub fn with_y_ppm_range(mut self, from_ppm: f64, to_ppm: f64) -> Self {
        self.options.y_from_ppm = from_ppm;
        self.options.y_to_ppm = to_ppm;
        self
    }

    /// Sets both output point counts.
    #[must_use]
    pub fn with_points(mut self, x_points: usize, y_points: usize) -> Self {
        self.options.x_points = x_points;
        self.options.y_points = y_points;
        self
    }

    /// Sets the integrated correlation volume.
    #[must_use]
    pub fn with_volume(mut self, volume: f64) -> Self {
        self.options.volume = volume;
        self
    }

    /// Sets the x-axis full width at half maximum in Hz.
    #[must_use]
    pub fn with_x_line_width_hz(mut self, line_width_hz: f64) -> Self {
        self.options.x_line_width_hz = line_width_hz;
        self
    }

    /// Sets the y-axis full width at half maximum in Hz.
    #[must_use]
    pub fn with_y_line_width_hz(mut self, line_width_hz: f64) -> Self {
        self.options.y_line_width_hz = line_width_hz;
        self
    }

    /// Sets the rendered line shape.
    #[must_use]
    pub fn with_line_shape(mut self, line_shape: LineShape) -> Self {
        self.options.line_shape = line_shape;
        self
    }

    /// Replaces exact transition generation options.
    #[must_use]
    pub fn with_transition_options(mut self, transition_options: ExactSpinOptions) -> Self {
        self.options.transition_options = transition_options;
        self
    }

    /// Sets the spectrometer frequency in MHz.
    #[must_use]
    pub fn with_spectrometer_mhz(mut self, spectrometer_mhz: f64) -> Self {
        self.options.transition_options.spectrometer_mhz = spectrometer_mhz;
        self
    }

    /// Adds one directed spin pair.
    #[must_use]
    pub fn with_spin_pair(mut self, x_spin: usize, y_spin: usize) -> Self {
        self.options
            .spin_pairs
            .push(ExactSpinPair::new(x_spin, y_spin));
        self
    }

    /// Replaces all directed spin pairs.
    #[must_use]
    pub fn with_spin_pairs(mut self, spin_pairs: impl IntoIterator<Item = ExactSpinPair>) -> Self {
        self.options.spin_pairs = spin_pairs.into_iter().collect();
        self
    }

    /// Uses scalar couplings from the spin system as directed spin pairs.
    #[must_use]
    pub fn without_spin_pairs(mut self) -> Self {
        self.options.spin_pairs.clear();
        self
    }

    /// Simulates the configured two-dimensional spectrum.
    ///
    /// # Errors
    ///
    /// Returns an error when the system, transition options, spin pairs, or
    /// rendering options are invalid.
    pub fn run(self) -> Result<Spectrum2D> {
        simulate_exact_spin_half_2d(self.system, &self.options)
    }

    /// Simulates the configured spectrum and per-correlation contributions.
    ///
    /// # Errors
    ///
    /// Returns an error when the system, transition options, spin pairs, or
    /// rendering options are invalid.
    pub fn decompose(self) -> Result<ExactSpectrumDecomposition2D> {
        decompose_exact_spin_half_2d(self.system, &self.options)
    }
}

/// Owned two-dimensional exact spectrum workflow from a fallible spin-system input.
#[derive(Debug)]
pub struct ExactSpinHalfSpectrum2DResultWorkflow {
    system: Result<SpinHalfSystem>,
    options: ExactSpectrum2DOptions,
}

impl ExactSpinHalfSpectrum2DResultWorkflow {
    /// Creates a two-dimensional exact spectrum workflow from an existing result.
    #[must_use]
    pub fn new(system: Result<SpinHalfSystem>, transition_options: ExactSpinOptions) -> Self {
        Self {
            system,
            options: exact_spectrum_2d_options(transition_options),
        }
    }

    /// Replaces all two-dimensional rendering options.
    #[must_use]
    pub fn with_options(mut self, options: ExactSpectrum2DOptions) -> Self {
        self.options = options;
        self
    }

    /// Sets the output x-axis ppm range.
    #[must_use]
    pub fn with_x_ppm_range(mut self, from_ppm: f64, to_ppm: f64) -> Self {
        self.options.x_from_ppm = from_ppm;
        self.options.x_to_ppm = to_ppm;
        self
    }

    /// Sets the output y-axis ppm range.
    #[must_use]
    pub fn with_y_ppm_range(mut self, from_ppm: f64, to_ppm: f64) -> Self {
        self.options.y_from_ppm = from_ppm;
        self.options.y_to_ppm = to_ppm;
        self
    }

    /// Sets both output point counts.
    #[must_use]
    pub fn with_points(mut self, x_points: usize, y_points: usize) -> Self {
        self.options.x_points = x_points;
        self.options.y_points = y_points;
        self
    }

    /// Sets the integrated correlation volume.
    #[must_use]
    pub fn with_volume(mut self, volume: f64) -> Self {
        self.options.volume = volume;
        self
    }

    /// Sets the x-axis full width at half maximum in Hz.
    #[must_use]
    pub fn with_x_line_width_hz(mut self, line_width_hz: f64) -> Self {
        self.options.x_line_width_hz = line_width_hz;
        self
    }

    /// Sets the y-axis full width at half maximum in Hz.
    #[must_use]
    pub fn with_y_line_width_hz(mut self, line_width_hz: f64) -> Self {
        self.options.y_line_width_hz = line_width_hz;
        self
    }

    /// Sets the rendered line shape.
    #[must_use]
    pub fn with_line_shape(mut self, line_shape: LineShape) -> Self {
        self.options.line_shape = line_shape;
        self
    }

    /// Replaces exact transition generation options.
    #[must_use]
    pub fn with_transition_options(mut self, transition_options: ExactSpinOptions) -> Self {
        self.options.transition_options = transition_options;
        self
    }

    /// Sets the spectrometer frequency in MHz.
    #[must_use]
    pub fn with_spectrometer_mhz(mut self, spectrometer_mhz: f64) -> Self {
        self.options.transition_options.spectrometer_mhz = spectrometer_mhz;
        self
    }

    /// Adds one directed spin pair.
    #[must_use]
    pub fn with_spin_pair(mut self, x_spin: usize, y_spin: usize) -> Self {
        self.options
            .spin_pairs
            .push(ExactSpinPair::new(x_spin, y_spin));
        self
    }

    /// Replaces all directed spin pairs.
    #[must_use]
    pub fn with_spin_pairs(mut self, spin_pairs: impl IntoIterator<Item = ExactSpinPair>) -> Self {
        self.options.spin_pairs = spin_pairs.into_iter().collect();
        self
    }

    /// Uses scalar couplings from the spin system as directed spin pairs.
    #[must_use]
    pub fn without_spin_pairs(mut self) -> Self {
        self.options.spin_pairs.clear();
        self
    }

    /// Simulates the configured two-dimensional spectrum.
    ///
    /// # Errors
    ///
    /// Returns the initial system error, or an error from rendering.
    pub fn run(self) -> Result<Spectrum2D> {
        let system = self.system?;
        simulate_exact_spin_half_2d(&system, &self.options)
    }

    /// Simulates the configured spectrum and per-correlation contributions.
    ///
    /// # Errors
    ///
    /// Returns the initial system error, or an error from rendering.
    pub fn decompose(self) -> Result<ExactSpectrumDecomposition2D> {
        let system = self.system?;
        decompose_exact_spin_half_2d(&system, &self.options)
    }
}

fn exact_spectrum_1d_options(transition_options: ExactSpinOptions) -> ExactSpectrumOptions {
    ExactSpectrumOptions {
        transition_options,
        ..ExactSpectrumOptions::default()
    }
}

fn exact_spectrum_2d_options(transition_options: ExactSpinOptions) -> ExactSpectrum2DOptions {
    ExactSpectrum2DOptions {
        transition_options,
        ..ExactSpectrum2DOptions::default()
    }
}

#[cfg(test)]
mod tests;
