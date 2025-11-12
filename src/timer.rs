pub struct Timer {
    pub counter: u16,
    pub mode: u16,
    pub target_value: u16,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            counter: 0,
            mode: 0,
            target_value: 0xFF,
        }
    }

    // Tick timer once. Returns true if IRQ
    pub fn tick(&mut self) -> bool {
        self.counter = self.counter.wrapping_add(1);
        if self.reset_after_target() && self.counter > self.target_value {
            self.counter -= self.target_value;
        }

        if self.irq_at_max() && self.counter == 0xFFFF {
            return true;
        }

        if self.irq_when_at_target() && self.counter == self.target_value {
            return true;
        }

        false
    }

    // Setters and Getters
    fn reset_after_target(&self) -> bool {
        self.mode & 0x8 > 0
    }

    fn irq_when_at_target(&self) -> bool {
        self.mode & 0x10 > 0
    }

    fn irq_at_max(&self) -> bool {
        self.mode & 0x20 > 0
    }

    // If true, IRQ repeats. If false, then one-shot (IRQ once then not until next write)
    fn irq_repeat(&self) -> bool {
        self.mode & 0x40 > 0
    }
}
