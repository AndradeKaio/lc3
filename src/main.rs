use std::fs;
use std::io::Read;
use std::io;
use std::io::Write;
use std::process;
use termios::*;

pub const MAX_SIZE: usize = (1 << 16) + 1;

pub enum TrapCode {
    Getc = 0x20,  /* get character from keyboard, not echoed onto the terminal */
    Out = 0x21,   /* output a character */
    Puts = 0x22,  /* output a word string */
    In = 0x23,    /* get character from keyboard, echoed onto the terminal */
    Putsp = 0x24, /* output a byte string */
    Halt = 0x25,  /* halt the program */
}

#[derive(Debug)]
pub enum OpCode {
    Br = 0,
    Add,
    Load,
    Store,
    Jsr, //jump register
    And,
    Ldr, //load register
    Str,
    Rti,
    Not,
    Ldi, //load indirect
    Sti, //store indirect
    Jmp, //jump
    Res,
    Lea, //load effective address
    Trap,
}

pub fn get_op_code(instr: u16) -> OpCode {
    match instr >> 12 {
        0 => OpCode::Br,
        1 => OpCode::Add,
        2 => OpCode::Load,
        3 => OpCode::Store,
        4 => OpCode::Jsr,
        5 => OpCode::And,
        6 => OpCode::Ldr,
        7 => OpCode::Str,
        8 => OpCode::Rti,
        9 => OpCode::Not,
        10 => OpCode::Ldi,
        11 => OpCode::Sti,
        12 => OpCode::Jmp,
        13 => OpCode::Res,
        14 => OpCode::Lea,
        15 => OpCode::Trap,
        _ => panic!("can't get ocode from unknown instruction {}", instr),
    }
}
pub enum Flag {
    Pos = 1 << 0,
    Zero = 1 << 1,
    Neg = 1 << 2,
}

pub struct Memory {
    slot: [u16; MAX_SIZE],
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            slot: [0; MAX_SIZE],
        }
    }

    pub fn read(&mut self, address: u16) -> u16 {
        if address == MemoryMappedRegisters::Kbsr as u16 {
            self.read_key_board();
        }
        self.slot[address as usize]
    }

    pub fn read_key_board(&mut self) {
        let mut buffer = [0; 1];
        std::io::stdin().read_exact(&mut buffer).unwrap();
        if buffer[0] != 0 {
            self.write(MemoryMappedRegisters::Kbsr as u16, 1 << 15);
            self.write(MemoryMappedRegisters::Kbdr as u16, buffer[0] as u16);
        } else {
            self.write(MemoryMappedRegisters::Kbsr as u16, 0);
        }
    }

    pub fn write(&mut self, address: u16, value: u16) {
        self.slot[address as usize] = value;
    }
}

pub struct Registers {
    pub r0: u16,
    pub r1: u16,
    pub r2: u16,
    pub r3: u16,
    pub r4: u16,
    pub r5: u16,
    pub r6: u16,
    pub r7: u16,
    pub pc: u16,
    pub cond: u16,
}

pub enum MemoryMappedRegisters {
    Kbsr = 0xFE00,
    Kbdr = 0xFE02,
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            r0: 0,
            r1: 0,
            r2: 0,
            r3: 0,
            r4: 0,
            r5: 0,
            r6: 0,
            r7: 0,
            pc: 0,
            cond: 0,
        }
    }

    pub fn get_reg_value(&self, reg: u16) -> u16 {
        match reg {
            0 => self.r0,
            1 => self.r1,
            2 => self.r2,
            3 => self.r3,
            4 => self.r4,
            5 => self.r5,
            6 => self.r6,
            7 => self.r7,
            8 => self.pc,
            9 => self.cond,
            _ => panic!("invalid register {}", reg),
        }
    }

    pub fn update_register(&mut self, reg: u16, value: u16) {
        match reg {
            0 => self.r0 = value,
            1 => self.r1 = value,
            2 => self.r2 = value,
            3 => self.r3 = value,
            4 => self.r4 = value,
            5 => self.r5 = value,
            6 => self.r6 = value,
            7 => self.r7 = value,
            8 => self.pc = value,
            9 => self.cond = value,
            _ => panic!("invalid register {}", reg),
        }
    }

    pub fn update_flag(&mut self, value: u16) {
        if self.get_reg_value(value) == 0 {
            self.cond = Flag::Zero as u16
        } else if self.get_reg_value(value) >> 15 != 0 {
            self.cond = Flag::Neg as u16
        } else {
            self.cond = Flag::Pos as u16
        }
    }
}

