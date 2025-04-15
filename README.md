# AloeVera OS 🌱

Welcome to AloeVera OS! This isn't your average operating system; it's a **hobby project sprouting** in the fertile ground of the **Rust programming language**. We're exploring the world of OS development with a specific focus on **asynchronous operations** and making **consistent, understandable interfaces** for interacting with the system's core components. Think of it as cultivating a unique digital plant, carefully nurtured from the kernel up.

Like our namesake plant, we aim for simplicity and resilience.

[Documentation] `rustdoc` for all crates used in the project are hosted on GitHub Pages.

> [!WARNING]  
> This project is currently in **very early** development. Furthermore, the project has recently been renamed from `QuantumOS` and might still contain references to such.  

[Documentation]: https://corigan01.github.io/AloeVera/aloe/index.html

## Our Philosophy: How AloeVera Thrives 🪴

AloeVera's growth is guided by a few key principles:

### Branching Out: A Fresh Approach
AloeVera isn't growing in the familiar soil of traditional Unix-based systems. We're intentionally **branching out**, seeking to fundamentally rethink how user programs interact with the operating system core.

Our approach is heavily inspired by **Rust's powerful type system**. We aim to cultivate interfaces between user space and the kernel that are more explicit, robust, and verifiable, potentially catching errors at compile-time that might only surface at runtime in other systems. This exploration moves away from mimicking standard POSIX interfaces towards potentially novel ways of requesting system resources and services.

This design philosophy is rooted in a core commitment to **stability and security**. By leveraging type safety and rethinking core interactions, we hope to build a more resilient and secure foundation from the ground up.

### Consistent Structure: Like a Healthy Plant's Form
Nature often exhibits beautiful, repeating patterns. Similarly, AloeVera strives for **consistency in its internal structure and interfaces**. Interacting with different parts of the system (files, devices, IPC) should feel familiar and predictable. We achieve this by defining common traits and patterns, aiming to lower the learning curve and promote a robust, maintainable architecture.

### Homegrown Ingredients: Nurtured In-House
One of the core tenets of AloeVera is its **"built from scratch" philosophy**. Like tending a garden with soil you mixed yourself, we rely almost entirely on **in-house libraries** for the core OS runtime, building directly upon Rust's `core` and `alloc` crates. You won't find external dependencies from `crates.io` powering the kernel's core logic (except, pragmatically, within the `meta` build system itself). This approach stems from a desire for deep learning and complete control over the system's foundations, ensuring it grows organically from known elements and providing a unique environment for exploring low-level concepts.

## Getting Your Hands Dirty

AloeVera is currently a budding hobby project, so expect rough edges! If you'd like to try building and running it:

1.  **Prerequisites:** You'll likely need a recent Rust nightly toolchain, QEMU or Bochs (for emulation), and core `llvm` libraries (for tools like `objdump` and `objcopy`).
2.  **Building and Running:** `cargo run` is all you need to get up and running in QEMU! For more configuration options, check `cargo run -- --help`.
3.  **Exporting:** Once built, `meta` can be used to generate a `qcow2` disk image using `cargo run -- build-disk`.

**Disclaimer:** Building a large project for the first time can be tricky! Check [build instructions](/BUILD.md) first. Feedback via GitHub Issues is welcome as well!

## Help Cultivate AloeVera

This project is cultivated with enthusiasm, and we warmly welcome fellow gardeners! Whether you're experienced with OS development or just starting with Rust, your contributions are valuable. This is a learning journey for everyone involved.

We appreciate contributions of all kinds:

* Implementing new features (drivers, syscalls, filesystem parts).
* Improving existing code and our internal libraries (refactoring, performance).
* Writing tests.
* Enhancing documentation (especially for our homegrown parts!).
* Reporting bugs and suggesting ideas.

Given our reliance on internal libraries and interfaces, contributions strengthening this core foundation are particularly helpful. Please check the **GitHub Issues tab** for tasks!

## License 📜

AloeVera OS is freely available and licensed under the **MIT License**. See the `LICENSE-MIT` file for more details.

---

Happy Planting! 🌱
We hope you enjoy exploring the roots and shoots of AloeVera!

