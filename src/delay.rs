use crate::dsp::Frame;

#[derive(Debug)]
pub struct DelayLine {
    buf_size: usize,
    samples: Vec<Frame>,
    write_ptr: usize,
    time: usize,
}

impl DelayLine {
    pub fn new(time: usize, buf_size: usize) -> Self {
        Self {
            buf_size,
            samples: vec![Frame::new(0.0); buf_size],
            write_ptr: 0,
            time,
        }
    }

    pub fn write(&mut self, sample: f32) {
        self.samples[self.write_ptr] = Frame::new(sample);
    }

    pub fn read(&self) -> f32 {
        let read_ptr_signed = self.write_ptr as isize - self.time as isize;
        let read_ptr = if read_ptr_signed < 0 {
            (read_ptr_signed + self.buf_size as isize) as usize
        } else {
            read_ptr_signed as usize
        };
        self.samples[read_ptr].mono()
    }

    pub fn advance(&mut self) {
        self.write_ptr = (self.write_ptr + 1) % self.buf_size;
    }
}
