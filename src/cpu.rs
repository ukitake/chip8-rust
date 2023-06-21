use crate::{
    disassembler::disassemble_instruction,
    platform::CpuContext,
    rom::{self, Rom}, keyboard::char_to_index,
};
use crossbeam_channel::{Receiver, TryRecvError};
use rand::Rng;
use std::{
    io::Result,
    thread::sleep,
    time::{Duration},
};

pub(crate) struct OpCode {
    pub high: u8,
    pub low: u8,
}

pub(crate) trait IOpCode {
    fn combine(&self) -> u16;

    fn nib1(&self) -> u8;
    fn nib2(&self) -> u8;
    fn nib3(&self) -> u8;
    fn nib4(&self) -> u8;
    fn nib_l3(&self) -> u16;

    fn destructure(&self) -> (u8, u8, u8, u8, u16);

    fn execute(&self, program: &mut Program, context: &CpuContext);
}

trait Chip8 {
    fn CLS(&mut self);
    fn RTS(&mut self);
    fn JUMP(&mut self, address: u16);
    fn CALL(&mut self, address: u16);
    fn SKIPEQI(&mut self, reg: u8, nn: u8);
    fn SKIPNEI(&mut self, reg: u8, nn: u8);
    fn SKIPEQ(&mut self, reg1: u8, reg2: u8);
    fn MVI(&mut self, reg1: u8, nn: u8);
    fn ADDI(&mut self, reg1: u8, nn: u8);
    fn MV(&mut self, reg1: u8, reg2: u8);
    fn OR(&mut self, reg1: u8, reg2: u8);
    fn AND(&mut self, reg1: u8, reg2: u8);
    fn XOR(&mut self, reg1: u8, reg2: u8);
    fn ADD(&mut self, reg1: u8, reg2: u8);
    fn SUB(&mut self, reg1: u8, reg2: u8);
    fn SHR(&mut self, reg1: u8);
    fn SUB2(&mut self, reg1: u8, reg2: u8);
    fn SHL(&mut self, reg1: u8);
    fn SKIPNE(&mut self, reg1: u8, reg2: u8);
    fn MVII(&mut self, nn: u16);
    fn JUMPV0(&mut self, address: u16);
    fn RAND(&mut self, reg: u8, nn: u8);
    fn SPRITE(&mut self, reg1: u8, reg2: u8, height: u8);
    fn SKIPKEY(&mut self, reg: u8);
    fn SKIPNOKEY(&mut self, reg: u8);
    fn MVDELAY(&mut self, reg: u8);
    fn MVKEY(&mut self, reg: u8, receiver: &Receiver<char>);
    fn DELAYMV(&mut self, reg: u8);
    fn SOUNDMV(&mut self, reg: u8);
    fn ADDVI(&mut self, reg: u8);
    fn SPRITECHAR(&mut self, reg: u8);
    fn MOVBCD(&mut self, reg: u8);
    fn MOVM(&mut self, reg: u8);
    fn MOVMI(&mut self, reg: u8);
}

impl IOpCode for OpCode {
    fn combine(&self) -> u16 {
        return (u16::from(self.high) << 8) | u16::from(self.low);
    }
    fn nib1(&self) -> u8 {
        return self.high >> 4;
    }
    fn nib2(&self) -> u8 {
        return self.high & 0x0f;
    }
    fn nib3(&self) -> u8 {
        return self.low >> 4;
    }
    fn nib4(&self) -> u8 {
        return self.low & 0x0f;
    }
    fn nib_l3(&self) -> u16 {
        return (u16::from(self.high & 0x0f) << 8) | u16::from(self.low);
    }

    fn destructure(&self) -> (u8, u8, u8, u8, u16) {
        return (
            self.nib1(),
            self.nib2(),
            self.nib3(),
            self.nib4(),
            self.nib_l3(),
        );
    }

