use std::error::Error;
use std::fs::read_dir;
use std::path::Path;
use rusty_audio::Audio;

/// Loads sound files from a specified directory into the Audio object.
///
/// # Arguments
///
/// * `audio` - A mutable reference to the Audio object for adding sounds.
/// * `directory` - The path to the directory containing sound files.
///
/// # Errors
///
/// Returns an error if reading the directory fails or if any file path operation fails.
///
/// # Examples
///
/// ```
/// let mut audio = Audio::new();
/// load_sounds(&mut audio, "./sounds").unwrap();
/// ```
pub fn load_sounds(audio: &mut Audio, directory: &str) -> Result<(), Box<dyn Error>> {
    // Read the directory, filtering for valid file entries with a `.wav` extension
    let sound_files = read_dir(Path::new(directory))?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_file() && entry.path().extension().map_or(false, |ext| ext == "wav"));

    for file in sound_files {
        let file_path = file.path();

        // Extract the sound name from the file path, to use as a key for the audio library
        let sound_name = file_path.file_stem().unwrap().to_str().unwrap();

        // Add the sound to the audio player, using the sound name as the identifier
        audio.add(sound_name, file_path.to_str().unwrap());
    }

    Ok(())
}
