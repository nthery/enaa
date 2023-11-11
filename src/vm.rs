//! Virtual machine

use anyhow::{anyhow, Context};

/// All supported bytecodes.
///
/// Some bytecodes have an operand which is the unsigned byte following the
/// opcode in the code segment.  An operand is either a (conditional) jump
/// absolute address (offset in bytecode sequence) or an immediate integer.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    /// Push on stack ASCII code of next character in input buffer or push 0 on
    /// end of input.
    ///
    /// IN -> X
    /// [...] --> [... X]
    In = 0,

    /// Pop topmost stack element, consider it is an ASCII code and copy it into
    /// the output buffer.
    ///
    /// [... X] --> [...]
    /// X --> OUT
    Out = 1,

    /// Duplicate topmost stack element.
    ///
    /// [... X] --> [... X X]
    Dup = 2,

    /// Pop two topmost stack elements and push back their sum.
    ///
    /// [... X Y] --> [... X+Y]
    Add = 3,

    /// Pop two topmost stack elements and push back their difference.
    ///
    /// [... X Y] --> [... X-Y]
    Sub = 4,

    /// Pop topmost stack element and jump if non-zero.
    ///
    /// [... X] --> [...]
    Bne = 5,

    /// Pop two topmost stack elements and jump if second topmost one is less
    /// than first one.
    ///
    /// [... X Y] --> [...]
    Blt = 6,

    /// Stop the VM.
    Exit = 7,

    /// Push byte following this opcode onto stack.
    ///
    /// [...] --> [... N]
    Push = 8,

    /// Jump to absolute address stored in byte following this opcode.
    ///
    /// [...] --> [...]
    Jmp = 9,

    /// Pop two topmost stack elements and jump if second topmost is equal to
    /// first one.
    ///
    /// [... X Y] --> [...]
    Beq = 10,

    /// Push content of auxiliary register onto stack.
    ///
    /// [...] --> [... AUX]
    Pusha = 11,

    /// Pop stack topmost element into auxiliary register.
    ///
    /// [... N] --> [...]
    /// N --> AUX
    Popa = 12,

    /// Pop two topmost stack elements and jump if second topmost is greater
    /// than first one.
    ///
    /// [... X Y] --> [...]
    Bgt = 13,

    /// Pop two topmost stack elements and jump if second topmost is less than
    /// or equal to first one.
    ///
    /// [... X Y] --> [...]
    Ble = 14,
}

impl TryFrom<u8> for Opcode {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Opcode::In),
            1 => Ok(Opcode::Out),
            2 => Ok(Opcode::Dup),
            3 => Ok(Opcode::Add),
            4 => Ok(Opcode::Sub),
            5 => Ok(Opcode::Bne),
            6 => Ok(Opcode::Blt),
            7 => Ok(Opcode::Exit),
            8 => Ok(Opcode::Push),
            9 => Ok(Opcode::Jmp),
            10 => Ok(Opcode::Beq),
            11 => Ok(Opcode::Pusha),
            12 => Ok(Opcode::Popa),
            13 => Ok(Opcode::Bgt),
            14 => Ok(Opcode::Ble),
            _ => Err(anyhow!("invalid opcode {}", value)),
        }
    }
}

/// Virtual machine state.
///
/// The VM is a stack machine that manipulates 32-bit unsigned integers.
///
/// The VM has:
/// - a code segment storing bytecodes to execute;
/// - a data stack used for computation and temporary storage;
/// - an auxiliary register;
/// - an input buffer containing a sequence of ASCII characters;
/// - an output buffer containing a sequence of ASCII characters;
/// - a program counter register indexing into the code segment.
struct Vm<'a> {
    program: &'a [u8],
    input_chars: std::str::Chars<'a>,
    output: String,
    pc: usize,
    stack: Vec<u32>,
    aux: u32,
}

impl<'a> Vm<'a> {
    /// Initialize VM.
    fn new(program: &'a [u8], input: &'a str) -> Vm<'a> {
        Vm {
            program,
            input_chars: input.chars(),
            output: String::new(),
            pc: 0,
            stack: Vec::with_capacity(16),
            aux: 0,
        }
    }

    /// Interpret VM.
    fn run(&mut self) -> anyhow::Result<String> {
        loop {
            let opcode = self.program[self.pc];
            match Opcode::try_from(opcode)? {
                Opcode::Exit => break,
                Opcode::In => {
                    let i = self.input_chars.next().map_or(0, |ch| ch as u32);
                    self.push(i);
                    self.pc += 1;
                }
                Opcode::Out => {
                    let ch = char::from_u32(self.pop()?).context("converting code point")?;
                    self.output.push(ch);
                    self.pc += 1;
                }
                Opcode::Jmp => {
                    self.pc = self.program[self.pc + 1] as usize;
                }
                Opcode::Dup => {
                    self.push(*self.stack.last().context("duplicating stack")?);
                    self.pc += 1;
                }
                Opcode::Bne => {
                    let top = self.pop()?;
                    if top != 0 {
                        self.pc = self.program[self.pc + 1] as usize;
                    } else {
                        self.pc += 2;
                    }
                }
                Opcode::Bgt => {
                    self.branch_if(|l, r| l > r)?;
                }
                Opcode::Blt => {
                    self.branch_if(|l, r| l < r)?;
                }
                Opcode::Ble => {
                    self.branch_if(|l, r| l <= r)?;
                }
                Opcode::Pusha => {
                    self.push(self.aux);
                    self.pc += 1;
                }
                Opcode::Push => {
                    self.push(self.program[self.pc + 1] as u32);
                    self.pc += 2;
                }
                Opcode::Popa => {
                    self.aux = self.pop()?;
                    self.pc += 1;
                }
                Opcode::Add => {
                    let rhs = self.pop()?;
                    let lhs = self.pop()?;
                    self.push(lhs + rhs);
                    self.pc += 1;
                }
                Opcode::Sub => {
                    let rhs = self.pop()?;
                    let lhs = self.pop()?;
                    self.push(lhs - rhs);
                    self.pc += 1;
                }
                _ => todo!(),
            }
        }
        Ok(self.output.clone())
    }

    fn push(&mut self, x: u32) {
        self.stack.push(x)
    }

    fn pop(&mut self) -> anyhow::Result<u32> {
        self.stack.pop().context("pop")
    }

    fn branch_if<Cmp: FnOnce(u32, u32) -> bool>(&mut self, cmp: Cmp) -> anyhow::Result<()> {
        let rhs = self.pop()?;
        let lhs = self.pop()?;
        if cmp(lhs, rhs) {
            self.pc = self.program[self.pc + 1] as usize;
        } else {
            self.pc += 2;
        }
        Ok(())
    }
}

/// Execute specified program on specified input and return generated output.
pub fn run(program: &[u8], input: &str) -> anyhow::Result<String> {
    debug_assert!(!program.is_empty());
    let mut vm = Vm::new(program, input);
    vm.run()
}