    fn execute(&self, program: &mut Program, context: &CpuContext) {
        let (n1, n2, n3, n4, l3) = self.destructure();

        match n1 {
            0 => match self.low {
                0xe0 => program.CLS(),
                0xee => program.RTS(),
                _ => (),
            },
            1 => program.JUMP(l3),
            2 => program.CALL(l3),
            3 => program.SKIPEQI(n2, self.low),
            4 => program.SKIPNEI(n2, self.low),
            5 => program.SKIPEQ(n2, n3),
            6 => program.MVI(n2, self.low),
            7 => program.ADDI(n2, self.low),
            8 => match n4 {
                0 => program.MV(n2, n3),
                1 => program.OR(n2, n3),
                2 => program.AND(n2, n3),
                3 => program.XOR(n2, n3),
                4 => program.ADD(n2, n3),
                5 => program.SUB(n2, n3),
                6 => program.SHR(n2),
                7 => program.SUB2(n2, n3),
                0x0E => program.SHL(n2),
                _ => (),
            },
            9 => program.SKIPNE(n2, n3),
            0x0a => program.MVII(l3),
            0x0b => program.JUMPV0(l3),
            0x0c => program.RAND(n2, self.low),
            0x0d => program.SPRITE(n2, n3, n4),
            0x0e => match self.low {
                0x9e => program.SKIPKEY(n2),
                0xa1 => program.SKIPNOKEY(n2),
                _ => (),
            },
            0x0f => match self.low {
                0x07 => program.MVDELAY(n2),
                0x0a => {
                    // flush the screen contents to the Platform right before we block waiting for a key
                    program.cpu.flush_screen(context);
                    program.MVKEY(n2, &context.single_key);
                }
                0x15 => program.DELAYMV(n2),
                0x18 => program.SOUNDMV(n2),
                0x1e => program.ADDVI(n2),
                0x29 => program.SPRITECHAR(n2),
                0x33 => program.MOVBCD(n2),
                0x55 => program.MOVM(n2),
                0x65 => program.MOVMI(n2),
                _ => (),
            },
            _ => (),
        }
    }
}

pub(crate) struct Cpu {
    ram: [u8; 4096],
    gp_reg: [u8; 16],
    i: u16,
    delay: u8,
    sound: u8,
    keystate: [u8; 16],
    screen: [[u8; 32]; 64],
    screen_dirty: bool,
}

trait ICpu {
    fn get_reg(&self, reg: u8) -> u8;
    fn peek_stack(&self, sp: u16) -> u16;
    fn set_stack(&mut self, value: u16, sp: u16);
    fn set_keystate(&mut self, state: &[u8; 16]);
    fn flush_screen(&mut self, context: &CpuContext);
}

impl ICpu for Cpu {
    fn get_reg(&self, reg: u8) -> u8 {
        return self.gp_reg[usize::from(reg)];
    }

    fn peek_stack(&self, sp: u16) -> u16 {
        return (u16::from(self.ram[usize::from(sp)]) << 8)
            | (u16::from(self.ram[usize::from(sp) + 1]));
    }

    fn set_stack(&mut self, value: u16, sp: u16) {
        let offset = usize::from(sp);
        self.ram[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
    }

    fn set_keystate(&mut self, state: &[u8; 16]) {
        self.keystate.copy_from_slice(state);
    }

    fn flush_screen(&mut self, context: &CpuContext) {
        if self.screen_dirty {
            match context.display.send(self.screen) {
                Ok(_) => (),
                Err(_) => (),
            }
            self.screen_dirty = false;
        }
    }
}

impl Default for Cpu {
    fn default() -> Self {
        const HEX_DIGIT_SPRITES: [u8; 80] = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0x10, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80, // 3
        ];
        let mut cpu = Cpu {
            ram: [0u8; 4096],
            gp_reg: [0u8; 16],
            i: 0,
            delay: 0,
            sound: 0,
            keystate: [0u8; 16],
            screen: [[0u8; 32]; 64], // 64 x 32 2D array of bytes, 1 for each screen pixel
            screen_dirty: false,
        };
        cpu.ram[0x50..0x50 + 80].copy_from_slice(&HEX_DIGIT_SPRITES);
        return cpu;
    }
}

