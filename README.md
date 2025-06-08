# Munal OS ∴

An experimental operating system written in Rust, with cooperative scheduling and a security model based on WASM sandboxing.

Features:

* Fully graphical interface in HD resolution with mouse and Keyboard support
* Sandboxed applications
* Network driver and TCP stack
* Customizable UI toolkit supporting responsive layouts and flexible text rendering
* Embedded selection of applications including:
  * A (very) basic web browser
  * Text editor
  * Python terminal

## Architecture

Munal OS started as a toy project to practice systems programming, and over the years morphed into a full-blown OS and a playground to explore new ideas. It aims to re-examine principles of OS design, and see how much is really needed today to make a functional OS, and where shortcuts can be taken using modern tools. The design has no pretention to be superior to anything else, rather it is an experiment focusing on the simplicity of the codebase (but not necessarily the codebase AND its dependencies).

In particular, here are usual cornerstones of OS design that Munal OS does **NOT** implement:

* Bootloader
* Page mapping
* Virtual address space
* Interrupts

### EFI binary 

Munal OS has no bootloader; instead, the entire OS is compiled into a single EFI binary that embeds the kernel, the WASM engine and all the applications. The UEFI boot services are exited almost immediately and no UEFI services are used except for the system clock.

### Address space

UEFI leaves the address space as identity-mapped and Munal OS does not remap it. In fact the page tables are not touched at all, because the OS does not make use of virtual address mechanisms. The entire OS technically runs within a single memory space, but something akin to the userspace/kernelspace distinction is provided by WASM sandboxing (see below), preventing arbitrary access to kernel memory by user applications.

### Drivers

### Event loop

### Applications

### UI Library

## Demo videos

## Credits & acknowledgements

## Demo running in QEMU:

[Screencast_20250215_121948.webm](https://github.com/user-attachments/assets/8cbf8a42-c012-4610-8668-014093efc09d)

Font credits:
* https://fontesk.com/xanmono-font/
* https://fontesk.com/libertinus-typeface/
* https://fontesk.com/major-mono-font/
