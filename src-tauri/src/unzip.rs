use std::fs::{read_dir, File};
use std::path::{self, PathBuf};
use std::thread;
use tauri::Emitter;
use unrar::Archive;
use simple_zip::zip::Decompress;

#[tauri::command]
pub fn unzip(
  window: tauri::Window,
  zipfile: String,
  destpath: String,
  top_level: Option<bool>,
  folder_if_loose: Option<bool>,
) {
  // Read file TODO: replace test file
  let f = match File::open(&zipfile) {
    Ok(f) => f,
    Err(e) => {
      println!("Failed to open zip file: {}", e);
      return;
    }
  };

  let write_path = path::PathBuf::from(&destpath);

  // Get a list of all current directories
  let mut dirs = vec![];
  for entry in read_dir(&write_path).unwrap() {
    let entry = entry.unwrap();
    let entry_path = entry.path();
    if entry_path.is_dir() {
      dirs.push(entry_path);
    }
  }

  // Run extraction in seperate thread
  thread::spawn(move || {
    let mut full_path = write_path.clone();

    window.emit("extract_start", &zipfile).unwrap();

    if folder_if_loose.unwrap_or(false) {
      // Create a new folder with the same name as the zip file
      let mut file_name = path::Path::new(&zipfile)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap();

      // remove ".zip" from the end of the file name
      file_name = &file_name[..file_name.len() - 4];

      let new_path = full_path.join(file_name);
      match std::fs::create_dir_all(&new_path) {
        Ok(_) => {}
        Err(e) => {
          println!("Failed to create directory: {}", e);
          return;
        }
      };

      full_path = new_path;
    }

    println!("Is rar file? {}", zipfile.ends_with(".rar"));

    let name;
    let success;

    // If file ends in zip, OR is unknown, extract as zip, otherwise extract as rar
    if zipfile.ends_with(".rar") {
      success = extract_rar(&zipfile, &f, &full_path, top_level.unwrap_or(true));

      let archive = Archive::new(&zipfile);
      name = archive.open_for_listing().unwrap().next().unwrap().unwrap().filename;
    } else if zipfile.ends_with(".7z") {
      success = Ok(extract_7z(&zipfile, &f, &full_path, top_level.unwrap_or(true)));

      name = PathBuf::from("banana");
    } else {
      success = Ok(extract_zip(&full_path));

      // Get the name of the inenr file in the zip file
      let mut zip = zip::ZipArchive::new(&f).unwrap();
      let file = zip.by_index(0).unwrap();
      name = PathBuf::from(file.name());
    }

    if success.unwrap_or(false) {
      let mut res_hash = std::collections::HashMap::new();

      res_hash.insert("path".to_string(), zipfile.to_string());

      window.emit("download_error", &res_hash).unwrap();
    }

    // If the contents is a jar file, emit that we have extracted a new jar file
    if name.to_str().unwrap().ends_with(".jar") {
      window
        .emit("jar_extracted", destpath.to_string() + name.to_str().unwrap())
        .unwrap();
    }

    // If downloading full build, emit that the jar was extracted with it
    if zipfile.contains("GrasscutterCulti") {
      window
        .emit("jar_extracted", destpath.to_string() + "grasscutter.jar")
        .unwrap();
    }

    // Alternate builds
    if zipfile.contains("GrasscutterQuests") || zipfile.contains("Grasscutter50") {
      window
        .emit("jar_extracted", destpath.to_string() + "grasscutter.jar")
        .unwrap();
    }

    if zipfile.contains("GIMI") {
      window
        .emit(
          "migoto_extracted",
          destpath.to_string() + "3DMigoto Loader.exe",
        )
        .unwrap();
    }

    // Delete zip file
    match std::fs::remove_file(&zipfile) {
      Ok(_) => {
        // Get any new directory that could have been created
        let mut new_dir: String = String::new();
        for entry in read_dir(&write_path).unwrap() {
          let entry = entry.unwrap();
          let entry_path = entry.path();
          if entry_path.is_dir() && !dirs.contains(&entry_path) {
            new_dir = entry_path.to_str().unwrap().to_string();
          }
        }

        let mut res_hash = std::collections::HashMap::new();
        res_hash.insert("file", zipfile.to_string());
        res_hash.insert("new_folder", new_dir);

        window.emit("extract_end", &res_hash).unwrap();
        println!("Deleted zip file: {}", zipfile);
      }
      Err(e) => {
        println!("Failed to delete zip file: {}", e);
      }
    };
  });
}

fn extract_rar(rarfile: &str, _f: &File, full_path: &path::Path, _top_level: bool) -> Result<bool, Box<dyn std::error::Error>> {
  let archive = Archive::new(&rarfile);

  let mut open_archive = archive
    .open_for_processing().unwrap();
  while let Some(header) = open_archive.read_header()? {
    open_archive = if header.entry().is_directory() || header.entry().is_file() {
      header.extract_to(full_path.to_str().unwrap().to_string())?
    } else {
      header.skip()?
    }
    
  }
  Ok(true)
}

fn extract_zip(full_path: &path::Path) -> bool {
  Decompress::local_buffer(full_path);
  true
}

fn extract_7z(sevenzfile: &str, _f: &File, full_path: &path::Path, _top_level: bool) -> bool {
  match sevenz_rust::decompress_file(sevenzfile, full_path) {
    Ok(_) => {
      println!(
        "Extracted 7zip file to: {}",
        full_path.to_str().unwrap_or("Error")
      );

      true
    }
    Err(e) => {
      println!("Failed to extract 7zip file: {}", e);
      false
    }
  }
}
