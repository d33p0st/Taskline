# Taskline Publishing Guide

## 📦 Publishing Strategy

Taskline uses a **separate package approach** where each component is published independently to crates.io, but the main package provides automatic installation of all components.

### Package Structure

1. **`taskline`** - Main CLI dispatcher and installer
2. **`taskline-init`** - Script initialization tool  
3. **`taskline-bump`** - Version bumping tool

### Publishing Steps

#### 1. Publish Component Packages First

```bash
# Publish taskline-init
cd taskline-init
cargo publish

# Publish taskline-bump  
cd ../taskline-bump
cargo publish
```

#### 2. Publish Main Package

```bash
# Publish main taskline package (from root)
cargo publish
```

### User Installation Experience

#### Option 1: Install Everything (Recommended)
```bash
# Install main CLI (this gives access to 'taskline install')
cargo install taskline

# Install all components automatically
taskline install
```

#### Option 2: Manual Installation
```bash
# Install each component separately
cargo install taskline
cargo install taskline-init
cargo install taskline-bump
```

#### Option 3: Check Installation Status
```bash
# Check what's installed
taskline doctor
```

### Development Installation

For local development and testing:

```bash
# Use the provided script
./install.sh

# Or install manually
cargo install --path .
cargo install --path taskline-init
cargo install --path taskline-bump
```

### User Experience Flow

1. User runs: `cargo install taskline`
2. User gets the main `taskline` CLI
3. User runs: `taskline install` (automatically installs taskline-init, taskline-bump)
4. User can now use: `taskline init`, `taskline bump`, etc.

### Benefits of This Approach

✅ **Clean separation** - Each tool is independently maintainable
✅ **Selective installation** - Users can install only what they need
✅ **Automatic setup** - Main package can install everything
✅ **Easy updates** - Each component can be updated independently
✅ **Clear dependencies** - No complex dependency trees

### Commands Available After Installation

```bash
taskline init <filename> [version]     # Initialize new scripts
taskline bump <filename> [type]        # Bump script versions
taskline install [--force]             # Install/update all components
taskline doctor                        # Check installation status
taskline --help                        # Show help
```