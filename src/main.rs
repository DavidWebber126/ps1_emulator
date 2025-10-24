mod bus;
mod cpu;
mod cop0;

use cpu::Cpu;

fn main() {
    let mut cpu = Cpu::new();
    cpu.step_instruction();
}
