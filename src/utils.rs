use base64::prelude::*;
use std::{
    env,
    fs::{self, File},
    io::Read,
    path::PathBuf,
    process::Command,
};
use uuid::Uuid;

pub fn generate_unique_filename(extension: &str) -> PathBuf {
    let uuid = Uuid::new_v4();

    let filename = format!("{}.{}", uuid, extension);

    PathBuf::from(filename)
}

pub fn get_root_folder() -> PathBuf {
    /*
    let flatpak_id = env::var("FLATPAK_ID").expect("FLATPAK_ID environment variable not set");

    let root_dir = env::var("HOME")
        .expect("HOME environment variable not set")
        .to_owned()
        + "/.var/app/"
        + &flatpak_id;
    PathBuf::from(root_dir)
    */
    let root_dir = "";
    PathBuf::from(root_dir)
}

pub fn get_filenames_from_folder(folder: PathBuf) -> Vec<PathBuf> {
    let folder = get_root_folder().join(folder);
    if let Ok(read_dir) = fs::read_dir(&folder) {
        read_dir
            .map(|dir_entry| dir_entry.unwrap().path())
            .collect()
    } else {
        println!("Creating folder: {:?}", folder);
        fs::create_dir(folder).unwrap();
        vec![]
    }
}

pub fn run_bash_search_script(script: PathBuf, prompt: &str) -> String {
    let script = get_root_folder().join(script);
    let output = Command::new(script)
        .arg(prompt)
        .output()
        .expect("Failed to execute command")
        .stdout;
    String::from_utf8(output).unwrap_or(String::from("error converting output to string"))
}

pub fn pdf_to_string(file_path: &PathBuf) -> String {
    String::from("")
}

pub fn image_to_b64(file_path: &PathBuf) -> String {
    let mut image_file = File::open(file_path).expect("Failed to open file");
    let mut buffer = Vec::new();
    image_file
        .read_to_end(&mut buffer)
        .expect("Failed to read file");
    BASE64_STANDARD.encode(buffer)
}
