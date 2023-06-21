use crate::cpu::{IOpCode, OpCode};

pub(crate) fn disassemble_instruction(code: OpCode, pc: usize) {
    let first_nib = code.nib1();

    print!("{:04x} {:02x} {:02x}     ", pc, code.high, code.low);
    match first_nib {
        0x00 => match code.low {
            0xe0 => print!("CLS"),
            0xee => print!("RTS"),
            _ => print!(""),
        },
        0x01 => {
            let address = code.nib_l3();
            print!("JUMP #{:02x}", address);
        }
        0x02 => {
            let address = code.nib_l3();
            print!("CALL #{:02x}", address);
        }
        0x03 => {
            let register = code.nib2();
            print!("SKIP.EQ V{:01x}, #{:02x}", register, code.low);
        }
        0x04 => {
            let register = code.nib2();
            print!("SKIP.NE V{:01x}, #{:02x}", register, code.low);
        }
        0x05 => {
            let reg_x = code.nib2();
            let reg_y = code.nib3();
            print!("SKIP.EQ V{:01x}, V{:01x}", reg_x, reg_y);
        }
        0x06 => {
            let register = code.nib2();
            print!("MVI V{:01x}, {:02x}", register, code.low);
        }
        0x07 => {
            let register = code.nib2();
            print!("ADD. V{:01x}, {:02x}", register, code.low);
        }
        0x08 => {
            let reg_x = code.nib2();
            let reg_y = code.nib3();
            match code.nib4() {
                0 => print!("MOV V{:01x}, V{:01x}", reg_x, reg_y),
                1 => print!("OR V{:01x}, V{:01x}", reg_x, reg_y),
                2 => print!("AND V{:01x}, V{:01x}", reg_x, reg_y),
                3 => print!("XOR V{:01x}, V{:01x}", reg_x, reg_y),
                4 => print!("ADD. V{:01x}, V{:01x}", reg_x, reg_y),
                5 => print!("SUB. V{:01x}, V{:01x}", reg_x, reg_y),
                6 => print!("SHR. V{:01x}", reg_x),
                7 => print!("SUBB. V{:01x}, V{:01x}", reg_x, reg_y),
                8 => print!("SHL. V{:01x}", reg_x),
                _ => print!(""),
            }
        }
        0x09 => {
            let reg_x = code.nib2();
            let reg_y = code.nib3();
            print!("SKIP.NE V{:01x}, V{:01x}", reg_x, reg_y);
        }
        0x0a => {
            let address = code.nib_l3();
            print!("MVI I {:01x}", address);
        }
        0x0b => {
            let address = code.nib_l3();
            print!("JUMP #{:02x}(V0)", address);
        }
        0x0c => {
            let register = code.nib2();
            print!("li V{:01x}, rand() & {:01x}", register, code.low);
        }
        0x0d => {
            let reg_x = code.nib2();
            let reg_y = code.nib3();
            print!("SPRITE V{:01x}, V{:01x}, {:01x}", reg_x, reg_y, code.nib4());
        }
        0x0e => {
            let register = code.high & 0x0f;
            match code.low {
                0x9e => print!("SKIP.KEY V{:01x}", register),
                0xa1 => print!("SKIP.NOKEY V{:01x}", register),
                _ => print!(""),
            }
        }
        0x0f => {
            let register = code.high & 0x0f;
            match code.low {
                0x07 => print!("MOV V{:01x}, DELAY", register),
                0x0a => print!("WAITKEY V{:01x}", register),
                0x15 => print!("MOV DELAY, V{:01x}", register),
                0x18 => print!("MOV SOUND, V{:01x}", register),
                0x1e => print!("ADD I, V{:01x}", register),
                0x29 => print!("SPRITECHAR V{:01x}", register),
                0x33 => print!("MOVBCD V{:01x}", register),
                0x55 => print!("MOVM (I), V0-V{:01x}", register),
                0x65 => print!("MOVM V0-V{:01x}, (I)", register),
                _ => print!(""),
            }
        }
        _ => print!("impossible first nib"),
    }
}