pub fn trap(trap_instr: u16, registers: &mut Registers, memory: &mut Memory) {
    match trap_instr & 0xFF {
        //TrapCode::Getc
        0x20 => {
            println!("Getc");
            let mut buffer = [0; 1];
            std::io::stdin().read_exact(&mut buffer).unwrap();
            registers.r0 = buffer[0] as u16;
        }
        //TrapCode::Out
        0x21 => {
            println!("OUt");
            let value = memory.read(registers.r0);
            print!("{}", (value as u8) as char);
        }
        //TrapCode::Puts
        0x22 => {
            // Puts
            let mut index = registers.r0;
            let mut c = memory.read(index);
            while c != 0x0000 {
                print!("{}", (c as u8) as char);
                index += 1;
                c = memory.read(index);
            }
            io::stdout().flush().expect("failed to flush");
        }
        //TrapCode::In
        0x23 => {
            print!("Enter a  character : ");
            io::stdout().flush().expect("failed to flush");
            let char = std::io::stdin()
                .bytes()
                .next()
                .and_then(|result| result.ok())
                .map(|byte| byte as u16)
                .unwrap();
            registers.r0 = char;
        }
        //TrapCode::Putsp
        0x24 => {
            println!("Putsp");
            let mut index = registers.r0;
            let mut c = memory.read(index);
            while c != 0x0000 {
                let c1 = ((c & 0xFF) as u8) as char;
                print!("{}", c1);
                let c2 = ((c >> 8) as u8) as char;
                if c2 != '\0' {
                    print!("{}", c2);
                }
                index += 1;
                c = memory.read(index);
            }
            io::stdout().flush().expect("failed to flush");
        }
        //TrapCode::Halt
        0x25 => {
            print!("halt");
            process::exit(1);
        }
        _ => panic!("unknown trap code"),
    }
}

pub fn sign_extend(mut value: u16, bit_count: usize) -> u16 {
    if (value >> (bit_count - 1)) & 1 != 0 {
        value |= 0xFFFF << bit_count;
    }
    value
}

pub fn store_r(instr: u16, registers: &mut Registers, memory: &mut Memory) {
    let r0: u16 = (instr >> 9) & 0x7;
    let r1: u16 = (instr >> 6) & 0x7;
    let offset: u16 = sign_extend(instr & 0x3F, 6);
    let value: u32 = registers.get_reg_value(r1) as u32 + offset as u32;
    memory.write(
        value as u16,
        registers.get_reg_value(r0),
    );
}

pub fn store_i(instr: u16, registers: &mut Registers, memory: &mut Memory) {
    let r0: u16 = (instr >> 9) & 0x7;
    let pc_offset: u16 = sign_extend(instr & 0x1FF, 9);
    let value = pc_offset as u32 + registers.pc as u32;
    let value = memory.read(value as u16);
    memory.write(value as u16, registers.get_reg_value(r0) as u16);
}

pub fn store(instr: u16, registers: &mut Registers, memory: &mut Memory) {
    let r0: u16 = (instr >> 9) & 0x7;
    let pc_offset: u16 = sign_extend(instr & 0x1FF, 9);
    let pc_value = registers.pc as u32 + pc_offset as u32;
    memory.write(pc_value as u16, registers.get_reg_value(r0) as u16);
}

pub fn load_e(instr: u16, registers: &mut Registers) {
    let r0: u16 = (instr >> 9) & 0x7;
    let pc_offset = sign_extend(instr & 0x1FF, 9);
    let value = registers.pc as u32 + pc_offset as u32;
    registers.update_register(r0, value as u16);
    registers.update_flag(r0);
}

pub fn load_r(instr: u16, registers: &mut Registers, memory: &mut Memory) {
    let r0 = (instr >> 9) & 0x7;
    let r1 = (instr >> 6) & 0x7;
    let offset = sign_extend(instr & 0x3F, 6);
    let r1_value: u32 = registers.get_reg_value(r1) as u32;
    let value = r1_value + offset as u32;
    registers.update_register(r0, memory.read(value as u16));
    registers.update_flag(r0);
}

pub fn load(instr: u16, registers: &mut Registers, memory: &mut Memory) {
    let r0: u16 = (instr >> 9) & 0x7;
    let pc_offset: u16 = sign_extend(instr & 0x1FF, 9);
    let value = registers.pc as u32 + pc_offset as u32;
    let value = memory.read(value as u16);
    registers.update_register(r0, value);
    registers.update_flag(r0);
}

pub fn jsr(instr: u16, registers: &mut Registers) {
    let long_flag: u16 = (instr >> 11) & 1;
    registers.r7 = registers.pc;

    if long_flag != 0 {
        let long_flag_offset = sign_extend(instr & 0x7FF, 11);
        let value = registers.pc as u32 + long_flag_offset as u32;
        registers.pc = value as u16;
    } else {
        let r1 = (instr >> 6) & 0x7;
        registers.pc = registers.get_reg_value(r1);
    }
}

pub fn jmp(instr: u16, registers: &mut Registers) {
    let r1: u16 = (instr >> 6) & 0x7;
    registers.pc = registers.get_reg_value(r1);
}

