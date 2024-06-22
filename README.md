# cloud_shell
A bundle of helpful command line utilities.
  * klog

## How to build
  * Install rust 
    * `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
    * `source $HOME/.cargo/env`
    * Verify rust installation by `rustc --version`. You should see output similar to `rustc 1.76.0 (07dca489a 2024-02-04)`
  * Clone Repository 
    * `git clone https://github.com/vishalcjha/cloud_shell.git`
    * `cd cloud_shell`
    * `cargo build` - this will build the project. It can take up to a minute to finish.
  * Running binaries - I am hoping to add multiple binaries. For now we just have `klog`.
    * `cargo run --bin klog -- [pod name prefix] [-a +5s]` . Here we are asking cargo to run binary `klog` with its parameter. All binary comes with help. `cargo run --bin klog -- -help` will do the trick. Please be mindful of `-` in command. First `--` after binary name specifies beginning of parameters. After required parameter all other option must have `-`.


### klog
Tail log in multiple pods based on pod name prefix and optional created time.

```
Usage: cargo run --bin klog -- <pod-label>... -[options]

Arguments:
  <pod-label>...  Pod name prefix(s)

Options:
  -a, --catime <catime>  [+]n[h|m|s]
  -b, --cbtime <cbtime>  [+]n[h|m|s]
  -h, --help             Print help (see more with '--help')