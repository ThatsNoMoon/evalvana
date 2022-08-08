# Evalvana

Evalvana is an extremely WIP REPL environment for any language with a plugin.

![Evalvana interface demo](https://github.com/ThatsNoMoon/evalvana/blob/a35454baa8d803e24643358a820cce3e5d85199a/assets/misc/demo_screenshot.png?raw=true)

## Intended direction

Evalvana's goal is to provide a clean, powerful REPL environment for any language. Being in an extremely early stage of development, how this is going to be realized isn't set in stone yet, but the current plan is that a plugin will be some program that takes evaluation requests in a JSON format through stdin, and outputs the result in JSON through stdout. This approach provides the flexibility of being able to implement the REPL in the language it's evaluating.

The benefits Evalvana wants to bring to developers include:
- Keeping the REPL for every language in one application, with a consistent interface
- Improve the REPL experience beyond what a barebones terminal interface can provide
- Make it easier to transition from using a REPL to using a file

## License

Evalvana is licensed under the [AGPL v3.0](https://choosealicense.com/licenses/agpl-3.0/).

Plugins included in this source tree (those under the plugins directory, not including the API) are licensed under the [BSD Zero Clause License](https://choosealicense.com/licenses/0bsd/); in summary, you may use that code with no restriction or warranty. Feel free to copy them to start your own plugin under any other license.

The plugin API (plugins/api) is licensed under [OSL 3.0](https://choosealicense.com/licenses/osl-3.0/). Derivatives must be licensed under OSL 3.0, but other projects linking to the API can use any license.

The editor widget (in the editor directory) is derived from the `iced_native` `TextInput` widget and licensed under the [MIT](https://choosealicense.com/licenses/mit/) license.
