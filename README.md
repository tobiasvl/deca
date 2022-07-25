Deca
====
[![crates.io](https://img.shields.io/crates/v/deca.svg)](https://crates.io/crates/deca)
[![docs.rs](https://img.shields.io/docsrs/deca.svg)](https://docs.rs/deca)
[![dependency status](https://deps.rs/repo/github/tobiasvl/deca/status.svg)](https://deps.rs/crate/deca)

Deca is basically just another CHIP-8 emulator written in Rust, but it

* is a backend library with no I/O, which can be embedded into a UI (like [Termin-8](https://crates.io/crates/termin-8), which runs in a terminal).
* has full CHIP-8, SUPER-CHIP and XO-CHIP support (multiple colors!)
* aims to behave as closely to [Octo](https://github.com/JohnEarnest/Octo)/[C-Octo](https://github.com/JohnEarnest/c-octo) as possible

Future plans
------------

- [ ] Support for various [CHIP-8 extensions and variations](https://chip-8.github.io/extensions/), so it can run as many historical CHIP-8 programs as possible
- [ ] Other crates that can compile Octo programs ([decasm](https://github.com/tobiasvl/decasm)) and read/write Octocarts ([decart](https://github.com/tobiasvl/decart)), inspired by the components of [C-Octo](https://github.com/JohnEarnest/c-octo#project-structure) (possibly even an IDE ([decade](https://github.com/tobiasvl/decade)) inspired by Octode??)

Why "Deca"?
-----------

Deca is inspired by (heavily based on) [Octo](https://github.com/JohnEarnest/Octo), which has an octopus as a mascot (as a reference to CHIP-8). Octopi are eight-limbed molluscs belonging to the order Octopoda. Deca is written in Rust, which has a crab as its mascot. Crabs are ten-limbed crustaceans of the order Decapoda.

To further justify the silly name, I plan to add support for [CHIP-10](https://chip-8.github.io/extensions/#chip-10) as well.
