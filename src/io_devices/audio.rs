use crate::bus::{Address, BusAccessFailure};

#[derive(Default)]
pub struct Audio {
    is_enabled: bool,
    master_volume_and_vin_panning: MasterVolumeAndVinPanning,
    channel_1: Channel1Or2,
    channel_2: Channel1Or2,
    channel_3: Channel3,
    channel_4: Channel4,
}

impl Audio {
    pub fn read(&self, address: Address) -> u8 {
        let Ok(register) = Registers::try_from(address) else {
            return BusAccessFailure::NothingMappedToAddress.into();
        };
        match register {
            Registers::AudioMasterControl => self.read_audio_master_control_reg(),
            Registers::SoundPanning => BusAccessFailure::TriedReadingWriteOnlyMemory.into(),
            Registers::MasterVolumeAndVinPanning => self.master_volume_and_vin_panning.read(),
            Registers::Channel1Sweep => self.channel_1.sweep.read(),
            Registers::Channel1LengthTimerAndDutyCycle => self.channel_1.length_timer_and_duty_cycle.read(),
            Registers::Channel1VolumeAndEnvelope => self.channel_1.volume_and_envelope.read(),
            Registers::Channel1PeriodLow => self.channel_1.read_period_low(),
            Registers::Channel1PeriodHighAndControl => self.channel_1.read_period_high_and_control(),
            Registers::Channel2LengthTimerAndDutyCycle => self.channel_2.length_timer_and_duty_cycle.read(),
            Registers::Channel2VolumeAndEnvelope => self.channel_2.volume_and_envelope.read(),
            Registers::Channel2PeriodLow => self.channel_2.read_period_low(),
            Registers::Channel2PeriodHighAndControl => self.channel_2.read_period_high_and_control(),
            Registers::Channel3DacEnable => self.channel_3.read_dac_enable(),
            Registers::Channel3LengthTimer => self.channel_3.read_length_timer(),
            Registers::Channel3OutputLevel => self.channel_3.read_output_level(),
            Registers::Channel3PeriodLow => self.channel_3.read_period_low(),
            Registers::Channel3PeriodHighAndControl => self.channel_3.read_period_high_and_control(),
            Registers::WavePatternRam { offset: _offset } => BusAccessFailure::Unimplemented.into(),
            Registers::Channel4LengthTimer => self.channel_4.read_length_timer(),
            Registers::Channel4VolumeAndEnvelope => self.channel_4.volume_and_envelope.read(),
            Registers::Channel4FrequencyAndRandomness => self.channel_4.frequency_and_randomness.read(),
            Registers::Channel4Control => self.channel_4.read_period_high_and_control(),
        }
    }
    pub fn write(&mut self, address: Address, value: u8) {
        let Ok(register) = Registers::try_from(address) else {
            return;
        };
        match register {
            Registers::AudioMasterControl => self.is_enabled = value & 0b10000000 != 0,
            Registers::SoundPanning => self.write_sound_panning(value),
            Registers::MasterVolumeAndVinPanning => self.master_volume_and_vin_panning.write(value),
            Registers::Channel1Sweep => self.channel_1.sweep.write(value),
            Registers::Channel1LengthTimerAndDutyCycle => self.channel_1.length_timer_and_duty_cycle.write(value),
            Registers::Channel1VolumeAndEnvelope => self.channel_1.volume_and_envelope.write(value),
            Registers::Channel1PeriodLow => self.channel_1.write_period_low(value),
            Registers::Channel1PeriodHighAndControl => self.channel_1.write_period_high_and_control(value),
            Registers::Channel2LengthTimerAndDutyCycle => self.channel_2.length_timer_and_duty_cycle.write(value),
            Registers::Channel2VolumeAndEnvelope => self.channel_2.volume_and_envelope.write(value),
            Registers::Channel2PeriodLow => self.channel_2.write_period_low(value),
            Registers::Channel2PeriodHighAndControl => self.channel_2.write_period_high_and_control(value),
            Registers::Channel3DacEnable => self.channel_3.write_dac_enable(value),
            Registers::Channel3LengthTimer => self.channel_3.write_length_timer(value),
            Registers::Channel3OutputLevel => self.channel_3.write_output_level(value),
            Registers::Channel3PeriodLow => self.channel_3.write_period_low(value),
            Registers::Channel3PeriodHighAndControl => self.channel_3.write_period_high_and_control(value),
            Registers::WavePatternRam { offset: _offset } => (),
            Registers::Channel4LengthTimer => self.channel_4.write_length_timer(value),
            Registers::Channel4VolumeAndEnvelope => self.channel_4.volume_and_envelope.write(value),
            Registers::Channel4FrequencyAndRandomness => self.channel_4.frequency_and_randomness.write(value),
            Registers::Channel4Control => self.channel_4.write_period_high_and_control(value),
        }
    }

