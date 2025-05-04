use std::io::{self, Read, Write, BufReader, Stdin};
use std::fs::File;
use std::env::args;
use std::process::exit;

/// Sign‑extend a u32 `value` whose low `bits` bits are the real data.
fn sign_extend(value: u32, bits: u8) -> i32 {
    let shift = 32 - bits as i32;
    ((value << shift) as i32) >> shift
}

/// Pop a 4‑byte little‑endian i32 from the stack and advance SP upward.
fn pop_i32(stack: &[u8; 4096], sp: &mut usize) -> i32 {
    let raw = &stack[*sp..*sp + 4];
    let v = i32::from_le_bytes(raw.try_into().unwrap());
    *sp = (*sp + 4).min(4096);
    v
}

/// Push a 4‑byte little‑endian i32 onto the stack, moving SP downward.
fn push(stack: &mut [u8; 4096], sp: &mut usize, v: i32) {
    let bytes = v.to_le_bytes();
    *sp = sp.saturating_sub(4);
    stack[*sp..*sp + 4].copy_from_slice(&bytes);
}

fn main() {
    let mut stack: [u8; 4096] = [0; 4096];
    let mut sp: usize = 4096;
    let mut pc: isize = 0;
    let mut buf: [u8; 4] = [0; 4];
    let mut instr: u32;
    let mut opcode: u8;

    let a: Vec<String> = args().collect(); 

    let inp: File = File::open(&a[1]).expect("Unable to find file");
    let mut reader: BufReader<File> = BufReader::new(inp);

    reader.read(&mut buf).expect("Failed to read file");

    if buf != [0xde, 0xad, 0xbe, 0xef] {
        println!("File did not contain magic bytes");
        return;
    }

    buf.iter().for_each(|x| println!("{:02x}", x));

    reader.read(&mut stack).expect("reading failure");

    loop {
        instr = (stack[((pc*4)+3) as usize] as u32) << 24 |
                (stack[((pc*4)+2) as usize] as u32) << 16 |
                (stack[((pc*4)+1) as usize] as u32) << 8 |
                (stack[(pc*4) as usize] as u32);

        stack[(pc*4) as usize..((pc*4)+3) as usize].iter().for_each(|x| println!("{:02x}: {}", x, *x as char));
        println!("{:032b}", instr);
        if pc > 32 {break;}

        opcode = (instr >> 28) as u8;

        match opcode {
            0 => {
                println!("0");
                let subcode = ((instr << 4) >> 28) as u8;

                match subcode {
                    0 => {
                        println!("exit");
                        exit((instr & 0b1111) as i32);
                    }
                    1 => {
                        println!("swap");
                        let from_offset: usize = ((instr << 8) >> 20) as usize;
                        let to_offset: usize = ((instr << 20) >> 20) as usize;
                        let from: [u8; 4] = stack[sp+from_offset..sp+from_offset+4]
                                            .try_into().expect("Swap from out of bounds");
                        let to: [u8; 4] = stack[sp+to_offset..sp+to_offset+4]
                                            .try_into().expect("Swap to out of bounds");
                        stack[sp+from_offset..sp+from_offset+4].copy_from_slice(&to);
                        stack[sp+to_offset..sp+to_offset+4].copy_from_slice(&from);
                    }
                    2 => {
                        println!("nop");
                    }
                    4 => {
                        println!("input");
                        let mut inp: String = String::new();
                        let inp_type: String;
                        let inp_val: String;
                        let inp_bytes: [u8; 4];

                        io::stdin().read_line(&mut inp).unwrap();
                        inp = inp.trim().to_uppercase();

                        inp_type = inp.chars().take(2).collect();

                        println!("{}", inp.to_uppercase());

                        if inp_type == "0X" {
                            println!("bytes");
                            inp_val = inp.chars().skip(2).collect();
                            inp_bytes = i32::from_str_radix(&inp_val, 16).expect("Input hex failed").to_le_bytes();
                            inp_bytes.iter().for_each(|x: &u8| println!("{:08b}", x));
                        } else if inp_type == "0B" {
                            println!("bits");
                            inp_val = inp.chars().skip(2).collect();
                            inp_bytes = i32::from_str_radix(&inp_val, 2).expect("Input bits failed").to_le_bytes();
                            inp_bytes.iter().for_each(|x: &u8| println!("{:08b}", x));
                        } else {
                            println!("base 10");
                            inp_bytes = inp.parse::<i32>().expect("Input base 10 failed").to_le_bytes();
                            inp_bytes.iter().for_each(|x: &u8| println!("{:08b}", x));
                        }
                    
                        sp -= 4;
                        stack[sp..sp+4].copy_from_slice(&inp_bytes);

                    }
                    5 => {
                        println!("stinput");
                        let mut max_char: usize = ((instr << 8) >> 8) as usize;
                        let mut inp: String = String::new();

                        if max_char == 0 {
                            max_char = 0xffffff;
                        }
                        println!("{}", max_char);

                        io::stdin().read_line(&mut inp).unwrap();

                        inp = inp.trim().to_string();

                        if inp.len() == 0 {
                            inp = "0".to_string();
                        }

                        inp = inp.chars().take(max_char).collect();

                        println!("{}", inp.len());

                        sp -= inp.len();
                        stack[sp..sp+inp.len()].copy_from_slice(&inp.chars().map(|x: char| x as u8).collect::<Vec<u8>>());

                    }
                    15 => {
                        println!("Debug Code: {}", (instr << 8) >> 8);
                    }
                    _ => println!("error inmpossible subcode: {} {}", opcode, subcode),

                }
                pc += 1;
            }
            1 => {
                println!("1");

                let mut offset: u32 = instr & 0xfffffff as u32;

                sp += offset as usize;

                if sp > 4095 {
                    sp = 4095;
                }

                println!("current sp: {}", sp);

                pc += 1;
            }
            2 => {
                // binary arithmetic
                let sub = ((instr >> 24) & 0xF) as u8;
                let r = pop_i32(&stack, &mut sp);
                let l = pop_i32(&stack, &mut sp);
                let res = match sub {
                    0 => l.wrapping_add(r),
                    1 => l.wrapping_sub(r),
                    2 => l.wrapping_mul(r),
                    3 => if r != 0 { l / r } else { 0 },
                    4 => if r != 0 { l % r } else { 0 },
                    5 => l & r,
                    6 => l | r,
                    7 => l ^ r,
                    8 => l.wrapping_shl(r as u32),
                    9 => ((l as u32).wrapping_shr(r as u32)) as i32,
                    11 => l.wrapping_shr(r as u32),
                    _ => 0,
                };
                push(&mut stack, &mut sp, res);
            }
            3 => {
                // unary arithmetic
                let sub = ((instr >> 24) & 0xF) as u8;
                let v = pop_i32(&stack, &mut sp);
                let res = match sub {
                    0 => -v,
                    1 => !v,
                    _ => v,
                };
                push(&mut stack, &mut sp, res);
            }
            4 => {
                // stprint
                let raw = (instr >> 2) & 0x03FF_FFFF;
                let off = (sign_extend(raw, 26) << 2) as isize;
                let mut addr = (sp as isize + off) as usize;
                while addr < 4096 {
                    let b = stack[addr];
                    addr += 1;
                    if b == 0 {
                        break;
                    }
                    if b == 1 {
                        continue;
                    }
                    print!("{}", b as char);
                }
                io::stdout().flush().unwrap();
            }
            5 => {
                // call
                let raw = (instr >> 2) & 0x03FF_FFFF;
                let off = (sign_extend(raw, 26) << 2) as isize;
                let ret = (pc * 4) as i32;
                push(&mut stack, &mut sp, ret);
                let new_pc = ((pc as isize) + off / 4) as usize;
                pc = new_pc.min(4096 / 4);
            }
            6 => {
                // return
                let framesize = ((instr >> 2) & 0x03FF_FFFF) as usize * 4;
                sp = (sp + framesize).min(4096);
                let ret_i32 = pop_i32(&stack, &mut sp);
                let ret_usz = (ret_i32 as usize) / 4;
                pc = ret_usz.min(4096 / 4);
            }
            7 => {
                println!("7");
                pc += 1;
                let offset: isize = (((instr << 4) as i32) 
                                    >> 4) as isize;

                pc += offset;
            }
            8 => {
                println!("8");
                let condition: u8 = ((instr << 4) >> 29) as u8;
                let offset: isize = (((((instr >> 2) & 0x007fffff)
                                    << 9) as i32) >> 7) as isize;
                let mut bytes: [u8; 4] = [0; 4];
                let left: i32;
                let right: i32;

                bytes[0] = if sp <= 4095 {stack[sp]} else {0};
                bytes[1] = if sp + 1 <= 4095 {stack[sp+1]} else {0};
                bytes[2] = if sp + 2 <= 4095 {stack[sp+2]} else {0};
                bytes[3] = if sp + 3 <= 4095 {stack[sp+3]} else {0};

                let right = (bytes[3] << 24) as i32 |
                                 (bytes[2] << 16) as i32 |
                                 (bytes[1] << 8) as i32  |
                                 (bytes[0]) as i32;

                bytes[0] = if sp + 4 <= 4095 {stack[sp]} else {0};
                bytes[1] = if sp + 5 <= 4095 {stack[sp+1]} else {0};
                bytes[2] = if sp + 6 <= 4095 {stack[sp+2]} else {0};
                bytes[3] = if sp + 7 <= 4095 {stack[sp+3]} else {0};

                let left = (bytes[3] << 24) as i32 |
                                 (bytes[2] << 16) as i32 |
                                 (bytes[1] << 8) as i32  |
                                 (bytes[0]) as i32;

                match condition {
                    0 => if left == right {pc += offset-1;}
                    1 => if left != right {pc += offset-1;}
                    2 => if left < right {pc += offset-1;}
                    3 => if left > right {pc += offset-1;}
                    4 => if left <= right {pc += offset-1;}
                    5 => if left >= right {pc += offset-1;}
                    _ => println!("BinIf invalid condition"),
                }
                pc += 1;

            }
            9 => {
                println!("9");

                let condition: u8 = ((instr << 5) >> 30) as u8;
                let offset: isize = (((instr << 7) as i32) >> 7) as isize;

                let value = (stack[sp+3] << 24) as i32 |
                                 (stack[sp+2] << 16) as i32 |
                                 (stack[sp+1] << 8) as i32  |
                                 stack[sp] as i32;

                match condition {
                    0 => if value == 0 {pc += offset-1;}
                    1 => if value != 0 {pc += offset-1;}
                    2 => if value < 0 {pc += offset-1;}
                    3 => if value >= 0 {pc += offset-1;}
                    _ => println!("UnIf bad condition"),
                }

                pc += 1;
            }
            12 => {
                println!("12");

                let offset: usize = ((instr << 4) >> 4) as usize;
                let mut value: [u8; 4] = [0; 4];

                sp -= 4;
    
                value.copy_from_slice(&stack[sp+4+offset..sp+8+offset]);  

                stack[sp..sp+4].copy_from_slice(&value);

                pc += 1;
            }
            13 => {
                println!("13");
                let offset: usize = ((instr << 4) >> 6) as usize;
                let fmt: u8 = (instr & 0b11) as u8;
                let value: i32 = ((stack[sp + offset + 3] as u32) << 24 |
                                 (stack[sp + offset + 2] as u32) << 16 |
                                 (stack[sp + offset + 1] as u32) << 8 | 
                                 stack[sp + offset] as u32) as i32;

                match fmt {
                    0 => {
                        println!("{}", value);
                    }
                    1 => {
                        println!("0x{:x}", value);
                    }
                    2 => {
                        println!("0b{:b}", value);
                    }
                    3 => {
                        println!("0o{:o}", value);
                    }
                    _ => println!("Bad fmt"),
                }

                pc += 1;
            }
            14 => {
                println!("14");
                pc += 1;
            }
            15 => {
                println!("15");
                let value: i32 = ((instr << 4) as i32) >> 4;

                // println!("value: {}", value);
                value.to_le_bytes().iter().for_each(|x| println!("{}", *x as char));
                sp -= 4;

                stack[sp..sp+4].copy_from_slice(&value.to_le_bytes());

                stack[sp..sp+4].iter().for_each(|x| println!("{:08b}", x));
                // println!("current sp: {}", sp);

                pc += 1;
            }
            _ => println!("Error, not an opcode: {}", opcode),
        }

    }

}


