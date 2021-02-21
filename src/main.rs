use anyhow::{bail, format_err, Result};
use chrono::prelude::*;
use chrono::{DateTime, Utc};

use fmt::Debug;
use psutil::process::processes;
use rand::seq::SliceRandom;
use rand::thread_rng;
use skim::prelude::*;
use std::fmt;
use std::{io::Cursor, ops::Not};
use structopt::StructOpt;
use sysinfo::{Process, ProcessExt, Signal, System, SystemExt};
use termion::color;

#[derive(StructOpt, Debug, Clone)]
#[structopt(
    name = "rkill",
    about = "interactive cli to kill processes. currently supports linux only."
)]
struct Opt {
    #[structopt(short = "p", long, help = "get process information")]
    pid: Option<String>,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    if let Some(pid) = opt.pid {
        let pid = get_pid(pid.into());
        match pid {
            Some(pid) => return info(pid),
            None => {
                return Err(format_err!("unable to get pid"));
            }
        }
    }

    let processes = processes()?;
    let mut ps_names = Vec::new();

    for p in processes {
        let p = p?;
        // prevent overflow of long process names
        let name: String = p.name()?.chars().skip(0).take(23).collect();
        ps_names.push(format!("{:25}{:<2}", name, p.pid().to_string(),));
    }

    ps_names.shuffle(&mut thread_rng());

    let formatted_ps_names = ps_names.join("\n");

    let options = SkimOptionsBuilder::default()
        .height(Some("50%"))
        .color(Some("molokai"))
        .preview(Some("rkill -p {}"))
        .preview_window(Some("right:60%:wrap"))
        .header(Some("Filter Processes(ctrl+c to exit):"))
        .build()
        .unwrap();
    let item_reader = SkimItemReader::default();
    let items = item_reader.of_bufread(Cursor::new(formatted_ps_names));
    Skim::run_with(&options, Some(items)).map(|out| match out.final_key {
        Key::Enter => out
            .selected_items
            .iter()
            .for_each(|i| stop_process(i.text())),
        _ => (),
    });
    Ok(())
}

fn stop_process(item: Cow<str>) {
    let s = System::new_all();
    let pid = get_pid(item);

    match pid {
        Some(pid) => {
            if let Some(process) = s.get_process(pid) {
                process.kill(Signal::Term);
            } else {
                println!("Unable to get process information");
                return;
            }
        }
        None => {
            println!("Unable to get pid");
            return;
        }
    }
}

fn get_pid(it: Cow<str>) -> Option<i32> {
    let item: Vec<String> = it
        .split(" ")
        .filter_map(|s| s.is_empty().not().then(|| s.to_string()))
        .collect();

    if item.is_empty() {
        return None;
    };

    if item.len() == 1 {
        let pids = item.iter().nth(0);
        return handle_arg(pids);
    }
    if item.len() >= 2 {
        let pids = item.iter().nth(1);
        return handle_arg(pids);
    }

    fn handle_arg(pids: Option<&String>) -> Option<i32> {
        match pids {
            Some(pid) => match pid.parse().ok() {
                Some(pid) => return Some(pid),
                None => return None,
            },
            None => return None,
        }
    };

    None
}

fn highlight<T>(present: &str, msg: T)
where
    T: Debug,
{
    println!(
        "{}{}: {} {:?}",
        color::Fg(color::Green),
        present,
        color::Fg(color::Yellow),
        msg
    );
}

fn get_time(p: &Process) -> String {
    let time = NaiveDateTime::from_timestamp(p.start_time() as i64, 0);
    // show time in utc
    let datetime_utc: DateTime<Utc> = DateTime::from_utc(time, Utc);
    let lstart = datetime_utc.to_string();
    lstart
}

fn info(pid: i32) -> Result<()> {
    let s = System::new_all();
    if let Some(p) = s.get_process(pid) {
        let lstart = get_time(p);
        highlight("Name", p.name());
        highlight("Pid", p.pid());
        highlight("Executable", p.exe());
        highlight("Status", p.status());
        highlight("Cmd", p.cmd());
        highlight("Running Since", lstart);
    } else {
        return Err(format_err!("unable to get process information"));
    }
    Ok(())
}