    fn read_audio_master_control_reg(&self) -> u8 {
        let mut register = (self.is_enabled as u8) << 7;

        register |= self.channel_1.is_on() as u8;
        register |= (self.channel_2.is_on() as u8) << 1;
        register |= (self.channel_3.is_on() as u8) << 2;
        register |= (self.channel_4.is_on() as u8) << 3;

        register
    }

    fn write_sound_panning(&mut self, value: u8) {
        if value & 0b1 == 1 {
            self.channel_1.pan(Pan::Right)
        }
        if (value >> 1) & 0b1 == 1 {
            self.channel_2.pan(Pan::Right)
        }
        if (value >> 2) & 0b1 == 1 {
            self.channel_3.pan(Pan::Right)
        }
        if (value >> 3) & 0b1 == 1 {
            self.channel_4.pan(Pan::Right)
        }
        if (value >> 4) & 0b1 == 1 {
            self.channel_1.pan(Pan::Left)
        }
        if (value >> 5) & 0b1 == 1 {
            self.channel_2.pan(Pan::Left)
        }
        if (value >> 6) & 0b1 == 1 {
            self.channel_3.pan(Pan::Left)
        }
        if (value >> 7) & 0b1 == 1 {
            self.channel_4.pan(Pan::Left)
        }
    }
}

enum Registers {
    AudioMasterControl,
    SoundPanning,
    MasterVolumeAndVinPanning,
    Channel1Sweep,
    Channel1LengthTimerAndDutyCycle,
    Channel1VolumeAndEnvelope,
    Channel1PeriodLow,
    Channel1PeriodHighAndControl,
    Channel2LengthTimerAndDutyCycle,
    Channel2VolumeAndEnvelope,
    Channel2PeriodLow,
    Channel2PeriodHighAndControl,
    Channel3DacEnable,
    Channel3LengthTimer,
    Channel3OutputLevel,
    Channel3PeriodLow,
    Channel3PeriodHighAndControl,
    WavePatternRam { offset: u8 },
    Channel4LengthTimer,
    Channel4VolumeAndEnvelope,
    Channel4FrequencyAndRandomness,
    Channel4Control,
}

impl TryFrom<Address> for Registers {
    type Error = String;

    fn try_from(value: Address) -> Result<Self, Self::Error> {
        match value {
            0xFF26 => Ok(Self::AudioMasterControl),
            0xFF25 => Ok(Self::SoundPanning),
            0xFF24 => Ok(Self::MasterVolumeAndVinPanning),
            0xFF10 => Ok(Self::Channel1Sweep),
            0xFF11 => Ok(Self::Channel1LengthTimerAndDutyCycle),
            0xFF12 => Ok(Self::Channel1VolumeAndEnvelope),
            0xFF13 => Ok(Self::Channel1PeriodLow),
            0xFF14 => Ok(Self::Channel1PeriodHighAndControl),
            0xFF16 => Ok(Self::Channel2LengthTimerAndDutyCycle),
            0xFF17 => Ok(Self::Channel2VolumeAndEnvelope),
            0xFF18 => Ok(Self::Channel2PeriodLow),
            0xFF19 => Ok(Self::Channel2PeriodHighAndControl),
            0xFF1A => Ok(Self::Channel3DacEnable),
            0xFF1B => Ok(Self::Channel3LengthTimer),
            0xFF1C => Ok(Self::Channel3OutputLevel),
            0xFF1D => Ok(Self::Channel3PeriodLow),
            0xFF1E => Ok(Self::Channel3PeriodHighAndControl),
            0xFF30..0xFF40 => Ok(Self::WavePatternRam { offset: (value - 0xFF30) as u8 }),
            0xFF20 => Ok(Self::Channel4LengthTimer),
            0xFF21 => Ok(Self::Channel4VolumeAndEnvelope),
            0xFF22 => Ok(Self::Channel4FrequencyAndRandomness),
            0xFF23 => Ok(Self::Channel4Control),
            _ => Err(format!("Address (0x{value:08x}) is not an audio address.")),
        }
    }
}

