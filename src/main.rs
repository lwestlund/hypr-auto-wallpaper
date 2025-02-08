use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use chrono::{Local, NaiveTime};
use hyprland::hyprpaper;
use tokio::time;

const MORNING: NaiveTime = NaiveTime::from_hms_opt(6, 0, 0).unwrap();
const DAY: NaiveTime = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
const EVENING: NaiveTime = NaiveTime::from_hms_opt(16, 0, 0).unwrap();
const NIGHT: NaiveTime = NaiveTime::from_hms_opt(20, 0, 0).unwrap();

const DEFAULT_WALLPAPER_DIR: &str = "~/Pictures/wallpapers";

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let wallpaper_dir: PathBuf = std::env::var("WALLPAPER_DIR")
        .unwrap_or_else(|err| {
            eprintln!("No WALLPAPER_DIR env var set, using default {DEFAULT_WALLPAPER_DIR}");
            DEFAULT_WALLPAPER_DIR.to_string()
        })
        .into();

    // let xdg_runtime_dir = std::env::var("XDG_RUNTIME_DIR").expect("No XDG_RUNTIME_DIR set");
    // let instance_signature = std::env::var("HYPRLAND_INSTANCE_SIGNATURE")
    //     .expect("Could not get HYPRLAND_INSTANCE_SIGNATURE");
    // let hyprland_socket: PathBuf = [
    //     xdg_runtime_dir,
    //     "hypr".into(),
    //     instance_signature,
    //     ".socket.sock".into(),
    // ]
    // .into_iter()
    // .collect();
    // let mut socket = UnixStream::connect(&hyprland_socket).await?;

    // TODO(lovew): Increase this to something bigger or get from env.
    let mut interval = time::interval(Duration::from_secs(5));
    interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);
    loop {
        interval.tick().await;

        let now = Local::now().time();
        println!("now {now}");
        let (old, new) = if now < MORNING {
            println!("now < morning => night");
            (TimeOfDay::Evening, TimeOfDay::Night)
        } else if now < DAY {
            println!("now < day => morning");
            (TimeOfDay::Night, TimeOfDay::Morning)
        } else if now < EVENING {
            println!("now < evening => day");
            (TimeOfDay::Morning, TimeOfDay::Day)
        } else if now < NIGHT {
            println!("now < night => evening");
            (TimeOfDay::Day, TimeOfDay::Evening)
        } else {
            println!("else => night");
            (TimeOfDay::Evening, TimeOfDay::Night)
        };

        let wallpaper = new.as_path();
        println!("wallpaper: {}", wallpaper.display());
        let wallpaper_path = {
            let mut dir = wallpaper_dir.clone();
            dir.push(wallpaper);
            dir
        };

        // preload
        preload(&wallpaper_path).await?;

        // wallpaper
        set_wallpaper(&wallpaper_path).await?;

        // unload
        let old_wallpaper = old.as_path();
        println!("old: {}", old_wallpaper.display());
        let wallpaper_path = {
            let mut dir = wallpaper_dir.clone();
            dir.push(old_wallpaper);
            dir
        };
        if let Err(err) = unload_wallpaper(&wallpaper_path).await {
            eprintln!("Failed to unload {} ({})", wallpaper_path.display(), err);
        }

        // TODO(lovew): Signal handling, listen to SIGTERM.
    }
}

#[derive(Clone, Copy)]
enum TimeOfDay {
    Night,
    Morning,
    Day,
    Evening,
}

impl TimeOfDay {
    fn as_path(&self) -> &Path {
        let p = match self {
            Self::Night => "big-sur-mountains-night-6016x6016.jpg",
            Self::Morning => "big-sur-mountains-morning-6016x6016.jpg",
            Self::Day => "big-sur-mountains-day-6016x6016.jpg",
            Self::Evening => "big-sur-mountains-evening-6016x6016.jpg",
        };
        Path::new(p)
    }
}

async fn preload(path: &Path) -> anyhow::Result<()> {
    let preload = hyprpaper::Preload {
        path: path.to_owned(),
    };
    let command = hyprpaper::Command::Preload(preload);
    hyprland::hyprpaper::hyprpaper_async(command).await?;
    Ok(())
}

async fn set_wallpaper(path: &Path) -> anyhow::Result<()> {
    let wallpaper = hyprpaper::Wallpaper {
        monitor: None,
        mode: None,
        path: path.to_owned(),
    };
    let command = hyprpaper::Command::Wallpaper(wallpaper);
    hyprpaper::hyprpaper_async(command).await?;
    Ok(())
}

async fn unload_wallpaper(path: &Path) -> anyhow::Result<()> {
    let unload = hyprpaper::Unload::Path(path.to_owned());
    let command = hyprpaper::Command::Unload(unload);
    hyprpaper::hyprpaper_async(command).await?;
    Ok(())
}

async fn auto_wallpaper() {}
