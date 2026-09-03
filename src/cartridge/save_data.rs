use crate::cartridge::{cartridge::CartridgeError, memory_bank_controllers::RealTimeClock};

#[derive(Clone, Debug)]
pub enum SaveData {
    // save data where RTC is a separate file
    SrmAndRtc { srm: Vec<u8>, rtc: Option<Vec<u8>> },
    // save data where everything is concatenated into one file
    Sav { data: Vec<u8> },
}
impl SaveData {
    pub fn srm_and_rtc(srm: Vec<u8>, rtc: Option<Vec<u8>>) -> Self {
        Self::SrmAndRtc { srm, rtc }
    }
    pub fn sav(sav: Vec<u8>) -> Self {
        Self::Sav { data: sav }
    }

    pub fn split(&self, expected_ram_len: usize) -> Result<(&[u8], Option<&[u8]>), CartridgeError> {
        match self {
            SaveData::SrmAndRtc { srm, rtc } => Ok((srm, rtc.as_deref())),
            SaveData::Sav { data } => {
                let Some(ram) = data.get(..expected_ram_len) else {
                    return Err(CartridgeError::MisMatchedRamSaveSize(format!(
                        "Save data was {} bytes. Ram is {} bytes.",
                        data.len(),
                        expected_ram_len
                    )));
                };
                let rtc = (data.len() > expected_ram_len).then(|| &data[expected_ram_len..]);
                Ok((ram, rtc))
            },
        }
    }

    pub fn append_ram(&mut self, new_data: &[u8]) {
        let existing_data = match self {
            SaveData::SrmAndRtc { srm, rtc: _ } => srm,
            SaveData::Sav { data } => data,
        };
        existing_data.extend_from_slice(new_data);
    }
    pub fn append_rtc(&mut self, rtc: &RealTimeClock) {
        match self {
            SaveData::SrmAndRtc { srm: _, rtc: data } => *data = Some(rtc.as_rtc_file_timestamp().into()),
            SaveData::Sav { data: existing_data } => existing_data.extend_from_slice(&rtc.as_sav_file_data()),
        }
    }
}

pub struct SaveDataReader<'a> {
    ram_cursor: usize,
    ram: &'a [u8],
    rtc: Option<&'a [u8]>,
    read_rtc: bool,
}
impl<'a> SaveDataReader<'a> {
    pub fn new(save_data: &'a SaveData, expected_ram_len: usize) -> Result<Self, CartridgeError> {
        let (ram, rtc) = save_data.split(expected_ram_len)?;
        Ok(Self { ram_cursor: 0, ram, rtc, read_rtc: false })
    }
    pub fn read_ram(&mut self, num_bytes: usize) -> Option<&[u8]> {
        let end_cursor = self.ram_cursor + num_bytes;
        let slice = self.ram.get(self.ram_cursor..end_cursor);

        // only increment cursor on valid reads
        if slice.is_some() {
            self.ram_cursor = end_cursor
        };
        slice
    }
    pub fn read_rtc(&mut self) -> Option<&'a [u8]> {
        self.read_rtc = true;
        self.rtc
    }

    pub fn has_remaining_data(&self) -> bool {
        self.ram_cursor < self.ram.len() || (self.rtc.is_some() && !self.read_rtc)
    }
}