pub fn br(instr: u16, registers: &mut Registers) {
    let pc_offset: u16 = sign_extend(instr & 0x1FF, 9);
    let cond_flag: u16 = (instr >> 9) & 0x7;
    if (cond_flag & registers.cond) != 0 {
        registers.pc = registers.pc + pc_offset;
    }
}

pub fn not(instr: u16, registers: &mut Registers) {
    let r0: u16 = (instr >> 9) & 0x7;
    let r1: u16 = (instr >> 6) & 0x7;
    let value = registers.get_reg_value(r1);
    registers.update_register(r1, !value);
    registers.update_flag(r0);
}

pub fn and(instr: u16, registers: &mut Registers) {
    let r0: u16 = (instr >> 9) & 0x7;
    let r1: u16 = (instr >> 6) & 0x7;
    let imm5_flag: u16 = (instr >> 5) & 0x1;
    let r1_value = registers.get_reg_value(r1);

    let value = if imm5_flag != 0 {
        let imm5 = sign_extend(instr & 0x1F, 5);
        r1_value & imm5
    } else {
        let r2_address: u16 = instr & 0x7;
        r1_value & registers.get_reg_value(r2_address)
    };
    registers.update_register(r0, value as u16);
    registers.update_flag(r0);
}

pub fn ldi(instr: u16, registers: &mut Registers, memory: &mut Memory) {
    let r0: u16 = (instr >> 9) & 0x7;
    let pc_offset = sign_extend(instr & 0x1FF, 9);
    let value = registers.pc as u32 + pc_offset as u32;
    let tmp = memory.read(value as u16);
    let value = memory.read(tmp as u16);
    registers.update_register(r0, value);
    registers.update_flag(r0);
}

pub fn add(instr: u16, registers: &mut Registers) {
    let r0: u16 = (instr >> 9) & 0x7;
    let r1: u16 = (instr >> 6) & 0x7;
    let imm5_flag: u16 = (instr >> 5) & 0x1;
    let r1_value = registers.get_reg_value(r1);
    let value = if imm5_flag != 0 {
        let imm5 = sign_extend(instr & 0x1F, 5);
        r1_value as u32 + imm5 as u32
    } else {
        let r2: u16 = instr & 0x7;
        r1_value as u32 + registers.get_reg_value(r2) as u32
    };
    registers.update_register(r0, value as u16);
    registers.update_flag(r0);
}

fn read_image_file(file_path: &str, memory: &mut Memory) {
    let mut file = fs::File::open(file_path).unwrap();
    let file_size = file.metadata().unwrap().len();
    let mut buffer = [0; 2];
    let mut limit = file_size - 1;
    let mut instruction: u16;

    let mut n = file.read(&mut buffer[..]);
    let mut address = ((buffer[0] as u16) << 8) | buffer[1] as u16;

    loop {
        n = file.read(&mut buffer[..]);
        if limit <= 0 {
            break;
        }
        match n {
            Err(_) => break,
            Ok(n) => limit -= 1 as u64,
        }

        instruction = (((buffer[0] as u16) << 8) | buffer[1] as u16) as u16;
        memory.write(address, instruction);
        address += 1;
    }
}

fn main() {
    let stdin = 0;
    let termios = termios::Termios::from_fd(stdin).unwrap();
    let mut new_termios = termios.clone();
    new_termios.c_iflag &= IGNBRK | BRKINT | PARMRK | ISTRIP | INLCR | IGNCR | ICRNL | IXON;
    new_termios.c_lflag &= !(ICANON | ECHO); // no echo and canonical mode
    tcsetattr(stdin, TCSANOW, &mut new_termios).unwrap();


    const START: u16 = 0x3000;
    let mut memory = Memory::new();
    let mut registers = Registers::new();

    //initial setup
    registers.cond = Flag::Zero as u16;
    registers.pc = START;

    //read obj file
    read_image_file("/Users/kaio/Projects/rust/lc3/src/2048.obj", &mut memory);

    loop {

        let instr = memory.read(registers.pc);
        registers.pc += 1;

        let opcode = get_op_code(instr);

        match opcode {
            OpCode::Br => br(instr, &mut registers),
            OpCode::Add => add(instr, &mut registers),
            OpCode::Load => load(instr, &mut registers, &mut memory),
            OpCode::Store => store(instr, &mut registers, &mut memory),
            OpCode::Jsr => jsr(instr, &mut registers),
            OpCode::Ldr => load_r(instr, &mut registers, &mut memory),
            OpCode::Str => store_r(instr, &mut registers, &mut memory),
            OpCode::Not => not(instr, &mut registers),
            OpCode::Ldi => ldi(instr, &mut registers, &mut memory),
            OpCode::Sti => store_i(instr, &mut registers, &mut memory),
            OpCode::Jmp => jmp(instr, &mut registers),
            OpCode::Lea => load_e(instr, &mut registers),
            OpCode::Trap => trap(instr, &mut registers, &mut memory),
            _ => break,
        }
    }
    tcsetattr(stdin, TCSANOW, &termios).unwrap();
}

