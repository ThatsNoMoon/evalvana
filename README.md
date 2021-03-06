# Evalvana

Evalvana is an extremely WIP (no functionality yet) REPL environment for any language with a plugin.

![Evalvana interface demo](https://github.com/ThatsNoMoon/evalvana/blob/stable/assets/misc/demo_screenshot.png?raw=true)

## Intended direction

The goal of evalvana is to be able to provide a clean, powerful REPL environment for any language. Being in an extremely early stage of development, how this is going to be realized isn't totally clear yet, but the current theory is that a plugin will be some program that takes evaluation requests in a JSON format through stdin, and outputs the result in JSON through stdout. This approach provides the flexibility of being able to implement the REPL in the language it's evaluating.

The point of evalvana is to improve the REPL experience beyond a barebones text environment in a CLI. Features like autocomplete and intellisense would be nice, but the priority is less on IDE-like features and more on REPL-specific features such as hovering over variables to inspect their value, and being able to easily transition from using a REPL to using a file (something most REPLs are sorely lacking).

## Building

Evalvana requires [Cargo](https://rustup.rs/), [CMake](https://cmake.org/download/), [Ninja](https://ninja-build.org/), and [Python 3](https://www.python.org/downloads/) to build. CMake, Ninja, and Python 3 must be in a folder in your PATH environment variable. After that, `cargo build` should handle building.

## License

Evalvana is licensed under the [OSL 3.0](https://choosealicense.com/licenses/osl-3.0/). Derivatives must be licensed under OSL 3.0, but this does not include any linking restrictions; you may link this code to closed-source code.
