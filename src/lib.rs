use std::{
    env, fs,
    time::{Duration, SystemTime, SystemTimeError},
};

pub fn crawl(config: &Config) -> Result<(), SystemTimeError> {
    let now = SystemTime::now();

    if let Ok(entries) = fs::read_dir(&config.dirname) {
        for entry in entries {
            if let Ok(entry) = entry {
                if entry.path().is_dir() {
                    // thats nasty
                    let subdir = entry
                        .path()
                        .into_os_string()
                        .into_string()
                        .expect("Error when parsing path");

                    crawl(&Config {
                        dirname: subdir,
                        ..*config
                    })?;
                } else {
                    if let Ok(metadata) = entry.metadata() {
                        let file_time = match config.mode {
                            Mode::Accessed => metadata.accessed(),
                            Mode::Modified => metadata.modified(),
                            Mode::Created => metadata.created(),
                        }
                        .unwrap();

                        let cutoff_time: Duration = config.time.into();
                        let duration = match now.duration_since(file_time) {
                            Ok(duration) => duration,
                            Err(err) => return Err(err),
                        };

                        match cutoff_time.cmp(&duration) {
                            std::cmp::Ordering::Less => {
                                println!("{:?}", entry.path())
                            }
                            _ => (),
                        }
                    } else {
                        println!("Couldn't get metadata for {:?}", entry.path());
                    }
                }
            }
        }
    }

    Ok(())
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum Mode {
    Accessed,
    Modified,
    Created,
}

impl TryFrom<String> for Mode {
    type Error = String;
    fn try_from(input: String) -> Result<Self, String> {
        if input.len() > 1 {
            return Err("Multiple modes not supported, provide single mode".into());
        }

        if let Some(ch) = input.to_lowercase().chars().next() {
            return Ok(match ch {
                'a' => Mode::Accessed,
                'm' => Mode::Modified,
                'c' => Mode::Created,
                _ => return Err(format!("Mode not supported, {}", input).into()),
            });
        }

        return Err(format!("Mode not supported, {}", input).into());
    }
}

// pub fn get_modes(input: String) -> Result<HashSet<Mode>, String> {
//     let mut modes = HashSet::<Mode>::with_capacity(3);

//     for ch in input.chars() {
//         match Mode::try_from(ch) {
//             Ok(mode) => modes.insert(mode),
//             Err(err) => return Err(err),
//         };
//     }

//     Ok(modes)
// }

#[derive(Clone, Copy, Debug)]
enum Time {
    Second(u64),
    Minute(u64),
    Hour(u64),
    Day(u64),
    Week(u64),
    Month(u64), // 30 days
    Year(u64),
}

impl Into<Duration> for Time {
    fn into(self) -> Duration {
        return Duration::from_secs(match self {
            Time::Second(n) => n,
            Time::Minute(n) => 60 * n,
            Time::Hour(n) => 3_600 * n,
            Time::Day(n) => 86_400 * n,
            Time::Week(n) => 604_800 * n,
            Time::Month(n) => 2_629_743 * n,
            Time::Year(n) => 31_556_926 * n,
        });
    }
}

impl TryFrom<(char, u64)> for Time {
    type Error = String;
    fn try_from(input: (char, u64)) -> Result<Self, String> {
        return Ok(match input {
            ('s', n) => Time::Second(n),
            ('m', n) => Time::Minute(n),
            ('h', n) => Time::Hour(n),
            ('d', n) => Time::Day(n),
            ('w', n) => Time::Week(n),
            ('M', n) => Time::Month(n),
            ('y', n) => Time::Year(n),
            _ => return Err("Time period not supported".into()),
        });
    }
}

#[derive(Debug)]
pub struct Config {
    dirname: String,
    mode: Mode,
    time: Time,
}

impl Config {
    pub fn new() -> Result<Self, String> {
        let mut args = env::args();

        args.next(); // first arg is exec name

        let dirname = match args.next() {
            Some(val) => val,
            None => return Err("Missing dirname".into()),
        };

        let mode = match args.next() {
            Some(val) => Mode::try_from(val),
            None => return Err("Missing modes".into()),
        }?;

        let time = match args.next() {
            Some(mut val_str) => {
                let period = match val_str.chars().next() {
                    Some(ch) => ch,
                    None => return Err("Provide time period".into()),
                };

                let val = match val_str.split_off(1).parse::<u64>() {
                    Ok(n) => n,
                    Err(_) => return Err("Invalid time".into()),
                };

                match Time::try_from((period, val)) {
                    Ok(time) => time,
                    Err(err) => return Err(err),
                }
            }
            None => return Err("Missing time".into()),
        };

        return Ok(Self {
            dirname,
            mode,
            time,
        });
    }
}
