pub const CYCLES_PER_SCANLINE: u32 = 128;
pub const VISIBLE_CYCLES: u32 = 256 * CYCLES_PER_SCANLINE;
pub const VERTICAL_BLANK_CYCLES: u32 = 56 * CYCLES_PER_SCANLINE;
pub const FRAME_CYCLES: u32 = VISIBLE_CYCLES + VERTICAL_BLANK_CYCLES;
pub const FRAMEBUFFER_SIZE: usize = 12_288;

pub struct VideoController {
    framebuffer: Box<[u8; FRAMEBUFFER_SIZE]>,
    frame_index: u64,
    cycle_in_frame: u32,
    captured: bool,
}

impl Default for VideoController {
    fn default() -> Self {
        Self {
            framebuffer: Box::new([0; FRAMEBUFFER_SIZE]),
            frame_index: 0,
            cycle_in_frame: 0,
            captured: false,
        }
    }
}

impl VideoController {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn advance(&mut self, cycles: u32, video_memory: &[u8]) {
        debug_assert_eq!(video_memory.len(), FRAMEBUFFER_SIZE);
        self.cycle_in_frame = self.cycle_in_frame.wrapping_add(cycles);
        if self.cycle_in_frame >= VISIBLE_CYCLES && !self.captured {
            self.framebuffer.copy_from_slice(video_memory);
            self.frame_index = self.frame_index.wrapping_add(1);
            self.captured = true;
        }
        if self.cycle_in_frame >= FRAME_CYCLES {
            self.cycle_in_frame %= FRAME_CYCLES;
            self.captured = false;
        }
    }

    pub fn framebuffer(&self) -> &[u8] {
        &self.framebuffer[..]
    }

    pub fn frame_index(&self) -> u64 {
        self.frame_index
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn captures_only_after_the_visible_area_and_again_next_frame() {
        let mut video = VideoController::default();
        let mut memory = [0; FRAMEBUFFER_SIZE];
        memory[0] = 0x80;

        video.advance(VISIBLE_CYCLES - 1, &memory);
        assert_eq!(video.frame_index(), 0);
        assert_eq!(video.framebuffer()[0], 0);
        video.advance(1, &memory);
        assert_eq!(video.frame_index(), 1);
        assert_eq!(video.framebuffer()[0], 0x80);

        memory[0] = 0x01;
        video.advance(VERTICAL_BLANK_CYCLES, &memory);
        assert_eq!(video.frame_index(), 1);
        assert_eq!(video.framebuffer()[0], 0x80);
        video.advance(VISIBLE_CYCLES, &memory);
        assert_eq!(video.frame_index(), 2);
        assert_eq!(video.framebuffer()[0], 0x01);
    }

    #[test]
    fn reset_restores_the_initial_frame() {
        let mut video = VideoController::default();
        video.advance(VISIBLE_CYCLES, &[0xa5; FRAMEBUFFER_SIZE]);
        video.reset();
        assert_eq!(video.frame_index(), 0);
        assert!(video.framebuffer().iter().all(|value| *value == 0));
    }
}
