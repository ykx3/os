#![no_std]
#![no_main]

use lib::*;
use crate::vec::Vec;
use crate::string::String;
extern crate lib;

fn main() -> isize {
    println!("starting shell...");
    shell();
    println!("exiting shell...");
    0
}

fn shell(){
    // print!("\x1B[2J\x1B[1;1H");
    //当前路径
    let mut path = String::from("/");
    loop {
        //提示词
        print!("\x1b[32;1mykx@YSOS\x1b[0m:\x1b[34;1m{}\x1b[0m$ ",path);
        let op=stdin().read_line();
        let ops:Vec<_>=op.split_whitespace().collect();
        if ops.is_empty() {
            continue;
        }
        match ops[0] {
            "ps"=>sys_stat(),
            "ls_app"=>sys_list_app(),
            "pwd"=>println!("{}",path),
            "ls" => {
                if let Some(target_path) = ops.get(1) {
                    // 如果提供了路径参数，则列出该路径下的内容
                    sys_list_dir(&normalize_path(&path, target_path));
                } else {
                    // 否则列出当前路径下的内容
                    sys_list_dir(&path);
                }
            }
            "cat" => {
                if let Some(target_path) = ops.get(1) {
                    // 如果提供了路径参数，则列出该路径下的内容
                    sys_cat(&normalize_path(&path, target_path));
                } else {
                    println!("Error: missing <file name>");
                    help(core::prelude::v1::Some("cat"));
                }
            },
            "cd" => {
                if let Some(target_path) = ops.get(1) {
                    // 如果提供了路径参数，则列出该路径下的内容
                    path = normalize_path(&path, target_path);
                } else {
                    println!("Error: missing <dir name>");
                    help(core::prelude::v1::Some("cd"));
                }
            },
            "run" => {
                if ops.len() > 1 {
                    run(ops[1]);
                }
            },
            "sleep"=>sleep(ops[1].parse().unwrap_or(0)),
            "help"=>{
                let maybe_command = ops.get(1); // 尝试获取用户可能提供的命令参数
                help(maybe_command.map(|&str| str)); // 传递该参数给 help 函数
            },
            "clear"=>print!("\x1B[2J\x1B[1;1H"),
            "exit"=>return (),
            _ => println!("{}: command not found",op),
        };
    }
}

fn help(maybe_command: Option<&str>) {
    let help_texts = vec![
        ("ps", "List all currently running processes."),
        ("ls", "List directory contents. Usage: ls [path]"),
        ("ls_app","List all available user programs."),
        ("run", "Run a specified user program. Replace <app> with the name of the program. Usage: run <app>"),
        ("sleep", "Sleep for a specified number of milliseconds. Usage: sleep <ms>"),
        ("help", "Display this help message. Usage: help [command]"),
        ("clear", "Clear the screen."),
        ("exit", "Exit the shell."),
        ("cd", "Change the current directory. Usage: cd <path>"),
        ("cat", "Concatenate and print files to the standard output. Usage: cat <file>"),
    ];

    match maybe_command {
        Some(command) => {
            // 定向查询命令帮助信息
            let matching_help = help_texts.iter().find(|(cmd, _)| *cmd == command);
            match matching_help {
                Some((_, info)) => println!("{} - {}", command, info),
                None => println!("Help not found for: {}", command),
            }
        },
        None => {
            // 默认输出全部帮助信息
            println!("Help from 22331116.");
            println!("Supported commands:");
            for (cmd, info) in help_texts {
                println!("{}\t- {}", cmd, info);
            }
        },
    }
}

fn run(name: &str){
    let ret = sys_wait_pid(sys_spawn(format!("/app/{}",name).as_str()));
    println!("{} exit with code {}", name, ret);
}

pub fn sleep(millisecs: u64) {
    let start = sys_time();
    println!("start sleep in {}", start);
    let mut current = start;
    while current - start < millisecs {
        current = sys_time();
    }
    println!("wake up in {}", current);
}

fn normalize_path(current_path: &str, input_path: &str) -> String {
    let mut path_stack: Vec<&str> = if current_path == "/" { vec![] } else { current_path.split('/').collect() };

    for part in input_path.split('/') {
        match part {
            ".." => { path_stack.pop(); }, // 返回上级目录
            "." => {}, // 表示当前目录，不做操作
            "" => {}, // 跳过空字符串，可能由连续的 '/' 产生
            _ => path_stack.push(part), // 其他情况加入到路径中
        }
    }
    
    // 如果路径是空的，说明它是根目录
    if path_stack.is_empty() {
        return String::from("/");
    }

    // 重建路径
    let normalized_path = path_stack.join("/");
   
    // 确保路径以 "/" 开头
    if !normalized_path.starts_with('/') {
        return format!("/{}", normalized_path);
    }
    normalized_path
}

entry!(main);
