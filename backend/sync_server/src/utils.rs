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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_sanitize_path() {
        assert_eq!(sanitize_path("/my/path/what?"), "/my/path/what");
        assert_eq!(sanitize_path("/my/path/\\\\:?"), "/my/path/_");
    }
}
