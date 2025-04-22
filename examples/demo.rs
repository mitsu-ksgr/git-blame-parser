//!
//! git-blame-parser example
//!

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Error: missing args <FILE_PATH>");
        std::process::exit(1);
    }

    let filepath = std::path::PathBuf::from(args[1].clone());
    if !filepath.is_file() {
        eprintln!("Error: invalid file path");
        std::process::exit(1);
    }

    let output = std::process::Command::new("git")
        .args(["blame", "--line-porcelain"])
        .arg(filepath)
        .output()
        .unwrap();

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        eprintln!("Error: {err}");
        std::process::exit(1);
    }

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
        println!("");
    }
}
