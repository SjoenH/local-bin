# lspkg

`lspkg` is a simple shell script that helps you list all the package names in your project. It searches for all `package.json` files in your project and extracts the `name` field from each of them.

### Usage

To use `lspkg`, navigate to your project's root directory in your terminal and run the script:

```bash
./lspkg
```

This will output a list of all package names in your project. Note that packages without a `name` field in their `package.json` will not be included in the output.

### Requirements

- `fd`: A simple, fast and user-friendly alternative to 'find'.
- `jq`: A lightweight and flexible command-line JSON processor.
- `xargs`: A command line utility to build and execute commands from standard input.
- `grep`: A command-line utility for searching plain-text data sets for lines that match a regular expression.

Please ensure that these utilities are installed and available in your system's PATH to use `lspkg`.

### Note

This script does not modify any files in your project. It only reads the `package.json` files to extract the package names.

# Depcheck Shell Script

The `depcheck` shell script is a utility for checking which projects in your current directory are using a specific npm package. It's designed to be run from the command line with the package name as an argument.

### Usage

To use the script, navigate to the directory containing your projects and run the script with the package name as an argument:

```shellscript
./depcheck <package_name>
```

Replace `<package_name>` with the name of the npm package you want to check for.

### How it Works

The script begins by checking if any arguments were provided when the script was called. If no arguments were provided, it prints a usage message and exits with a status of 1, indicating an error.

If an argument was provided, it's stored in the variable `package_name` for later use. The script then prints a message to the console indicating which package it's checking for.

The script then uses the `find` command to search for `package.json` files in the current directory and its subdirectories, excluding any `node_modules` directories. For each `package.json` file it finds, it uses `grep` to check if the file contains a reference to the specified package. If it does, it uses `jq` to extract the name of the project from the `package.json` file and prints it to the console.

This way, you can quickly see which projects are using a specific npm package.

### Requirements

The script requires `jq` to be installed on your system. You can install it using your package manager. For example, on Ubuntu you would use:

```shellscript
sudo apt-get install jq
```

On macOS, you can use Homebrew:

```shellscript
brew install jq
```

### Note

This script is designed to be used in a Unix-like environment and may not work correctly on other systems.
