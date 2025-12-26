#[derive(Default)]
pub struct Interrupt {
    pub stat: u32,
    pub mask: u32,
}

impl Interrupt {
    pub fn new() -> Self {
        Self { stat: 0, mask: 0 }
    }
}
