use std::io::{self, Read, Write, BufReader, Stdin};
use std::fs::File;
use std::env::args;
use std::process::exit;

fn main() {
    let mut stack: [u8; 4096] = [0; 4096];
    let mut sp: usize = 4096;
    let mut pc: usize = 0;
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

    //stack.iter().for_each(|x| println!("{:02x}", x));
    // println!();

    // for i in 0..bytes_read {
    //     println!("{:02x}", instr[i]);
    // }

    // println!("{}", stack_pointer);
    // println!("{:02x}", stack[stack_pointer]);
    
    loop {
        instr = (stack[(pc*4)+3] as u32) << 24 |
                (stack[(pc*4)+2] as u32) << 16 |
                (stack[(pc*4)+1] as u32) << 8 |
                (stack[pc*4] as u32);
        println!("{:032b}", instr);
        pc += 1;
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
                            inp_bytes = i32::from_str_radix(&inp_val, 16).expect("Input hex failed").to_be_bytes();
                            inp_bytes.iter().for_each(|x: &u8| println!("{:08b}", x));
                        } else if inp_type == "0B" {
                            println!("bits");
                            inp_val = inp.chars().skip(2).collect();
                            inp_bytes = i32::from_str_radix(&inp_val, 2).expect("Input bits failed").to_be_bytes();
                            inp_bytes.iter().for_each(|x: &u8| println!("{:08b}", x));
                        } else {
                            println!("base 10");
                            inp_bytes = inp.parse::<i32>().expect("Input base 10 failed").to_be_bytes();
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

                sp += 4;
                if sp > 4095 {
                    sp = 4095;
                }

                pc += 1;
            }
            2 => {
                // binary arithmetic
                let sub = ((inst >> 24) & 0xF) as u8;
                let r = self.pop_i32();
                let l = self.pop_i32();
                let res = match sub {
                    0 => l.wrapping_add(r),
                    1 => l.wrapping_sub(r),
                    2 => l.wrapping_mul(r),
                    3 => if r != 0 { l / r } else { 0 },
                    4 => if r != 0 { l % r } else { 0 },
                    5 => l & r,
                    6 => l | r,
                    7 => l ^ r,
                    8 => l.wrapping_shl(r as u32), // potentially need different implementation here?
                    9 => ((l as u32).wrapping_shr(r as u32)) as i32,
                    11 => l.wrapping_shr(r as u32), // asr
                    _ => 0,
                };
                self.push(res); 
            
            }
            3 => {
                // unary
                let sub = ((inst >> 24) & 0xF) as u8;
                let v = self.pop_i32();
                let res = match sub {
                    0 => -v,
                    1 => !v,
                    _ => v,
                };
                self.push(res);
            }
            4 => {
                println!("4");
            }
            5 => {
                println!("5");
            }
            6 => {
                println!("6");
            }
            7 => {
                println!("7");
            }
            8 => {
                println!("8");
            }
            9 => {
                println!("9");
            }
            12 => {
                println!("12");
            }
            13 => {
                println!("13");
            }
            14 => {
                println!("14");
            }
            15 => {
                println!("15");
            }
            _ => println!("Error, not an opcode: {}", opcode),
        }

    }

}