#[derive(Default)]
struct MasterVolumeAndVinPanning {
    left_vin: bool,
    right_vin: bool,
    left_vol: u8,
    right_vol: u8,
}

impl MasterVolumeAndVinPanning {
    fn read(&self) -> u8 {
        let mut register = (self.left_vin as u8) << 7;

        register |= self.left_vol << 4;
        register |= (self.right_vin as u8) << 3;
        register |= self.right_vol;

        register
    }
    fn write(&mut self, value: u8) {
        self.left_vin = value & 0b10000000 != 0;
        self.left_vol = (value >> 4) & 0b111;
        self.right_vin = value & 0b00001000 != 0;
        self.right_vol = value & 0b111;
    }
}

/// Channel 2 doesn't have sweep but is identical otherwise.
/// Instead of having a second structure, we're just reusing this one and never mapping access to sweep
#[derive(Default)]
struct Channel1Or2 {
    
    sweep: Channel1Sweep, 
    length_timer_and_duty_cycle: Channel1Or2LengthTimerAndDutyCycle,
    volume_and_envelope: VolumeAndEnvelope,
    period: u16,
    length_enabled: bool,
}

impl Channel1Or2 {
}

impl Channel for Channel1Or2 {
    fn trigger(&mut self) {
        
    }

    fn access_period(&mut self) -> Option<&mut u16> {
       Some(&mut self.period)
    }

    fn update_length_enabled(&mut self, value: bool) {
        self.length_enabled = value
    }

    fn get_length_enable(&self) -> bool {
        self.length_enabled
    }
}

#[derive(Default)]
struct Channel1Sweep {
    pace: u8,
    direction: bool,
    step: u8,
}

impl Channel1Sweep {
    #[rustfmt::skip]
    fn read(&self) -> u8 {
        ((self.pace & 0b111) << 4) | 
        ((self.direction as u8) << 3) | 
        (self.step & 0b111)
    }
    fn write(&mut self, value: u8) {
        self.pace = (value >> 4) & 0b111;
        self.direction = value & 0b00001000 != 0;
        self.step = value & 0b111;
    }
}

#[derive(Default)]
struct Channel1Or2LengthTimerAndDutyCycle {
    duty_cycle: DutyCycle,
    length_timer: u8,
}

impl Channel1Or2LengthTimerAndDutyCycle {
    #[rustfmt::skip]
    fn read(&self) -> u8 {
        (Into::<u8>::into(&self.duty_cycle) << 6) | 
        (self.length_timer & 0b11111)
    }
    fn write(&mut self, value: u8) {
        self.duty_cycle = DutyCycle::try_from(value >> 6).expect("Adjusted Bitwise");
        self.length_timer = value & 0b11111;
    }
}

#[derive(Default)]
struct VolumeAndEnvelope {
    volume: u8,
    env_dir: bool,
    sweep_pace: u8,
}
impl VolumeAndEnvelope {
    #[rustfmt::skip]
    fn read(&self) -> u8 {
        (self.volume << 4) |
        ((self.env_dir as u8) << 3) | 
        (self.sweep_pace & 0b111)
    }
    fn write(&mut self, value: u8) {
        self.volume = value >> 4;
        self.env_dir = value & 0b00001000 != 0;
        self.sweep_pace = value & 0b111;
    }
}
#[derive(Default)]
struct Channel3 {
    dac_enable: bool,
    length_timer: u8,
    output_level: OutputLevel,
    period: u16,
    length_enabled: bool,
}

impl Channel3 {
    fn write_dac_enable(&mut self, value: u8) {
        self.dac_enable = value & 0b10000000 != 0;
    }
    fn read_dac_enable(&self) -> u8 {
        (self.dac_enable as u8 ) << 7
    }
    fn write_length_timer(&mut self, value: u8) {
        self.length_timer = value
    }
    fn read_length_timer(&self) -> u8 {
        self.length_timer
    }
    fn write_output_level(&mut self, value: u8) {
        self.output_level = ((value >> 5) & 0b11).try_into().expect("Adjusted Bitwise");
    }
    fn read_output_level(&self) -> u8 {
        u8::from(&self.output_level) << 5
    }

}

impl Channel for Channel3 {
    fn trigger(&mut self) {
        
    }

