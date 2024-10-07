use std::path::Path;

pub fn get_dirname(path: &str) -> Option<&str> {
    if let Some(slash_position) = path.rfind('/') {
        Some(&path[0..slash_position])
    } else {
        None
    }
}

// #todo Consider moving to util, but what if we extract the foreign-library implementation?
// #todo Also support getting the last part of the extension.
// #todo Optimize this.
pub fn get_full_extension(path: impl AsRef<Path>) -> Option<String> {
    let mut file_name = path
        .as_ref()
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    if file_name.starts_with(".") {
        file_name = file_name[1..].to_string();
    }

    file_name
        .find('.')
        .map(|dot_position| file_name[(dot_position + 1)..].to_string())
}
