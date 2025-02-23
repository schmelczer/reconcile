use regex::Regex;

/// Sanitize the document's path to allow all clients to create the same path in
/// their filesystem. If we didn't do this server-side, client's would need to
/// deal with mapping invalid names to valid ones and then back.
pub fn sanitize_path(path: &str) -> String {
    let options = sanitize_filename::Options {
        truncate: true,
        windows: true, // Windows is the lowest common denominator
        replacement: "",
    };

    path.split('/')
        .map(|part| {
            let proposal = sanitize_filename::sanitize_with_options(part, options.clone());
            if !part.is_empty() && proposal.is_empty() {
                "_".to_owned()
            } else {
                proposal
            }
        })
        .collect::<Vec<_>>()
        .join("/")
}

pub fn deduped_file_paths(path: &str) -> impl Iterator<Item = String> {
    let mut path_parts = path.split('/').collect::<Vec<_>>();
    let file_name = path_parts.pop().unwrap().to_owned();

    let mut directory = path_parts.join("/");
    if !directory.is_empty() {
        directory.push('/');
    }

    let name_parts = file_name.rsplitn(2, '.').collect::<Vec<_>>();
    let mut reverse_parts = name_parts.into_iter().rev();
    let (stem, extension) = match (reverse_parts.next(), reverse_parts.next()) {
        (Some(stem), maybe_extension) => (
            stem.to_owned(),
            maybe_extension
                .map(|ext| format!(".{ext}"))
                .unwrap_or_default(),
        ),
        _ => unreachable!("Path must have at least one part"),
    };

    let regex = Regex::new(r" \((\d+)\)$").unwrap();
    let start_number = regex
        .captures(&stem)
        .and_then(|caps| caps.get(1))
        .and_then(|m| m.as_str().parse::<u32>().ok())
        .unwrap_or(0);

    let clean_stem = regex.replace(&stem, "").to_string();

    (start_number..).map(move |dedup_number| {
        if dedup_number == 0 {
            format!("{directory}{clean_stem}{extension}")
        } else {
            format!("{directory}{clean_stem} ({dedup_number}){extension}")
        }
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_sanitize_path() {
        assert_eq!(sanitize_path("/my/path/what?"), "/my/path/what");
        assert_eq!(sanitize_path("/my/path/\\\\:?"), "/my/path/_");
    }

    #[test]
    fn test_deduped_file_paths() {
        let mut deduped = deduped_file_paths("file.txt");
        assert_eq!(deduped.next(), Some("file.txt".to_owned()));
        assert_eq!(deduped.next(), Some("file (1).txt".to_owned()));
        assert_eq!(deduped.next(), Some("file (2).txt".to_owned()));

        let mut deduped = deduped_file_paths("file");
        assert_eq!(deduped.next(), Some("file".to_owned()));
        assert_eq!(deduped.next(), Some("file (1)".to_owned()));
        assert_eq!(deduped.next(), Some("file (2)".to_owned()));

        let mut deduped = deduped_file_paths("file (51).md");
        assert_eq!(deduped.next(), Some("file (51).md".to_owned()));
        assert_eq!(deduped.next(), Some("file (52).md".to_owned()));
        assert_eq!(deduped.next(), Some("file (53).md".to_owned()));

        let mut deduped = deduped_file_paths("file (5)");
        assert_eq!(deduped.next(), Some("file (5)".to_owned()));
        assert_eq!(deduped.next(), Some("file (6)".to_owned()));
        assert_eq!(deduped.next(), Some("file (7)".to_owned()));

        let mut deduped = deduped_file_paths("my/path.with.dots/file (5).md");
        assert_eq!(
            deduped.next(),
            Some("my/path.with.dots/file (5).md".to_owned())
        );
        assert_eq!(
            deduped.next(),
            Some("my/path.with.dots/file (6).md".to_owned())
        );

        let mut deduped = deduped_file_paths("my/path.with.dots/file (5)");
        assert_eq!(
            deduped.next(),
            Some("my/path.with.dots/file (5)".to_owned())
        );
        assert_eq!(
            deduped.next(),
            Some("my/path.with.dots/file (6)".to_owned())
        );
    }
}
