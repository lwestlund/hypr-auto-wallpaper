use std::{io::Read, str::FromStr, time::Duration};

use anyhow::anyhow;
use chrono::{Local, NaiveTime};
use hyprland::hyprpaper::{self, hyprpaper_async};
use tokio::time;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let entries = parse_wallpaper_config()?;

    let mut current_wallpaper: Option<&String> = None;

    // TODO(lovew): Get from config or a default value as fallback.
    const EVAL_PERIOD: Duration = Duration::from_secs(60 * 5);

    let mut interval = time::interval(EVAL_PERIOD);
    interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);
    loop {
        interval.tick().await;

        let now = Local::now().time();
        println!("now {now}");
        let new_wallpaper = entries
            .iter()
            .cycle()
            .find_map(|e| {
                if now < e.time {
                    Some(&e.wallpaper)
                } else {
                    None
                }
            })
            .expect("Did not find a wallpaper to display at {now}");

        if matches!(current_wallpaper, Some(current_wallpaper) if current_wallpaper == new_wallpaper)
        {
            // Wallpaper should already be the correct one, keep on ticking.
        } else {
            println!("reloading with {new_wallpaper}");
            reload(new_wallpaper).await?;
            current_wallpaper = Some(new_wallpaper);
        }

        // TODO(lovew): Signal handling, listen to SIGTERM?
    }
}

fn parse_wallpaper_config() -> anyhow::Result<Vec<Entry>> {
    let mut config_file = {
        let xdg_dirs = xdg::BaseDirectories::with_prefix("hypr").unwrap();
        let config = xdg_dirs.get_config_file("auto-wallpaper.conf");
        std::fs::File::open(config)?
    };
    let mut contents = {
        let size = config_file
            .metadata()
            .map(|m| m.len() as usize)
            .unwrap_or_default();
        String::with_capacity(size)
    };
    config_file.read_to_string(&mut contents)?;

    let mut entries: Vec<Entry> = contents
        .lines()
        .map(|line| line.parse())
        .collect::<Result<_, _>>()?;
    entries.sort();

    Ok(entries)
}

#[derive(Debug, PartialEq, Eq)]
struct Entry {
    time: NaiveTime,
    wallpaper: String,
}

impl FromStr for Entry {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (time, wallpaper) = s
            .split_once('=')
            .ok_or(anyhow!("Failed to parse entry from '{s}'"))?;
        let time = time.trim();
        let time = NaiveTime::parse_from_str(time, "%H:%M")
            .or_else(|_parse_error| NaiveTime::parse_from_str(time, "%H:%M:%S"))?;

        let wallpaper = wallpaper.trim().to_owned();

        Ok(Self { time, wallpaper })
    }
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Entry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.time.cmp(&other.time)
    }
}

async fn reload(path: &str) -> anyhow::Result<()> {
    let reload = hyprpaper::Keyword::Reload(hyprpaper::Reload {
        monitor: None,
        mode: None,
        path: path.to_owned(),
    });
    hyprpaper_async(reload).await?;
    Ok(())
}
