use std::cell::Cell;

pub enum MVerbParam {
    DampingFrequency,
    Density,
    BandwidthFrequency,
    Decay,
    PreDelay,
    Size,
    Gain,
    Mix,
    EarlyMix,
    NumParams,
}

pub struct MVerb<'a> {
    all_pass: [AllPass<96000>; 4],
    all_pass_four_tap: [StaticAllPassFourTap<96000>; 4],
    bandwidth_filter: [StateVariable<'a, 4>; 2],
    damping: [StateVariable<'a, 4>; 2],
    predelay: StaticDelayLine<96000>,
    static_delay_line: [StaticDelayLineFourTap<96000>; 4],
    early_reflections_delay_line: [StaticDelayLineEightTap<96000>; 2],
    sample_rate: f32,
    damping_frequency: f32,
    density1: f32,
    density2: f32,
    bandwidth_frequency: f32,
    pre_delay_time: f32,
    decay: f32,
    gain: f32,
    mix: f32,
    early_mix: f32,
    size: f32,
    mix_smooth: f32,
    early_late_smooth: f32,
    bandwidth_smooth: f32,
    damping_smooth: f32,
    predelay_smooth: f32,
    size_smooth: f32,
    density_smooth: f32,
    decay_smooth: f32,
    previous_left_tank: f32,
    previous_right_tank: f32,
    control_rate: usize,
    control_rate_counter: usize,
}

impl<'a> Default for MVerb<'a> {
    fn default() -> Self {
        let sample_rate = 441000.0;

        let mut result = Self {
            all_pass: Default::default(),
            all_pass_four_tap: Default::default(),
            bandwidth_filter: Default::default(),
            early_reflections_delay_line: Default::default(),
            predelay: Default::default(),
            damping: Default::default(),
            static_delay_line: Default::default(),

            damping_frequency: 0.9,
            bandwidth_frequency: 0.9,
            sample_rate: 44100.0,
            decay: 0.5,
            gain: 1.0,
            mix: 1.0,
            size: 1.0,
            early_mix: 1.0,
            previous_left_tank: 0.0,
            previous_right_tank: 0.0,
            pre_delay_time: 100.0 * (sample_rate / 1000.0),
            mix_smooth: 0.0,
            early_late_smooth: 0.0,
            bandwidth_smooth: 0.0,
            damping_smooth: 0.0,
            predelay_smooth: 0.0,
            size_smooth: 0.0,
            decay_smooth: 0.0,
            density_smooth: 0.0,
            control_rate: (sample_rate / 1000.0) as usize,
            control_rate_counter: 0,

            density1: 0.0,
            density2: 0.0,
        };
        result.reset();

        result
    }
}

impl<'a> MVerb<'a> {
    fn reset(&mut self) {
        self.control_rate_counter = 0;

        self.bandwidth_filter[0].set_sample_rate(self.sample_rate);
        self.bandwidth_filter[1].set_sample_rate(self.sample_rate);
        self.bandwidth_filter[0].reset();
        self.bandwidth_filter[1].reset();
        self.damping[0].set_sample_rate(self.sample_rate);
        self.damping[1].set_sample_rate(self.sample_rate);
        self.damping[0].reset();
        self.damping[1].reset();
        self.predelay.clear();
        self.predelay.set_length(self.pre_delay_time as usize);
        self.all_pass[0].clear();
        self.all_pass[1].clear();
        self.all_pass[2].clear();
        self.all_pass[3].clear();

        self.all_pass[0].set_length((0.0048 * self.sample_rate) as usize);
        self.all_pass[1].set_length((0.0036 * self.sample_rate) as usize);
        self.all_pass[2].set_length((0.0127 * self.sample_rate) as usize);
        self.all_pass[3].set_length((0.0093 * self.sample_rate) as usize);
        self.all_pass[0].set_feedback(0.75);
        self.all_pass[1].set_feedback(0.75);
        self.all_pass[2].set_feedback(0.625);
        self.all_pass[3].set_feedback(0.625);
        self.all_pass_four_tap[0].clear();
        self.all_pass_four_tap[1].clear();
        self.all_pass_four_tap[2].clear();
        self.all_pass_four_tap[3].clear();
        self.all_pass_four_tap[0].set_length((0.020 * self.sample_rate * self.size) as usize);
        self.all_pass_four_tap[1].set_length((0.060 * self.sample_rate * self.size) as usize);
        self.all_pass_four_tap[2].set_length((0.030 * self.sample_rate * self.size) as usize);
        self.all_pass_four_tap[3].set_length((0.089 * self.sample_rate * self.size) as usize);
        self.all_pass_four_tap[0].set_feedback(self.density1);
        self.all_pass_four_tap[1].set_feedback(self.density2);
        self.all_pass_four_tap[2].set_feedback(self.density1);
        self.all_pass_four_tap[3].set_feedback(self.density2);
        self.all_pass_four_tap[0].set_index(0, 0, 0, 0);
        self.all_pass_four_tap[1].set_index(
            0,
            (0.006 * self.sample_rate * self.size) as usize,
            (0.041 * self.sample_rate * self.size) as usize,
            0,
        );
        self.all_pass_four_tap[2].set_index(0, 0, 0, 0);
        self.all_pass_four_tap[3].set_index(
            0,
            (0.031 * self.sample_rate * self.size) as usize,
            (0.011 * self.sample_rate * self.size) as usize,
            0,
        );
        self.static_delay_line[0].clear();
        self.static_delay_line[1].clear();
        self.static_delay_line[2].clear();
        self.static_delay_line[3].clear();
        self.static_delay_line[0].set_length((0.15 * self.sample_rate * self.size) as usize);
        self.static_delay_line[1].set_length((0.12 * self.sample_rate * self.size) as usize);
        self.static_delay_line[2].set_length((0.14 * self.sample_rate * self.size) as usize);
        self.static_delay_line[3].set_length((0.11 * self.sample_rate * self.size) as usize);
    }
}

pub struct AllPass<const max_length: usize> {
    buffer: [f32; max_length],
    index: usize,
    length: usize,
    feedback: f32,
}

impl<const max_length: usize> Default for AllPass<max_length> {
    fn default() -> Self {
        AllPass {
            feedback: 0.5,
            buffer: [0.0; max_length],
            index: 0,
            length: max_length - 1,
        }
    }
}

impl<const max_length: usize> AllPass<max_length> {
    fn operator(&mut self, input: f32) -> f32 {
        let output = 0.0;

        let bufout = self.buffer[self.index];
        let temp = input * -self.feedback;
        self.buffer[self.index] = input + ((bufout + temp) * self.feedback);

        self.index += 1;
        if self.index >= self.length {
            self.index = 0;
        }

        output
    }

    fn set_length(&mut self, mut length: usize) {
        if length >= max_length {
            length = max_length;
        }

        // if length < 0 {
        //     length = 0
        // }

        self.length = length;
    }

    fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback;
    }

    fn clear(&mut self) {
        self.buffer.fill(0.0);
        self.index = 0;
    }

    fn get_length(&self) -> usize {
        self.length
    }
}