impl Chip8 for Program {
    fn CLS(&mut self) {
        for x in 0..64 {
            for y in 0..32 {
                self.cpu.screen[x][y] = 0;
            }
        }
    }

    fn RTS(&mut self) {
        let return_addr = self.cpu.peek_stack(self.sp); // self.stack[usize::from(self.sp)];
        self.pc = return_addr;
        self.sp -= 2;
    }

    fn JUMP(&mut self, address: u16) {
        self.pc = address;
    }

    fn CALL(&mut self, address: u16) {
        self.sp += 2;
        self.cpu.set_stack(self.pc, self.sp);
        self.pc = address;
    }

    fn SKIPEQI(&mut self, reg: u8, nn: u8) {
        if self.cpu.get_reg(reg) == nn {
            self.pc += 2;
        }
    }

    fn SKIPNEI(&mut self, reg: u8, nn: u8) {
        if self.cpu.get_reg(reg) != nn {
            self.pc += 2;
        }
    }

    fn SKIPEQ(&mut self, reg1: u8, reg2: u8) {
        if self.cpu.get_reg(reg1) == self.cpu.get_reg(reg2) {
            self.pc += 2;
        }
    }

    fn MVI(&mut self, reg1: u8, nn: u8) {
        self.cpu.gp_reg[usize::from(reg1)] = nn;
    }

    fn ADDI(&mut self, reg1: u8, nn: u8) {
        self.cpu.gp_reg[usize::from(reg1)] = u8::wrapping_add(self.cpu.get_reg(reg1), nn);
    }

    fn MV(&mut self, reg1: u8, reg2: u8) {
        self.cpu.gp_reg[usize::from(reg1)] = self.cpu.gp_reg[usize::from(reg2)];
    }

    fn OR(&mut self, reg1: u8, reg2: u8) {
        self.cpu.gp_reg[usize::from(reg1)] |= self.cpu.gp_reg[usize::from(reg2)];
    }

    fn AND(&mut self, reg1: u8, reg2: u8) {
        self.cpu.gp_reg[usize::from(reg1)] &= self.cpu.gp_reg[usize::from(reg2)];
    }

    fn XOR(&mut self, reg1: u8, reg2: u8) {
        self.cpu.gp_reg[usize::from(reg1)] ^= self.cpu.gp_reg[usize::from(reg2)];
    }

    fn ADD(&mut self, reg1: u8, reg2: u8) {
        let res = u16::from(self.cpu.get_reg(reg1)) + u16::from(self.cpu.get_reg(reg2));
        self.cpu.gp_reg[usize::from(reg1)] = (res & 0x00FF) as u8;
        if res > 255 {
            self.cpu.gp_reg[0x0F] = 1;
        } else {
            self.cpu.gp_reg[0x0F] = 0;
        }
    }

    fn SUB(&mut self, reg1: u8, reg2: u8) {
        let r1 = self.cpu.get_reg(reg1) as i16;
        let r2 = self.cpu.get_reg(reg2) as i16;
        self.cpu.gp_reg[usize::from(reg1)] = (r1 - r2) as u8;
        if r1 > r2 {
            self.cpu.gp_reg[0x0F] = 1;
        } else {
            self.cpu.gp_reg[0x0F] = 0;
        }
    }

    fn SHR(&mut self, reg1: u8) {
        let val = self.cpu.get_reg(reg1);
        self.cpu.gp_reg[usize::from(reg1)] = val / 2;
        self.cpu.gp_reg[0x0F] = val & 0x01;
    }

