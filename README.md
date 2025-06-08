# Munal OS ∴

An experimental operating system written in Rust, with cooperative scheduling and a security model based on WASM sandboxing.

Features:
* Fully graphical interface in HD resolution with mouse and keyboard support
* Sandboxed applications
* Network driver and TCP stack
* Customizable UI toolkit providing various widgets, responsive layouts and flexible text rendering
* Embedded selection of applications including:
  * A (very) basic web browser
  * A text editor
  * A Python terminal

## Architecture

Munal OS started as a toy project to practice systems programming, and over the years morphed into a full-blown OS and a playground to explore new ideas. It aims to re-examine principles of OS design, and see how much is really needed today to make a functional OS, and where shortcuts can be taken using modern tools. The design has no pretention to be superior to anything else, rather it is an experiment striving for simplicity of the codebase (but not necessarily a lightweight binary or minimal dependencies).

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

The downside of course is that each step of the loop is not allowed to hold the CPU for arbitrary amounts of time, and must explicitly yield for long-running tasks.

### Applications

Munal OS embeds the [wasmi](https://github.com/wasmi-labs/wasmi) WASM engine for running WASM applications. This achieves full sandboxing of user applications and memory separation from the kernel without the use of a virtual address space. A "system call" API is provided by the kernel so that apps can interact with the system. In particular, apps can query mouse/keyboard events, open/use TCP sockets, and send output framebuffers which are then read by the OS and composited onto the desktop. This lets apps use any drawing library they want (at the cost of a framebuffer copy).

Because of its custom "system call" API, Munal OS does not aim for compatibility with the WASI standards. However, the [WASI Preview 1](https://github.com/WebAssembly/WASI/blob/main/legacy/README.md) standard is partially supported, mostly so that applications can be compiled without using `#![no_std]` (which is often a blocker for pulling in external dependencies). Only the bare minimum is implemented, and WASI functions that have no analog in Munal OS (e.g `path_rename()`) are simply stubbed.

Munal OS relies on cooperative scheduling, meaning that applications are given control of the CPU every iteration of the global event loop, and must explicitly relinquish it. This is less an intentional design decision and more a consequence of using Wasmi as the WASM engine, which does not support interrupting and resuming functions mid-excution. However Wasmi does support fuel limiting, and so it would be possible to terminate misbehaving apps that hold the CPU for too long (though that's not implemented yet).

### UI Library

Munal OS has its own UI toolkit (plainly named Uitk) which is used throughout the system. It is also used by WASM applications, thought that's just for convenience and consistency with the desktop styling; it is just a shared library and applications could in theory for any other library they wish, as long as it can render to a generic framebuffer.

Uitk is an immediate mode toolkit which supports some basic widgets: buttons, progress bars, text editing, scrollable canvas...a generic triangle rasterizer is also provided (which is used to draw the radial pie menu and the 3D cube demo),

Styling is supported via a global stylesheet which can be shared between OS and apps, and overriden for individual UI elements.

A basic caching system is implemented to avoid unnecessary redraws: for example, generic scrollable canvases (like the web browser) are split into "tiles" tracked with a unique content ID. A system based on Rust's mutability automatically keeps track of changes to the content, and so tiles are only redrawn if the content ID changes, and pulled from a cache otherwise.

## Screenshots & videos

**TODO**

[Screencast_20250215_121948.webm](https://github.com/user-attachments/assets/8cbf8a42-c012-4610-8668-014093efc09d)

## Credits & acknowledgements

* [Philipp Oppermann's great Rust OS tutorial](https://os.phil-opp.com/), which was the starting point of this whole project
* [The OSDev Wiki](https://wiki.osdev.org/) for resources on x86_64 and PCI drivers
* [The Wasmi WASM engine](https://github.com/wasmi-labs/wasmi), a great alternative to Wasmtime for embedded
* [smoltcp](https://github.com/smoltcp-rs/smoltcp) for the TCP stack
* [Rustls](https://github.com/rustls/rustls) for the TLS primitives
* [RustPython](https://github.com/RustPython/RustPython) for the embeddable Python implementations

Fonts used:
* https://fonts.google.com/noto/specimen/Noto+Sans
* https://fontesk.com/xanmono-font/
* https://fontesk.com/libertinus-typeface/
* https://fontesk.com/major-mono-font/

Icons used:
**TODO**
