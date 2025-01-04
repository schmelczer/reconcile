/// Sanitize the document's path to allow all clients to create the same path in
/// their filesystem. If we didn't do this server-side, client's would need to
/// deal with mapping invalid names to valid ones and then back.
pub fn sanitize_path(path: &str) -> String {
    let options = sanitize_filename::Options {
        truncate: true,
        windows: true, // Windows is the lowest common denominator
        replacement: "",
    };

    sanitize_filename::sanitize_with_options(path, options)
}