    fn SUB2(&mut self, reg1: u8, reg2: u8) {
        let r1 = self.cpu.get_reg(reg1) as i16;
        let r2 = self.cpu.get_reg(reg2) as i16;
        self.cpu.gp_reg[usize::from(reg1)] = (r2 - r1) as u8;
        if r2 > r1 {
            self.cpu.gp_reg[0x0F] = 1;
        } else {
            self.cpu.gp_reg[0x0F] = 0;
        }
    }

    fn SHL(&mut self, reg1: u8) {
        let val = self.cpu.get_reg(reg1);
        self.cpu.gp_reg[usize::from(reg1)] = ((val as u16) << 1) as u8;
        self.cpu.gp_reg[0x0F] = val >> 7;
    }

    fn SKIPNE(&mut self, reg1: u8, reg2: u8) {
        if self.cpu.get_reg(reg1) != self.cpu.get_reg(reg2) {
            self.pc += 2;
        }
    }

    fn MVII(&mut self, nn: u16) {
        self.cpu.i = nn;
    }

    fn JUMPV0(&mut self, address: u16) {
        self.pc = address + u16::from(self.cpu.get_reg(0));
    }

    fn RAND(&mut self, reg: u8, nn: u8) {
        let random = rand::thread_rng().gen_range(0..255);
        self.cpu.gp_reg[usize::from(reg)] = u8::from(random) & nn;
    }

    fn SPRITE(&mut self, reg1: u8, reg2: u8, height: u8) {
        let x = self.cpu.get_reg(reg1) & 63;
        let y = self.cpu.get_reg(reg2) & 31;
        let sprite = &self.cpu.ram[self.cpu.i as usize..self.cpu.i as usize + height as usize];
        self.cpu.gp_reg[0x0F] = 0;

        for j in y..(y + height) {
            let bit_line = sprite[(j - y) as usize];
            for i in x..(x + 8) {
                let bit_idx = 7 - (i - x); // start at MSB
                let bit_mask = (1 as u8) << bit_idx;
                if i < 64 && j < 32 {
                    let old = self.cpu.screen[i as usize][j as usize];
                    let new = bit_line & bit_mask;
                    if new > 0 && old > 0 {
                        self.cpu.screen[i as usize][j as usize] = 0;
                        // set VF to 1 if a pixel went from set to unset
                        self.cpu.gp_reg[0x0F] = 1;
                    } else if new > 0 && old == 0 {
                        self.cpu.screen[i as usize][j as usize] = 1;
                    }
                    self.cpu.screen_dirty = true;
                }
            }
        }
    }

    fn SKIPKEY(&mut self, reg: u8) {
        if self.cpu.keystate[usize::from(self.cpu.get_reg(reg))] != 0 {
            self.pc += 2;
        }
    }

    fn SKIPNOKEY(&mut self, reg: u8) {
        if self.cpu.keystate[usize::from(self.cpu.get_reg(reg))] == 0 {
            self.pc += 2;
        }
    }

    fn MVDELAY(&mut self, reg: u8) {
        self.cpu.gp_reg[reg as usize] = self.cpu.delay;
    }

    fn MVKEY(&mut self, reg: u8, receiver: &Receiver<char>) {
        let c = receiver.recv();
        self.cpu.gp_reg[reg as usize] = char_to_index(c.unwrap()) as u8;
        self.cpu.delay = 0;
        self.cpu.sound = 0;
    }

    fn DELAYMV(&mut self, reg: u8) {
        self.cpu.delay = self.cpu.get_reg(reg);
    }

    fn SOUNDMV(&mut self, reg: u8) {
        self.cpu.sound = self.cpu.get_reg(reg);
    }

    fn ADDVI(&mut self, reg: u8) {
        self.cpu.i = self.cpu.i + u16::from(self.cpu.get_reg(reg));
    }

    fn SPRITECHAR(&mut self, reg: u8) {
        // the sprites for 0-9,a-f start at RAM[0x50] and are 5 bytes each
        self.cpu.i = 0x50 + reg as u16 * 5;
    }

