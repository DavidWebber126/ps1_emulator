pub struct Timer {
    pub counter: u16,
    pub mode: u16,
    pub target_value: u16,
    allow_irq: bool,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            counter: 0,
            mode: 0,
            target_value: 0xFF,
            allow_irq: true,
        }
    }

    // Tick timer once. Returns true if IRQ
    pub fn tick(&mut self) -> bool {
        self.counter = self.counter.wrapping_add(1);
        if self.reset_after_target() && self.counter == self.target_value.wrapping_add(1) {
            self.counter = 0;
        }

        if self.irq_at_max() && self.counter == 0xFFFF && self.allow_irq {
            if self.irq_repeat() {
                self.allow_irq = false;
            }
            if self.is_toggle_mode() {
                self.toggle_int();
            } else {
                self.mode &= 0xFBFF;
            }
            return true;
        }

        if self.irq_when_at_target() && self.counter == self.target_value && self.allow_irq {
            if self.irq_repeat() {
                self.allow_irq = false;
            }
            if self.is_toggle_mode() {
                self.toggle_int();
            } else {
                self.mode &= 0xFBFF;
            }
            return true;
        }

        if !self.irq_repeat() {
            self.allow_irq = true;
        }

        false
    }

    pub fn write_to_mode(&mut self, val: u16) {
        self.counter = 0;
        self.allow_irq = true;
        self.mode |= 0x400;
        self.mode = val & 0x3FF;
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

    fn is_toggle_mode(&self) -> bool {
        self.mode & 0x80 > 0
    }

    fn toggle_int(&mut self) {
        let test = self.mode & 0x400 > 0;
        if test {
            self.mode &= 0xFBFF;
        } else {
            self.mode |= 0x400;
        }
    }

    // If true, IRQ repeats. If false, then one-shot (IRQ once then not until next write)
    fn irq_repeat(&self) -> bool {
        self.mode & 0x40 > 0
    }
}
