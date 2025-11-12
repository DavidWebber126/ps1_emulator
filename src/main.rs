mod bus;
mod cop0;
mod cpu;
mod timer;

use cpu::Cpu;

fn main() {
    let mut cpu = Cpu::new();
    cpu.step_instruction();
}
