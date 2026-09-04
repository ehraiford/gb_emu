use std::{
    ffi::OsStr,
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
    time::SystemTime,
};

use crate::cartridge::save_data::{SaveData, SaveLayout};
pub struct SaveFile {
    save_type: SaveType,
    last_saved: Option<SystemTime>,
}

impl SaveFile {
    pub fn new(save_dir: &Path, rom_stem: &OsStr) -> Result<(Self, Option<SaveData>), SavingError> {
        let save_type = match SaveType::find_existing_save_files(save_dir, rom_stem)? {
            Some(save_type) => save_type,
            None => SaveType::new(save_dir, rom_stem)?,
        };

        let mut this = Self { save_type, last_saved: None };

        let save_data = match this.read_save_file() {
            Ok(save_data) => save_data,
            Err(error) if error.file_not_found() => None, // this'll be the first save to the dir
            Err(error) => return Err(error),
        };

        Ok((this, save_data))
    }

    pub fn layout(&self) -> SaveLayout {
        match self.save_type {
            SaveType::Sav(_) => SaveLayout::Sav,
            SaveType::SrmAndRtc { .. } => SaveLayout::SrmAndRtc,
        }
    }

    pub fn read_save_file(&mut self) -> Result<Option<SaveData>, SavingError> {
        match &self.save_type {
            SaveType::Sav(path_buf) => Ok(Some(SaveData::Sav { data: fs::read(path_buf)? })),
            SaveType::SrmAndRtc { srm, rtc } => Ok(Some(SaveData::SrmAndRtc {
                srm: fs::read(srm)?,
                rtc: rtc.as_ref().and_then(|p| fs::read(p).ok()),
            })),
        }
    }
    pub fn save_data_to_file(&mut self, save_data: SaveData) -> Result<(), SavingError> {
        let mut file_data_pairs = Vec::new();

        // collect all the paths and data
        match (&mut self.save_type, save_data) {
            (SaveType::Sav(path_buf), SaveData::Sav { data }) => file_data_pairs.push((path_buf.clone(), data)),
            (SaveType::SrmAndRtc { srm, rtc }, SaveData::SrmAndRtc { srm: srm_data, rtc: rtc_data }) => {
                file_data_pairs.push((srm.clone(), srm_data));
                match (&rtc, rtc_data) {
                    (Some(path), Some(data)) => file_data_pairs.push((path.clone(), data)), // Save rtc data
                    (None, None) => (), // No rtc data and none expected
                    (Some(_), None) => return Err(SavingError::MissingRtc), // No rtc data but some expected
                    (None, Some(data)) => {
                        // Rtc data but no file on disk.
                        *rtc = Some(srm.with_extension("rtc"));
                        file_data_pairs.push((rtc.clone().unwrap(), data));
                    },
                }
            },
            _ => return Err(SavingError::FileSaveMismatch),
        }

        // go and save the data
        for (path, data) in file_data_pairs {
            let temp_path = path.with_added_extension("tmp");
            fs::write(&temp_path, data)?; // first to temp
            fs::rename(&temp_path, path)?; // then to the actual file
        }

        self.last_saved = Some(SystemTime::now());

        Ok(())
    }
}

enum SaveType {
    Sav(PathBuf),
    SrmAndRtc { srm: PathBuf, rtc: Option<PathBuf> },
}

impl SaveType {
    fn new(save_dir: &Path, rom_stem: &OsStr) -> Result<Self, SavingError> {
        fs::create_dir_all(save_dir)?;

        Ok(SaveType::SrmAndRtc { srm: save_path(save_dir, rom_stem, ".srm"), rtc: None })
    }

    fn find_existing_save_files(save_dir: &Path, rom_stem: &OsStr) -> Result<Option<SaveType>, SavingError> {
        let srm_path = save_path(save_dir, rom_stem, ".srm");
        if srm_path.try_exists()? {
            let rtc_path = save_path(save_dir, rom_stem, ".rtc");

            let rtc = rtc_path.try_exists()?.then_some(rtc_path);

            return Ok(Some(SaveType::SrmAndRtc { srm: srm_path, rtc }));
        }

        let sav_path = save_path(save_dir, rom_stem, ".sav");
        if sav_path.try_exists()? {
            return Ok(Some(SaveType::Sav(sav_path)));
        }

        Ok(None)
    }
}

fn save_path(save_dir: &Path, rom_stem: &OsStr, extension: &str) -> PathBuf {
    let mut file_name = rom_stem.to_os_string();
    file_name.push(extension);

    save_dir.join(file_name)
}

#[derive(Debug)]
pub enum SavingError {
    Io(std::io::Error),
    FileSaveMismatch,
    MissingRtc,
}
impl SavingError {
    fn file_not_found(&self) -> bool {
        if let Self::Io(error) = self {
            error.kind() == ErrorKind::NotFound
        } else {
            false
        }
    }
}
impl std::fmt::Display for SavingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SavingError::Io(error) => write!(f, "{}", error),
            SavingError::FileSaveMismatch => write!(f, "Save data type does not match save file (.srm & .rtc vs. sav"),
            SavingError::MissingRtc => write!(f, "Expected RTC data but didn't receive any"),
        }
    }
}
impl std::error::Error for SavingError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SavingError::Io(error) => Some(error),
            _ => None,
        }
    }
}
impl From<std::io::Error> for SavingError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}