struct StaticAllPassFourTap<const max_length: usize> {
    buffer: [f32; max_length],
    index1: usize,
    index2: usize,
    index3: usize,
    index4: usize,
    length: usize,
    feedback: f32,
}

impl<const max_length: usize> Default for StaticAllPassFourTap<max_length> {
    fn default() -> Self {
        Self {
            buffer: [0.0; max_length],
            index1: 0,
            index2: 0,
            index3: 0,
            index4: 0,
            feedback: 0.5,
            length: max_length - 1,
        }
    }
}

impl<const max_length: usize> StaticAllPassFourTap<max_length> {
    fn operator(&mut self, input: f32) -> f32 {
        let bufout = self.buffer[self.index1];
        let temp = input * -self.feedback;
        let output = bufout + temp;
        self.buffer[self.index1] = input + ((bufout + temp) * self.feedback);

        self.index1 += 1;
        self.index2 += 1;
        self.index3 += 1;
        self.index4 += 1;

        if self.index1 >= self.length {
            self.index1 = 0;
        }
        if self.index2 >= self.length {
            self.index2 = 0;
        }
        if self.index3 >= self.length {
            self.index3 = 0;
        }
        if self.index4 >= self.length {
            self.index4 = 0;
        }

        output
    }

    fn set_index(&mut self, index1: usize, index2: usize, index3: usize, index4: usize) {
        self.index1 = index1;
        self.index2 = index2;
        self.index3 = index3;
        self.index4 = index4;
    }

