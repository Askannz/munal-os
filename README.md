# Munal OS ∴

An experimental operating system written in Rust, with cooperative scheduling and a security model based on WASM sandboxing.

Features:

* Fully graphical interface in HD resolution with mouse and keyboard support
* Sandboxed applications
* Network driver and TCP stack
* Customizable UI toolkit supporting various widgets, responsive layouts and flexible text rendering
* Embedded selection of applications including:
  * A (very) basic web browser
  * A text editor
  * A Python terminal

## Architecture

Munal OS started as a toy project to practice systems programming, and over the years morphed into a full-blown OS and a playground to explore new ideas. It aims to re-examine principles of OS design, and see how much is really needed today to make a functional OS, and where shortcuts can be taken using modern tools. The design has no pretention to be superior to anything else, rather it is an experiment focusing on the simplicity of the codebase (even if it relies on heavy dependencies like a WASM engine).

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

Munal OS does not rely on PS/2 inputs or VGA/UEFI GOP framebuffers for display. Instead, it implements a PCI driver which is used to communicate with QEMU via the [VirtIO 1.1 specification](https://docs.oasis-open.org/virtio/virtio/v1.1/csprd01/virtio-v1.1-csprd01.html). A generic virtqueue system serves as the basis for 4 different VirtIO drivers: keyboard, mouse, network and GPU. Notably, the drivers are entirely polling-based and do not rely on system interrupts at all (in fact Munal OS does not implement any).

The reliance on VirtIO means Munal OS does not support running on real hardware yet; more work would be needed, either to use BIOS/UEFI-provided methods (such as PS/2, VGA, GOP) or to implement full-blown GPU and USB drivers.

### Event loop

For simplicity, Munal OS does not implement multi-core support or even interrupts, and everything happens linearly within one single, global event loop. Every iteration of the loop polls the network and input drivers, draws the desktop interface, runs one step of each active WASM application, and flushes the GPU framebuffer.

One advantage of this approach is that it is trivial to inspect the performance of each OS component and user application, simply by measuring how much of the total frametime they eat. For now, the loop should run at well over 60 FPS on a modern CPU with all applications open.

The downside of course is that each step of the loop is not allowed to hold the CPU for too long, and must explicitly yield for long-running tasks.

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