    fn access_period(&mut self) -> Option<&mut u16> {
       Some(&mut self.period)
    }

    fn update_length_enabled(&mut self, value: bool) {
        self.length_enabled = value
    }

    fn get_length_enable(&self) -> bool {
        self.length_enabled
    }
}
#[derive(Default)]
struct Channel4 {
    length_enabled: bool,
    length_timer: u8,
    volume_and_envelope: VolumeAndEnvelope,
    frequency_and_randomness: FrequencyAndRandomness,
}

impl Channel4 {
    fn write_length_timer(&mut self, value: u8) {
        self.length_timer = value
    }
    fn read_length_timer(&self) -> u8 {
        self.length_timer
    }
}

impl Channel for Channel4 {
    fn trigger(&mut self) {
        
    }

    fn access_period(&mut self) -> Option<&mut u16> {
        None    
    }

    fn update_length_enabled(&mut self, value: bool) {
        self.length_enabled = value
    }

    fn get_length_enable(&self) -> bool {
        self.length_enabled
    }
}

#[derive(Default)]
struct FrequencyAndRandomness {
    clock_shift: u8,
    lfsr_width: bool,
    clock_divider: u8,
}
impl FrequencyAndRandomness {
    #[rustfmt::skip]
    fn read(&self) -> u8 {
        (self.clock_shift << 4) |
        ((self.lfsr_width as u8) << 3) |
        (self.clock_divider & 0b111)
    }
    fn write(&mut self, value: u8) {
        self.clock_shift = value >> 4;
        self.lfsr_width = value & 0b0000_1000 != 0;
        self.clock_divider = value & 0b111;
    }
}

trait Channel {
    fn is_on(&self) -> bool {
        false
    }
    fn pan(&mut self, _pan: Pan) {
        
    }

    fn trigger(&mut self);
    fn access_period(&mut self) -> Option<&mut u16>;
    fn update_length_enabled(&mut self, value: bool);
    fn get_length_enable(&self) -> bool;

    fn write_period_low(&mut self, value: u8) {
        let Some(period) = self.access_period() else {
            return;
        };
        *period &= 0xFF00;
        *period |= value as u16;
    }
    fn write_period_high_and_control(&mut self, value: u8) {
        if let Some(period) = self.access_period() {
            *period &= 0x00FF;
            *period |= ((value & 0b111) as u16) << 8;
        }

        self.update_length_enabled(value & 0b0100_0000 != 0);
        
        if value & 0b1000_0000 != 0 {
            self.trigger();
        }
    }
    fn read_period_low(&self) -> u8 {
        BusAccessFailure::TriedReadingWriteOnlyMemory.into()
    }
    fn read_period_high_and_control(&self) -> u8 {
        (self.get_length_enable() as u8) << 6
    }
}

#[derive(Default, Copy, Clone)]
enum DutyCycle {
    #[default]
    Eigth,
    Quarter,
    Half,
    ThreeQuarters,
}

impl From<&DutyCycle> for u8 {
    fn from(value: &DutyCycle) -> Self {
        match value {
            DutyCycle::Eigth => 0,
            DutyCycle::Quarter => 1,
            DutyCycle::Half => 2,
            DutyCycle::ThreeQuarters => 3,
        }
    }
}
impl TryFrom<u8> for DutyCycle {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(DutyCycle::Eigth),
            1 => Ok(DutyCycle::Quarter),
            2 => Ok(DutyCycle::Half),
            3 => Ok(DutyCycle::ThreeQuarters),
            _ => Err(format!("Valid values are 0..=3. Received: {value}")),
        }
    }
}

enum Pan {
    Left,
    Right,
}
#[derive(Default)]
enum OutputLevel {
    #[default]
    Mute,
    Full,
    Half,
    Quarter,
}

impl From<&OutputLevel> for u8 {
    fn from(value: &OutputLevel) -> Self {
        match value {
            OutputLevel::Mute => 0, 
            OutputLevel::Full => 1,
            OutputLevel::Half => 2,
            OutputLevel::Quarter => 3,
        }
    }
}

impl TryFrom<u8> for OutputLevel {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Mute), 
            1 => Ok(Self::Full),
            2 => Ok(Self::Half),
            3 => Ok(Self::Quarter),
            _ => Err(format!("Valid values are 0..=3. Received: {value}")),
        }
    }
}