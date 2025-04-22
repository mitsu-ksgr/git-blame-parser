//!
//! git-blame-parser
//!

/// The porcelain format parser error
#[derive(Debug, Clone)]
pub struct ParseError(String);

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Error parsing git-blame: {}", self.0)
    }
}

impl std::error::Error for ParseError {}

/// The blame information
///
/// This struct stores the `git blame` output in PORCELAIN format.
///
/// ## The Porcelain Format
/// The porcelain format is the output format produced by `git blame` when
/// the `--porcelain` option is used.
///
/// Note that the `--porcelain` option generally suppresses commit information
/// that has already been seen.
///
/// The `--line-porcelain` option can be used to outputs the full commit
/// information for each line, use this when parsing.
/// so use this for parsing.
///
/// More information about the porcelain format can be
/// For more information, see [git doc](https://git-scm.com/docs/git-blame#_the_porcelain_format).
///
/// ## time
/// `author_time` and `committer_time` are UNIX times (seconds).
///
/// ## boundary
/// `boundary` is a metadata indicating the commit where the history tracking
/// is stopped in git blame. This line means that the history will not be followed
/// this point.
///
/// It is output only when necessary, and in that case, it is set to `true`.
#[derive(Debug, Default)]
pub struct Blame {
    pub commit: String,
    pub original_line_no: usize,
    pub final_line_no: usize,

    pub filename: String,
    pub summary: String,

    /// The contents of the actual line
    pub content: String,

    // previous
    pub previous_commit: Option<String>,
    pub previous_filepath: Option<String>,

    /// Set to true when blame output contains `boundary`.
    pub boundary: bool,

    pub author: String,
    pub author_mail: String,
    pub author_time: u64,
    pub author_tz: String,

    pub committer: String,
    pub committer_mail: String,
    pub committer_time: u64,
    pub committer_tz: String,
}

impl Blame {
    /// Returns the abbreviated (short-hand) version of the commit hash.
    pub fn short_commit(&self) -> String {
        self.commit[..7.min(self.commit.len())].to_string()
    }
}

/// Parses the porcelain format output corresponding to a single line to
/// construct a Blame object.
pub fn parse_one_blame(porcelain: &[&str]) -> Result<Blame, ParseError> {
    let mut blame = Blame::default();

    // Parse header
    if let Some(header) = porcelain.get(0) {
        let parts: Vec<&str> = header.split_whitespace().collect();
        blame.commit = parts[0].to_string();

        if let Some(lineno) = parts.get(1) {
            blame.original_line_no = lineno.parse::<usize>().unwrap_or(0);
        }
        if let Some(lineno) = parts.get(2) {
            blame.final_line_no = lineno.parse::<usize>().unwrap_or(0);
        }
    } else {
        return Err(ParseError("no header".to_string()));
    }

    // Parse details
    for line in porcelain.iter().skip(1) {
        if line.starts_with('\t') {
            let src = line.strip_prefix('\t').unwrap_or(line);
            blame.content = src.to_string();
        } else {
            match line.split_once(' ') {
                Some(("filename", value)) => blame.filename = value.to_string(),
                Some(("summary", value)) => blame.summary = value.to_string(),

                Some(("author", value)) => blame.author = value.to_string(),
                Some(("author-mail", value)) => blame.author_mail = value.to_string(),
                Some(("author-time", value)) => {
                    blame.author_time = value.parse::<u64>().unwrap_or(0)
                }
                Some(("author-tz", value)) => blame.author_tz = value.to_string(),

                Some(("committer", value)) => blame.committer = value.to_string(),
                Some(("committer-mail", value)) => blame.committer_mail = value.to_string(),
                Some(("committer-time", value)) => {
                    blame.committer_time = value.parse::<u64>().unwrap_or(0)
                }
                Some(("committer-tz", value)) => blame.committer_tz = value.to_string(),

                Some(("previous", value)) => {
                    if let Some((commit, filepath)) = value.split_once(' ') {
                        blame.previous_commit = Some(commit.to_string());
                        blame.previous_filepath = Some(filepath.to_string());
                    }
                }

                None => match *line {
                    "boundary" => blame.boundary = true,
                    _ => continue,
                },

                _ => continue,
            }
        }
    }

    Ok(blame)
}

