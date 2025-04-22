git-blame-parser
================

[![Badge Workflow](https://github.com/mitsu-ksgr/git-blame-parser/actions/workflows/rust.yml/badge.svg)](https://github.com/mitsu-ksgr/git-blame-parser/actions)

Parses the output of `git blame` command in
[the porcelain format](https://git-scm.com/docs/git-blame#_the_porcelain_format)
into a struct.
the output must be generated using the `--line-porcelain` option.


## Usage
Run the following Cargo command in your project directory:

```sh
% cargo add git-blame-parser
```

Or add the following line to your `Cargo.toml`:

```toml
[dependencies]
git-blame-parser = "0.1.0"
```

Then:

```rust
let output = std::process::Command::new("git")
    .args(["blame", "--line-porcelain"])
    .arg(filepath)
    .output()
    .unwrap();

if output.status.success() {
    let raw_blame = String::from_utf8_lossy(&output.stdout);
    let blames = match git_blame_parser::parse(&raw_blame) {
        Ok(blames) => blames,
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    };

    for blame in blames.iter() {
        println!(
            "* {}: {:0>4} by {} {}",
            blame.short_commit(),
            blame.original_line_no,
            blame.author,
            blame.author_mail
        );
        println!("summary: {}", blame.summary);
        println!("content: `{}`", blame.content);
        println!();
    }
}
```

See also [examples](./examples/).

## License
[MIT](./LICENSE)
