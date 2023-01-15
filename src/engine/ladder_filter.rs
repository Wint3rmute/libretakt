//! Multimode resonant ladder filter emulation.
//!
//! Copied from https://github.com/RustAudio/vst-rs
//!
//! The MIT License (MIT)
//! Copyright (c) 2015 Marko Mijalkovic
//!
//! Permission is hereby granted, free of charge, to any person obtaining a copy
//! of this software and associated documentation files (the "Software"), to deal
//! in the Software without restriction, including without limitation the rights
//! to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
//! copies of the Software, and to permit persons to whom the Software is
//! furnished to do so, subject to the following conditions:
//!
//! The above copyright notice and this permission notice shall be included in all
//! copies or substantial portions of the Software.
//!
//! THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//! IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//! FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//! AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//! LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//! OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
//! SOFTWARE.

//! This zero-delay feedback filter is based on a 4-stage transistor ladder filter.
//! It follows the following equations:
//! x = input - tanh(self.res * self.vout[3])
//! vout[0] = self.params.g.get() * (tanh(x) - tanh(self.vout[0])) + self.s[0]
//! vout[1] = self.params.g.get() * (tanh(self.vout[0]) - tanh(self.vout[1])) + self.s[1]
//! vout[0] = self.params.g.get() * (tanh(self.vout[1]) - tanh(self.vout[2])) + self.s[2]
//! vout[0] = self.params.g.get() * (tanh(self.vout[2]) - tanh(self.vout[3])) + self.s[3]
//! since we can't easily solve a nonlinear equation,
//! Mystran's fixed-pivot method is used to approximate the tanh() parts.
//! Quality can be improved a lot by oversampling a bit.
//! Feedback is clipped independently of the input, so it doesn't disappear at high gains.

use std::f32::consts::PI;

// this is a 4-pole filter with resonance, which is why there's 4 states and vouts
#[derive(Clone)]
pub struct LadderFilter {
    // Store a handle to the plugin's parameter object.
    pub params: LadderParameters,
    // the output of the different filter stages
    vout: [f32; 4],
    // s is the "state" parameter. In an IIR it would be the last value from the filter
    // In this we find it by trapezoidal integration to avoid the unit delay
    s: [f32; 4],
}

impl LadderFilter {
    pub fn new() -> Self {
        Self {
            vout: [0f32; 4],
            s: [0f32; 4],
            params: LadderParameters::default(),
        }
    }

    pub fn process(&mut self, input_sample: f32) -> f32 {
        self.tick_pivotal(input_sample);
        // the poles parameter chooses which filter stage we take our output from.
        self.vout[self.params.poles]
    }
}

impl Default for LadderFilter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct LadderParameters {
    // the "cutoff" parameter. Determines how heavy filtering is
    cutoff: f32,
    g: f32,
    // needed to calculate cutoff.
    pub sample_rate: f32,
    // makes a peak at cutoff
    pub res: f32,
    // used to choose where we want our output to be
    poles: usize,
    // pole_value is just to be able to use get_parameter on poles
    pole_value: f32,
    // a drive parameter. Just used to increase the volume, which results in heavier distortion
    #[allow(dead_code)] // TODO: add a filter drive param
    drive: f32,
}

impl Default for LadderParameters {
    fn default() -> LadderParameters {
        LadderParameters {
            cutoff: 1000.0,
            res: 2.,
            poles: 3,
            pole_value: 1.,
            drive: 0.,
            // sample_rate: 44100.,
            sample_rate: 44100. * 2.0,
            g: 0.07135868,
        }
    }
}

// member methods for the struct
impl LadderFilter {
    // the state needs to be updated after each process. Found by trapezoidal integration
    fn update_state(&mut self) {
        self.s[0] = 2. * self.vout[0] - self.s[0];
        self.s[1] = 2. * self.vout[1] - self.s[1];
        self.s[2] = 2. * self.vout[2] - self.s[2];
        self.s[3] = 2. * self.vout[3] - self.s[3];
    }

