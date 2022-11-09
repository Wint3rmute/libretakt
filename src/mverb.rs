pub struct MVerb {
    all_pass: AllPass<16>, //todo: remove
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
    // TODO
}