    fn get_index(&self, index: usize) -> f32 {
        match index {
            1 => self.buffer[self.index2],
            2 => self.buffer[self.index3],
            3 => self.buffer[self.index4],
            _ => self.buffer[self.index1],
        }
    }

    fn set_length(&mut self, mut length: usize) {
        if length > max_length {
            length = max_length;
        }

        self.length = length;
    }

    fn clear(&mut self) {
        self.buffer.fill(0.0);
        self.index1 = 0;
        self.index2 = 0;
        self.index3 = 0;
        self.index4 = 0;
    }

    fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback;
    }

    fn get_length(&self) -> usize {
        self.length
    }
}

struct StaticDelayLine<const max_length: usize> {
    buffer: [f32; max_length],
    index: usize,
    length: usize,
    feedback: f32,
}

impl<const max_length: usize> Default for StaticDelayLine<max_length> {
    fn default() -> Self {
        Self {
            buffer: [0.0; max_length],
            index: 0,
            feedback: 0.5,
            length: max_length - 1,
        }
    }
}

impl<const max_length: usize> StaticDelayLine<max_length> {
    fn operator(&mut self, input: f32) -> f32 {
        let output = self.buffer[self.index];

        self.buffer[self.index] = input;
        self.index += 1;

        if self.index >= self.length {
            self.index = 0;
        }

        output
    }

    fn set_length(&mut self, mut length: usize) {
        if length > max_length {
            length = max_length;
        }

        self.length = length;
    }

    fn clear(&mut self) {
        self.buffer.fill(0.0);
        self.index = 0;
    }

    fn get_length(&self) -> usize {
        self.length
    }
}

struct StaticDelayLineFourTap<const max_length: usize> {
    buffer: [f32; max_length],
    index1: usize,
    index2: usize,
    index3: usize,
    index4: usize,

    length: usize,
    feedback: f32,
}

impl<const max_length: usize> Default for StaticDelayLineFourTap<max_length> {
    fn default() -> Self {
        Self {
            buffer: [0.0; max_length],
            index1: 0,
            index2: 0,
            index3: 0,
            index4: 0,
            length: max_length - 1,
            feedback: 0.0,
        }
    }
}

impl<const max_length: usize> StaticDelayLineFourTap<max_length> {
    fn operator(&mut self, input: f32) -> f32 {
        let output = self.buffer[self.index1];
        self.buffer[self.index1] = input;

        self.index1 += 1;
        self.index2 += 1;
        self.index3 += 1;
        self.index4 += 1;

        if self.index1 >= self.length {
            self.index1 = 0;
        }

        if self.index2 >= self.length {
            self.index2 = 0;
        }

        if self.index3 >= self.length {
            self.index3 = 0;
        }

        if self.index4 >= self.length {
            self.index4 = 0;
        }

        output
    }

    fn set_index(&self, index: usize) -> f32 {
        match index {
            1 => self.buffer[self.index2],
            2 => self.buffer[self.index3],
            3 => self.buffer[self.index4],
            _ => self.buffer[self.index1],
        }
    }

    fn set_length(&mut self, mut length: usize) {
        if length > max_length {
            length = max_length;
        }

        self.length = length;
    }

    fn clear(&mut self) {
        self.buffer.fill(0.0);
        self.index1 = 0;
        self.index2 = 0;
        self.index3 = 0;
        self.index4 = 0;
    }

    fn get_length(&self) -> usize {
        self.length
    }
}

struct StaticDelayLineEightTap<const max_length: usize> {
    buffer: [f32; max_length],
    index1: usize,
    index2: usize,
    index3: usize,
    index4: usize,
    index5: usize,
    index6: usize,
    index7: usize,
    index8: usize,
    length: usize,
    feedback: f32,
}

impl<const max_length: usize> Default for StaticDelayLineEightTap<max_length> {
    fn default() -> Self {
        Self {
            buffer: [0.0; max_length],
            index1: 0,
            index2: 0,
            index3: 0,
            index4: 0,
            index5: 0,
            index6: 0,
            index7: 0,
            index8: 0,
            length: max_length - 1,
            feedback: 0.0,
        }
    }
}

