use instant::Instant;

#[derive(Copy, Clone)]
pub struct FpsCounter {
    cur_second: Instant,
    cur_counter: usize,
    fps: usize,
}

impl Default for FpsCounter {
    fn default() -> Self {
        Self {
            cur_second: Instant::now(),
            cur_counter: 0,
            fps: 0,
        }
    }
}

impl FpsCounter {
    pub fn new_frame(&mut self, instant: Instant, is_rendering: bool) {
        let millis = instant.duration_since(self.cur_second).as_millis();
        if millis < 1000 {
            if is_rendering {
                self.cur_counter += 1;
            }
        } else {
            self.fps = self.cur_counter;
            self.cur_second = instant;
            self.cur_counter = 0;
        }
    }

    pub fn get_fps(&self) -> usize {
        self.fps
    }
}