/// Parses the output of `git blame` command in the porcelain format.
/// the output must be generated using the `--line-porcelain` option.
pub fn parse(porcelain: &str) -> Result<Vec<Blame>, ParseError> {
    let mut lines = porcelain.lines();
    let mut blames = Vec::new();

    let mut blob: Vec<&str> = Vec::new();
    while let Some(line) = lines.next() {
        blob.push(line);

        // end of one blame output.
        if line.starts_with('\t') {
            match parse_one_blame(&blob) {
                Ok(blame) => blames.push(blame),
                Err(e) => return Err(e),
            }

            blob.clear();
        }
    }

    Ok(blames)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let path = std::path::PathBuf::from("tests").join("sample-blame.txt");
        let raw_blame = std::fs::read_to_string(path).unwrap();

        let blames = parse(&raw_blame);
        assert!(blames.is_ok());

        let blames = blames.unwrap();
        assert_eq!(blames.len(), 43);

        let first = blames.first().unwrap();

        assert_eq!(first.commit, "c9a79e91e05355fc42ec519593806466c2f66de0");
        assert_eq!(first.original_line_no, 1);
        assert_eq!(first.final_line_no, 1);
        assert_eq!(first.boundary, false);

        assert_eq!(first.filename, "README.md");
        assert_eq!(first.summary, "Update README.md");
        assert_eq!(first.content, "<div align=\"center\">");

        assert_eq!(first.author, "mitsu-ksgr");
        assert_eq!(first.author_mail, "<mitsu-ksgr@users.noreply.github.com>");
        assert_eq!(first.author_time, 1744981061);
        assert_eq!(first.author_tz, "+0900");

        assert_eq!(first.committer, "GitHub");
        assert_eq!(first.committer_mail, "<noreply@github.com>");
        assert_eq!(first.committer_time, 1744981061);
        assert_eq!(first.committer_tz, "+0900");

        assert!(first.previous_commit.is_some());
        assert!(first.previous_filepath.is_some());

        let prev_commit = first.previous_commit.as_ref().unwrap();
        let prev_filepath = first.previous_filepath.as_ref().unwrap();
        assert_eq!(prev_commit, "5d31b11bd146562bb1b472e1334233a6a8ef66e5");
        assert_eq!(prev_filepath, "README.md");
    }

    #[test]
    fn one_line_blame() {
        let path = std::path::PathBuf::from("tests").join("one-line-blame.txt");
        let raw_blame = std::fs::read_to_string(path).unwrap();

        let blames = parse(&raw_blame);
        assert!(blames.is_ok());

        let blames = blames.unwrap();
        assert_eq!(blames.len(), 1);

        let first = blames.first().unwrap();
        assert_eq!(first.commit, "6cebf082a694d9dec6c1928531fcb649791885ec");
        assert_eq!(first.original_line_no, 1);
        assert_eq!(first.final_line_no, 1);
        assert_eq!(first.boundary, true);
        assert_eq!(first.summary, "Initial commit");
        assert_eq!(first.content, "# git-blame-parser");
    }

    #[test]
    fn no_commited_yet() {
        let path = std::path::PathBuf::from("tests").join("no-committed.txt");
        let raw_blame = std::fs::read_to_string(path).unwrap();

        let blames = parse(&raw_blame);
        assert!(blames.is_ok());

        let blames = blames.unwrap();
        let first = blames.first().unwrap();
        assert_eq!(first.commit, "0000000000000000000000000000000000000000");
        assert_eq!(first.author, "Not Committed Yet");
        assert_eq!(first.author_mail, "<not.committed.yet>");
        assert_eq!(first.committer, "Not Committed Yet");
        assert_eq!(first.committer_mail, "<not.committed.yet>");
    }

    #[test]
    fn test_shor_commit() {
        let mut blame = Blame::default();
        blame.commit = String::from("abcdefghijklmnopqrstuvwxyz1234567890abcd");

        assert_eq!(blame.short_commit(), "abcdefg");
    }
}
