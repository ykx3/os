# RustOS: An Educational Operating System in Rust

## Introduction

RustOS is an operating system developed as part of the undergraduate Operating Systems course at Sun Yat-sen University, 2022. This project aims to provide hands-on experience in building an OS kernel using Rust, a modern systems programming language that offers safety and performance.

## Project Background

This repository is based on the [YatSenOS Tutorial Volume 2](https://github.com/YatSenOS/YatSenOS-Tutorial-Volume-2), a comprehensive guide for students to learn and implement their own operating system kernel. The tutorial covers various aspects of OS development including bootstrapping, memory management, process scheduling, and device drivers.

## Project Scope

As a student project, this implementation is not intended for production use but serves educational purposes. It demonstrates the fundamental concepts and functionalities expected from a basic OS kernel. Due to time constraints and limited resources, the scope of this project is restricted to achieving the core objectives outlined by the course curriculum.

## Features

- Bootstrapping: Basic booting and initialization.
- Memory Management: Simple memory allocation and management.
- Process Scheduling: Basic process creation and scheduling.
- Interrupt Handling: Support for interrupt handling and system calls.
- Device Drivers: Minimal support for common devices.

## Getting Started

To compile and run RustOS, you will need:

- A working Rust toolchain (rustup, cargo)
- A QEMU emulator or similar virtual machine environment

### Installation

Clone this repository:

```
git clone https://github.com/ykx3/os.git
cd os
```

Run the OS:

```
python ysos.py run
```

### Documentation

For detailed instructions and further documentation, refer to the [中山大学 YatSenOS v2 操作系统实验教程](https://ysos.gzti.me).

## License

This project is licensed under the MIT License - see the "LICENSE" file for details.

---
*Last updated: August 4, 2024*