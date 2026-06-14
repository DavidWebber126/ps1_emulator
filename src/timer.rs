pub struct Timer {
    id: u8,
    counter_mode: CounterMode,
    pub counter: u16,
    pub mode: u16,
    pub target_value: u16,
    allow_irq: bool,
    sync_mode: u8,
    sync_enabled: bool,
}

impl Timer {
    pub fn new(id: u8) -> Self {
        Self {
            id,
            counter_mode: CounterMode::SystemClock,
            counter: 0,
            mode: 0,
            target_value: 0xFF,
            allow_irq: true,
            sync_mode: 0,
            sync_enabled: false,
        }
    }

    // Tick timer once. Returns true if IRQ
    pub fn tick(&mut self, dotclocks: u16, hblanks: u16) -> bool {
        self.increment_counter(dotclocks, hblanks);

        if self.reset_after_target() && (self.counter == self.target_value.wrapping_add(1)) {
            self.counter = 0;
        }

        if self.irq_at_max() && (self.counter == 0xFFFF) && self.allow_irq {
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

        if self.irq_when_at_target() && (self.counter == self.target_value) && self.allow_irq {
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

    pub fn write_mode(&mut self, val: u16) {
        self.counter = 0;
        self.allow_irq = true;
        self.mode |= 0x400;
        self.mode = val & 0x3FF;
        self.sync_enabled = val & 1 > 0;

        match (val >> 8) & 0b11 {
            0 => self.counter_mode = CounterMode::SystemClock,
            1 => {
                if self.id == 0 {
                    self.counter_mode = CounterMode::Dotclock
                }
                if self.id == 1 {
                    self.counter_mode = CounterMode::Hblank
                }
                if self.id == 2 {
                    self.counter_mode = CounterMode::SystemClock
                }
            }
            2 => {
                if self.id == 0 || self.id == 1 {
                    self.counter_mode = CounterMode::SystemClock
                }
                if self.id == 2 {
                        self.counter_mode = CounterMode::SystemClockEighth
                }
            }
            3 => {
                if self.id == 0 {
                    self.counter_mode = CounterMode::Dotclock
                }
                if self.id == 1 {
                    self.counter_mode = CounterMode::Hblank
                }
                if self.id == 2 {
                    self.counter_mode = CounterMode::SystemClockEighth
                }
            }
            _ => panic!("Impossible")
        }
    }

    pub fn read_mode(&self) -> u16 {

        self.mode
    }

    fn increment_counter(&mut self, dotclocks: u16, hblanks: u16) {
        match self.counter_mode {
            CounterMode::SystemClock => {
                self.counter = self.counter.wrapping_add(1);
            }
            CounterMode::Dotclock => {
                self.counter = dotclocks;
            }
            CounterMode::Hblank => {
                self.counter = hblanks;
            }
            CounterMode::SystemClockEighth => {
                todo!()
            }
        }
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

#[derive(Debug)]
enum CounterMode {
    SystemClock,
    Dotclock,
    Hblank,
    SystemClockEighth,
}