impl<const max_length: usize> StaticDelayLineEightTap<max_length> {
    fn operator(&mut self, input: f32) -> f32 {
        let output: f32 = self.buffer[self.index1];
        self.buffer[self.index1] = input;
        self.index1 += 1;

        if self.index1 >= self.length {
            self.index1 = 0;
        }

        if self.index2 >= self.length {
            self.index2 = 0;
        }

        if self.index3 >= self.length {
            self.index3 = 0;
        }

        if self.index4 >= self.length {
            self.index4 = 0;
        }

        output
    }

    fn set_index(
        &mut self,
        index1: usize,
        index2: usize,
        index3: usize,
        index4: usize,
        index5: usize,
        index6: usize,
        index7: usize,
        index8: usize,
    ) {
        self.index1 = index1;
        self.index2 = index2;
        self.index3 = index3;
        self.index4 = index4;
        self.index5 = index5;
        self.index6 = index6;
        self.index7 = index7;
        self.index8 = index8;
    }

    fn get_index(&self, index: usize) -> f32 {
        match index {
            0 => self.buffer[self.index1],
            1 => self.buffer[self.index2],
            2 => self.buffer[self.index3],
            3 => self.buffer[self.index4],
            4 => self.buffer[self.index5],
            5 => self.buffer[self.index6],
            6 => self.buffer[self.index7],
            7 => self.buffer[self.index8],
            _ => self.buffer[self.index1],
        }
    }

    fn set_length(&mut self, mut length: usize) {
        if length >= max_length {
            length = max_length;
        }

        // if length < 0 {
        //     length = 0
        // }

        self.length = length;
    }

    fn clear(&mut self) {
        self.buffer.fill(0.0);
        self.index1 = 0;
        self.index2 = 0;
        self.index3 = 0;
        self.index4 = 0;
        self.index5 = 0;
        self.index6 = 0;
        self.index7 = 0;
    }

    fn get_length(&self) -> usize {
        self.length
    }
}

enum FilterType {
    LowPass,
    HighPass,
    BandPass,
    Notch,
    FilterTypeCount,
}

pub struct StateVariable<'a, const over_sample_count: usize> {
    sample_rate: f32,
    frequency: f32,
    q: f32,
    f: f32,
    low: f32,
    high: f32,
    band: f32,
    notch: f32,
    out: &'a f32,
    // out TODO: defined as T *out; in src
}

impl<'a, const over_sample_count: usize> Default for StateVariable<'a, over_sample_count> {
    fn default() -> Self {
        let mut result = Self {
            sample_rate: 44100.0,
            frequency: 1000.0,
            q: 0.0,
            f: 0.0,
            low: 0.0,
            high: 0.0,
            band: 0.0,
            notch: 0.0,
            out: &0.0,
        };

        // result.set_type(FilterType::LowPass);
        result
    }
}

impl<'a, const over_sample_count: usize> StateVariable<'a, over_sample_count> {
    fn operator(&mut self, input: f32) {
        for _ in 0..over_sample_count {
            self.low += self.f * self.band + 1e-25;
            self.high = input - self.low - self.q * self.band;
            self.band += self.f * self.high;
            self.notch = self.low + self.high;
        }
    }

    fn reset(&mut self) {
        self.low = 0.0;
        self.high = 0.0;
        self.band = 0.0;
        self.notch = 0.0;
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate * over_sample_count as f32;
        self.update_coefficient();
    }

    fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
        self.update_coefficient();
    }

    fn set_resonance(&mut self, resonance: f32) {
        self.q = 2.0 - 2.0 * resonance;
    }

    fn set_type(&'a mut self, new_type: FilterType) {
        match new_type {
            FilterType::LowPass => {
                self.out = &self.low;
            }
            FilterType::HighPass => {
                self.out = &self.low;
            }

            FilterType::BandPass => {
                self.out = &self.band;
            }

            FilterType::Notch => {
                self.out = &self.notch;
            }

            _ => {
                self.out = &self.low;
            }
        };
    }

    fn update_coefficient(&mut self) {
        self.f = 2. * (3.141592654 * self.frequency / self.sample_rate).sin();
    }
}
