# SE202 Rust - LED Matrix
The purpose of this repository is to show a little of the Rust course I followed in my master's 1 programme, as part of the Embedded Systems study track I chose. Here it's possible to find a Rust programme for controlling a LED Matrix through a STM32L4 MCU. 

## Meta
 * **Master in Electrical Engineering - Institut Polytechnique de Paris**
 * **Course:** SE202 Rust
 * **Author:** Alaf DO NASCIMENTO SANTOS
 * **License**: [MIT](LICENSE)
 * **Year:** 2023

## Installing Rust
If you are using a Linux machine, the following command would be enough: ***curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh***
For more information, please take a look at the Rust official website: https://www.rust-lang.org/tools/install

## How to compile and execute the project?
Firstly, use the following commands in order to add some pre-compiled components that we will need in our project:

```
rustup target add thumbv7m-none-eabi

rustup target add thumbv7em-none-eabihf
```

After installing cargo, you can use the command ***cargo build***, which compiles the project.

For running a Rust programme, you can use the command line ***cargo run***, but in this project you must give some arguments to the loader in order to run the binary. It is crucial to remember that this is an embedded project focused on a STM32L4 MCU with a 8x8 LED Matrix, without the right connected hardware, the previous command is not supposed to work.

## How to Contribute to the Project
- Any implementation that could lead to a more optimised code for the different methods already designed would be a nice improvement for this project. 

So feel free to fork, change, and pull request 😊




