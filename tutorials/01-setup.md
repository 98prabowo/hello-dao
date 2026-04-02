# Environment Setup

## Prerequisites

### - [Rust](https://rust-lang.org) installed:

1. Install Rust using [rustup](https://rust-lang.org/learn/get-started) by entering the following command in your terminal:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. Reload your PATH environment variable to include [Cargo's](https://github.com/rust-lang/cargo) bin directory:

```sh
. "$HOME/.cargo/env"
```

3. Verify that the installation was successful.

```sh
rustc --version
```

### - [Solana CLI](https://solana.com/docs/intro/installation/solana-cli-basics) installed:

1. Install the Solana CLI tool suite by using the official install command:

```sh
sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"
```

2. Add a PATH environment variable (Mac):

```sh
echo 'export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"' >> ~/.zshrc
```

3. Add a PATH environment variable (Windows & Linux):

a.  Check which shell you are using:

```sh
echo $SHELL
```

- If the output contains `/bash`, use `.bashrc`.
- If the output contains `/zsh`, use `.zshrc`.

b. Run the appropriate command, based on your shell.

For Bash (`bashrc`):

```sh
echo 'export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"' >> ~/.bashrc
```

For Zsh (`zshrc`):

```sh
echo 'export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"' >> ~/.zshrc
```

4. Restart your terminal or run the following command to refresh the terminal session:

```sh
source ~/.bashrc # If using Bash
source ~/.zshrc # If using Zsh
```

5. Verify that the installation succeeded by checking the Solana CLI version:

```sh
solana --version
```

## installation

```sh
git clone https://github.com/98prabowo/hello_dao.git # With HTTPS
git clone git@github.com:98prabowo/hello_dao.git # With SSH
cd hello_dao
cargo build
```

[⬅️ Previous: Readme](../README.md) | [Next: State Definitions ➡️](02-states.md)