    // performs a complete filter process (mystran's method)
    pub fn tick_pivotal(&mut self, input: f32) {
        // if self.params.drive > 0. {
        //     self.run_ladder_nonlinear(input * (self.params.drive + 0.7));
        // } else {
        //
        self.run_ladder_linear(input);
        // }
        self.update_state();
    }

    // nonlinear ladder filter function with distortion.
    fn run_ladder_nonlinear(&mut self, input: f32) {
        let mut a = [1f32; 5];
        let base = [input, self.s[0], self.s[1], self.s[2], self.s[3]];
        // a[n] is the fixed-pivot approximation for tanh()
        for n in 0..base.len() {
            if base[n] != 0. {
                a[n] = base[n].tanh() / base[n];
            } else {
                a[n] = 1.;
            }
        }
        // denominators of solutions of individual stages. Simplifies the math a bit
        let g0 = 1. / (1. + self.params.g * a[1]);
        let g1 = 1. / (1. + self.params.g * a[2]);
        let g2 = 1. / (1. + self.params.g * a[3]);
        let g3 = 1. / (1. + self.params.g * a[4]);
        //  these are just factored out of the feedback solution. Makes the math way easier to read
        let f3 = self.params.g * a[3] * g3;
        let f2 = self.params.g * a[2] * g2 * f3;
        let f1 = self.params.g * a[1] * g1 * f2;
        let f0 = self.params.g * g0 * f1;
        // outputs a 24db filter
        self.vout[3] = (f0 * input * a[0]
            + f1 * g0 * self.s[0]
            + f2 * g1 * self.s[1]
            + f3 * g2 * self.s[2]
            + g3 * self.s[3])
            / (f0 * self.params.res * a[3] + 1.);
        // since we know the feedback, we can solve the remaining outputs:
        self.vout[0] = g0
            * (self.params.g * a[1] * (input * a[0] - self.params.res * a[3] * self.vout[3])
                + self.s[0]);
        self.vout[1] = g1 * (self.params.g * a[2] * self.vout[0] + self.s[1]);
        self.vout[2] = g2 * (self.params.g * a[3] * self.vout[1] + self.s[2]);
    }

    // linear version without distortion
    pub fn run_ladder_linear(&mut self, input: f32) {
        // denominators of solutions of individual stages. Simplifies the math a bit
        let g0 = 1. / (1. + self.params.g);
        let g1 = self.params.g * g0 * g0;
        let g2 = self.params.g * g1 * g0;
        let g3 = self.params.g * g2 * g0;
        // outputs a 24db filter
        self.vout[3] = (g3 * self.params.g * input
            + g0 * self.s[3]
            + g1 * self.s[2]
            + g2 * self.s[1]
            + g3 * self.s[0])
            / (g3 * self.params.g * self.params.res + 1.);
        // since we know the feedback, we can solve the remaining outputs:
        self.vout[0] = g0 * (self.params.g * (input - self.params.res * self.vout[3]) + self.s[0]);
        self.vout[1] = g0 * (self.params.g * self.vout[0] + self.s[1]);
        self.vout[2] = g0 * (self.params.g * self.vout[1] + self.s[2]);
    }
}

impl LadderParameters {
    pub fn set_cutoff(&mut self, value: f32) {
        // cutoff formula gives us a natural feeling cutoff knob that spends more time in the low frequencies
        self.cutoff = 20000. * (1.8f32.powf(10. * value - 10.));
        // bilinear transformation for g gives us a very accurate cutoff
        self.g = (PI * self.cutoff / (self.sample_rate)).tan();
    }

    // returns the value used to set cutoff. for get_parameter function
    pub fn get_cutoff(&self) -> f32 {
        1. + 0.17012975 * (0.00005 * self.cutoff).ln()
    }

    pub fn set_poles(&mut self, value: f32) {
        self.pole_value = value;
        // self.poles
        //     .store(((value * 3.).round()) as usize, Ordering::Relaxed);
        self.poles = (value * 3.).round() as usize;
    }
}
