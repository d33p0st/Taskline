# taskline-init

Ultra-fast script initialization tool for the Taskline scripting framework.

## Overview

`taskline-init` is a high-performance initialization utility that creates optimized Taskline script templates with proper metadata headers and version management. Built with Rust for maximum speed and minimal resource usage.

## Features

- ⚡ **Ultra-fast initialization** - Optimized for sub-millisecond script creation
- 🎯 **Minimal dependencies** - Only essential dependencies for maximum performance
- 📝 **Smart templating** - Automatic header generation with metadata
- ✅ **Version validation** - Built-in semantic version parsing and validation
- 🔒 **Safe operations** - Prevents accidental file overwrites
- 🚀 **Async I/O** - Non-blocking file operations for better performance

## Installation

```bash
cargo install taskline-init
```

## Usage

### Basic Initialization
```bash
taskline-init my-script
# Creates: my-script.tskln
```

### With Version
```bash
taskline-init my-script v1.0.0
# Creates: my-script.v1.0.0.tskln
```

### Generated Template
```taskline
@Taskline codename my-script
@Taskline version v1.0.0

-> Your Taskline script content goes here
```

## Performance

- **Binary size**: ~2MB (stripped)
- **Cold start**: <1ms
- **Memory usage**: ~1MB peak
- **Initialization time**: <0.1ms per script

## Part of Taskline Framework

`taskline-init` is part of the larger Taskline ecosystem - a Rust-backed script bridge for creating ultra-fast executing scripts with custom syntax that compiles to optimized Rust code for both synchronous and asynchronous execution.

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.