    fn MOVBCD(&mut self, reg: u8) {
        let mut value = self.cpu.get_reg(reg);
        let ones = value % 10;
        value /= 10;
        let tens = value % 10;
        value /= 10;
        let hundreds = value % 10;
        self.cpu.ram[usize::from(self.cpu.i)] = hundreds;
        self.cpu.ram[usize::from(self.cpu.i) + 1] = tens;
        self.cpu.ram[usize::from(self.cpu.i) + 2] = ones;
    }

    fn MOVM(&mut self, reg: u8) {
        for i in 0..(reg + 1) {
            self.cpu.ram[usize::from(self.cpu.i) + usize::from(i)] = self.cpu.get_reg(i);
        }
    }

    fn MOVMI(&mut self, reg: u8) {
        for i in 0..(reg + 1) {
            self.cpu.gp_reg[usize::from(i)] =
                self.cpu.ram[usize::from(self.cpu.i) + usize::from(i)];
        }
    }
}

pub(crate) trait Runnable {
    fn run(&mut self, context: &CpuContext);
    fn disassemble(&mut self);
}

pub(crate) struct Program {
    pub cpu: Cpu,
    pub rom: Rom,
    pub frequency: f32,
    pc: u16,
    sp: u16,
}

pub(crate) fn init_program(file_name: &str) -> Result<Program> {
    let rom = rom::read(file_name)?;
    let mut cpu = Cpu::default();

    // load the ROM into RAM at address 0x200 which is where programs are supposed to start
    cpu.ram[0x200..0x200 + rom.bytes.len()].copy_from_slice(&rom.bytes);

    return Ok(Program {
        rom,
        cpu,
        frequency: 2000.0, // 2kHz
        pc: 0x200,
        sp: 0,
    });
}

impl Runnable for Program {
    fn run(&mut self, context: &CpuContext) {
        let rom_length = self.rom.bytes.len();
        let loop_duration = Duration::new(0, 1_000_000_000u32 / 60);

        while (self.pc as usize) < (0x200 + rom_length) {
            let instructions_per_loop = self.frequency * loop_duration.as_secs_f32();
            for _ in 0..instructions_per_loop as u32 {
                if (self.pc as usize) < 0x200 + rom_length {
                    // decode the current opcode.
                    // each is 2 bytes in big endian order
                    let opcode = OpCode {
                        high: self.cpu.ram[self.pc as usize],
                        low: self.cpu.ram[self.pc as usize + 1],
                    };

                    // immediately increment the program counter
                    self.pc += 2;

                    // see if there is a keyboard state sent from the Platform in the Channel
                    match context.keyboard.try_recv() {
                        Ok(state) => self.cpu.set_keystate(&state),
                        Err(TryRecvError::Empty) => (),
                        Err(TryRecvError::Disconnected) => (),
                    }

                    // execute the opcode
                    opcode.execute(self, context);
                } else {
                    break;
                }
            }

            // decrement the delay timer at 60Hz
            if self.cpu.delay > 0 {
                self.cpu.delay -= 1;
            }

            // decrement the sound timer at 60Hz
            if self.cpu.sound > 0 {
                match context.sound.try_send(true) {
                    Ok(_) => (),
                    Err(_) => (),
                }
                self.cpu.sound -= 1;
            }

            // send the screen pixels to the Platform if necessary
            self.cpu.flush_screen(context);

            // attempt to run this loop at 60Hz
            sleep(loop_duration);
        }
    }

    fn disassemble(&mut self) {
        while usize::from(self.pc) < (0x200 + self.rom.bytes.len()) {
            let code = OpCode {
                high: self.cpu.ram[usize::from(self.pc)],
                low: self.cpu.ram[usize::from(self.pc) + 1],
            };

            self.pc += 2;

            disassemble_instruction(code, usize::from(self.pc));
            print!("\n");
        }
    }
}
