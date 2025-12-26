use serde::de::DeserializeOwned;
use serde::ser::{self};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, LineWriter, Write},
    path::Path,
};

pub fn write_lines_file(file_path: &Path, lines: Vec<String>) -> io::Result<()> {
    // Create or open (truncate) a file for writing
    // check if parent folder need to be created first
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let file = File::create(file_path)?;
    let mut file = LineWriter::new(file);

    lines
        .iter()
        .for_each(|line| write!(file, "{}", line.as_str()).expect("Cannot write to file"));

    file.flush()?;

    log::debug!("file written {}", file_path.to_str().unwrap());

    Ok(())
}

pub fn append_line_to_file(file_path: &Path, line: &str) -> io::Result<()> {
    // check if parent folder need to be created first
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Open the file in append mode, creating it if it doesn't exist
    let mut file = OpenOptions::new()
        .append(true) // Enable append mode
        .create(true) // Create the file if it doesn't exist
        .open(file_path)?; // Open the file and handle potential errors

    // Write the line to the file
    writeln!(file, "{}", line)?;

    log::debug!(
        "Line appended successfully to {}",
        file_path.to_str().unwrap_or("")
    );

    Ok(())
}

/// tries to read a number from a file.
///
/// # Arguments
/// * `path` - A reference to the path of the file to read.
/// # Returns
///
/// * `Result<usize, Box<dyn Error>>` - The parsed number on success, or a boxed error on failure.
pub fn try_read_number(path: &Path) -> Result<usize, io::Error> {
    // Attempt to read the file's content into a string.
    let content = fs::read_to_string(path)?;
    // The file was read successfully

    log::info!("Successfully read file: {}", path.display());
    // Trim whitespace and parse the content into an usize integer.
    // If parsing fails, return a custom error.
    let parts = content.trim().split(" ");
    for part in parts {
        let number = part.trim().parse::<usize>();
        if let Ok(num) = number {
            return Ok(num);
        }
    }
    Err(io::Error::other("failed reading  last status"))
}

pub fn write_yaml<T>(file_name: &str, value: &T) -> io::Result<()>
where
    T: ?Sized + ser::Serialize,
{
    // check if parent folder need to be created first
    let file_path = Path::new(file_name);
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let f = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(file_path)?;

    serde_yaml::to_writer(f, value).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(())
}

pub fn read_yaml<T>(file_name: &str) -> io::Result<T>
where
    T: DeserializeOwned,
{
    let yaml_content = fs::read_to_string(file_name)?;
    let res: T = serde_yaml::from_str(&yaml_content)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    return Ok(res);
}

pub fn check_or_create_folder(folder_path: String) -> io::Result<()> {
    let p = Path::new(&folder_path);
    if p.exists() {
        log::debug!("Folder '{}' definitely exists.", folder_path);
    } else {
        fs::create_dir_all(&folder_path)?;
        log::warn!(
            "Folder '{}' does not exist (this should not happen after create_dir_all).",
            folder_path
        );
    }
    Ok(())
}
