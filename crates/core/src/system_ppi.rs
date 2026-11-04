use crate::keyboard::{Key, Keyboard};

pub(crate) struct SystemPpi {
    keyboard: Keyboard,
    port_a: u8,
    port_b: u8,
    port_c: u8,
    port_a_input: bool,
    port_b_input: bool,
    port_c_high_input: bool,
    port_c_low_input: bool,
}

impl Default for SystemPpi {
    fn default() -> Self {
        Self {
            keyboard: Keyboard::default(),
            port_a: 0,
            port_b: 0,
            port_c: 0,
            port_a_input: true,
            port_b_input: true,
            port_c_high_input: true,
            port_c_low_input: true,
        }
    }
}

impl SystemPpi {
    pub(crate) fn reset(&mut self) {
        *self = Self::default();
    }

    pub(crate) fn set_key(&mut self, key: Key, pressed: bool) {
        self.keyboard.set_key(key, pressed);
    }

    fn row_mask(&self) -> u16 {
        let low = if self.port_a_input { 0xff } else { self.port_a };
        let high = if self.port_c_low_input {
            0x0f
        } else {
            self.port_c & 0x0f
        };
        u16::from(low) | (u16::from(high) << 8)
    }

    fn column_mask(&self) -> u8 {
        if self.port_b_input {
            0x3f
        } else {
            (self.port_b & 0xfc) >> 2
        }
    }

    pub(crate) fn read(&self, register: u8) -> u8 {
        match register & 3 {
            0 => {
                if self.port_a_input {
                    self.keyboard.rows(self.column_mask()) as u8
                } else {
                    self.port_a
                }
            }
            1 => {
                if self.port_b_input {
                    (self.keyboard.columns(self.row_mask()) << 2)
                        | if self.keyboard.shift_pressed() { 0 } else { 2 }
                        | 1
                } else {
                    self.port_b
                }
            }
            2 => {
                let mut value = self.port_c;
                if self.port_c_low_input {
                    value = (value & 0xf0)
                        | ((self.keyboard.rows(self.column_mask()) >> 8) as u8 & 0x0f);
                }
                if self.port_c_high_input {
                    value &= 0x0f;
                }
                value
            }
            _ => 0xff,
        }
    }

    pub(crate) fn write(&mut self, register: u8, value: u8) {
        match register & 3 {
            0 => {
                if !self.port_a_input {
                    self.port_a = value;
                }
            }
            1 => {
                if !self.port_b_input {
                    self.port_b = value;
                }
            }
            2 => {
                if !self.port_c_high_input {
                    self.port_c = (self.port_c & 0x0f) | (value & 0xf0);
                }
                if !self.port_c_low_input {
                    self.port_c = (self.port_c & 0xf0) | (value & 0x0f);
                }
            }
            _ => self.write_control(value),
        }
    }

    fn write_control(&mut self, value: u8) {
        if value & 0x80 == 0 {
            let bit = (value >> 1) & 7;
            if (bit < 4 && !self.port_c_low_input) || (bit >= 4 && !self.port_c_high_input) {
                self.port_c = (self.port_c & !(1 << bit)) | ((value & 1) << bit);
            }
            return;
        }

        self.port_a = 0;
        self.port_b = 0;
        self.port_c = 0;
        if value & 0x40 == 0 {
            self.port_a_input = value & 0x10 != 0;
            self.port_b_input = value & 0x02 != 0;
            self.port_c_high_input = value & 0x08 != 0;
            self.port_c_low_input = value & 0x01 != 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scans_columns_then_rows_with_active_low_signals() {
        let mut ppi = SystemPpi::default();
        ppi.set_key(Key::A, true);
        ppi.set_key(Key::Q, true);
        ppi.set_key(Key::Shift, true);

        ppi.write(3, 0x82);
        ppi.write(0, 0);
        ppi.write(2, 0);
        assert_eq!(ppi.read(1), 0b1110_0101);

        ppi.write(3, 0x91);
        ppi.write(1, 0);
        assert_eq!(ppi.read(0), 0xff);
        assert_eq!(ppi.read(2) & 0x0f, 0b0110);

        ppi.set_key(Key::A, false);
        ppi.set_key(Key::Q, false);
        ppi.set_key(Key::Shift, false);
        assert_eq!(ppi.read(0), 0xff);
        assert_eq!(ppi.read(2) & 0x0f, 0x0f);
        ppi.write(3, 0x82);
        ppi.write(0, 0);
        ppi.write(2, 0);
        assert_eq!(ppi.read(1), 0xff);
    }

    #[test]
    fn output_ports_return_their_latches_and_control_reads_ff() {
        let mut ppi = SystemPpi::default();
        ppi.write(3, 0x80);
        ppi.write(0, 0x5a);
        ppi.write(1, 0xa5);
        ppi.write(2, 0x3c);
        assert_eq!(ppi.read(0), 0x5a);
        assert_eq!(ppi.read(1), 0xa5);
        assert_eq!(ppi.read(2), 0x3c);
        assert_eq!(ppi.read(3), 0xff);
    }
}
