#![no_std]
#![no_main]

use lib::*;
use crate::vec::Vec;
extern crate lib;

fn main() -> isize {
    println!("starting shell...");
    shell();
    println!("exiting shell...");
    233
}

fn shell(){
    loop {
        print!("shell > ");
        let mut op=stdin().read_line();
        let ops:Vec<_>=op.split_whitespace().collect();
        if ops.is_empty() {
            continue;
        }
        match ops[0] {
            "ps"=>sys_stat(),
            "ls"=>sys_list_app(),
            "run"=>run(&ops[1]),
            "help"=>help(),
            "clear"=>print!("\x1B[2J\x1B[1;1H"),
            "exit"=>return (),
            _ => println!("{}: command not found",op),
        };
    }
}

fn help(){
    println!("Help from 22331116.");
    println!("Supported commands:");
    println!("ps\t\tList all currently running processes.");
    println!("ls\t\tList all available user programs.");
    println!("run <app>\tRun a specified user program. Replace <app> with the name of the program.");
    println!("help\t\tDisplay this help message.");
    println!("clear\t\tClear the screen.");
    println!("exit\t\tExit the shell.");
}

fn run(name: &str){
    let ret = sys_wait_pid(sys_spawn(name));
    println!("{} exit with code {}", name, ret);
}

entry!(main);
