use std::{
    error::Error,
    fs::{self, File, OpenOptions},
    io::{self, LineWriter, Write},
    path::Path,
};

pub fn write_lines_file(path: &Path, lines: Vec<String>) -> io::Result<()> {
    // Create or open (truncate) a file for writing
    let file = File::create(path)?;
    let mut file = LineWriter::new(file);

    lines
        .iter()
        .map(|line| write!(file, "{}", line.as_str()).expect("Cannot write to file"))
        .collect::<Vec<_>>();

    file.flush()?;

    log::debug!("file written {}", path.to_str().unwrap());

    Ok(())
}

pub fn append_line_to_file(path: &Path, line: &str) -> io::Result<()> {
    // Open the file in append mode, creating it if it doesn't exist
    let mut file = OpenOptions::new()
        .append(true) // Enable append mode
        .create(true) // Create the file if it doesn't exist
        .open(path)?; // Open the file and handle potential errors

    // Write the line to the file
    writeln!(file, "{}", line)?;

    log::debug!(
        "Line appended successfully to {}",
        path.to_str().unwrap_or("")
    );

    Ok(())
}

/// Reads a number from a file.
///
/// If the file or its parent directories do not exist, it creates them
/// and initializes the file with the content "1".
///
/// # Arguments
/// * `path` - A reference to the path of the file to read.
/// # Returns
///
/// * `Result<usize, Box<dyn Error>>` - The parsed number on success, or a boxed error on failure.
pub fn read_number_or_create(path: &Path) -> Result<usize, Box<dyn Error>> {
    // Attempt to read the file's content into a string.
    match fs::read_to_string(path) {
        // The file was read successfully
        Ok(content) => {
            log::info!("Successfully read file: {}", path.display());
            // Trim whitespace and parse the content into an usize integer.
            // If parsing fails, return a custom error.
            let number = content.trim().parse::<usize>().map_err(|e| {
                // TODO: map to error type
                format!(
                    "Could not parse file content '{}' as a number: {}",
                    content.trim(),
                    e
                )
            })?;
            Ok(number)
        }
        //  An error occurred while reading
        Err(error) => {
            // Check if the error was "File Not Found".
            if error.kind() == io::ErrorKind::NotFound {
                log::info!("File not found at {}. Creating it...", path.display());

                // Get the parent directory of the specified path.
                if let Some(parent_dir) = path.parent() {
                    // Create the full directory path if it doesn't exist.
                    // `create_dir_all` is convenient as it won't error if the path already exists.
                    fs::create_dir_all(parent_dir)?;
                    log::info!("Ensured directory exists: {}", parent_dir.display());
                }

                // Create the file and write the default value "1" to it.
                fs::write(path, "1")?;
                log::info!("Created and initialized file with '1'.");

                // Since we just created it with "1", we can return 1.
                Ok(1)
            } else {
                Err(Box::new(error))
            }
        }
    }
}
