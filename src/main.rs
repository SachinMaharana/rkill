use anyhow::{format_err, Result};
use chrono::prelude::*;
use chrono::{DateTime, Utc};

use fmt::Debug;
use psutil::process::processes;
use rand::seq::SliceRandom;
use rand::thread_rng;
use skim::prelude::*;
use std::convert::TryFrom;
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
        let pid = get_pid(&pid);
        match pid {
            Some(pid) => return info(pid),
            None => {
                return Err(format_err!("unable to get pid"));
            }
        }
    }

    let processes = processes()?;
    let ps_names = {
        let mut ps_names = Vec::new();
        for p in processes {
            let p = p?;
            // prevent overflow of long process names
            let name: String = p.name()?.chars().skip(0).take(23).collect();
            ps_names.push(format!("{:25}{:<2}", name, p.pid().to_string(),));
        }
        ps_names.shuffle(&mut thread_rng());
        ps_names.join("\n")
    };

    let options = SkimOptionsBuilder::default()
        .height(Some("50%"))
        .color(Some("molokai"))
        .preview(Some("rkill -p {}"))
        .preview_window(Some("right:60%:wrap"))
        .header(Some("Filter Processes(ctrl+c to exit):"))
        .build()
        .unwrap();
    let item_reader = SkimItemReader::default();
    let items = item_reader.of_bufread(Cursor::new(ps_names));

    if let Some(out) = Skim::run_with(&options, Some(items)) {
        if out.final_key == Key::Enter {
            out.selected_items
                .iter()
                .for_each(|i| stop_process(i.text().as_ref()));
        }
    };
    Ok(())
}

fn stop_process(item: &str) {
    let s = System::new_all();
    let pid = get_pid(item);

    match pid {
        Some(pid) => {
            if let Some(process) = s.get_process(pid) {
                process.kill(Signal::Term);
            } else {
                eprintln!("Unable to get process information");
            }
        }
        None => {
            eprintln!("Unable to get pid");
        }
    }
}

fn get_pid(it: &str) -> Option<i32> {
    fn handle_arg(pids: Option<&String>) -> Option<i32> {
        match pids {
            Some(pid) => match pid.parse().ok() {
                Some(pid) => Some(pid),
                None => None,
            },
            None => None,
        }
    }

    let item: Vec<String> = it
        .split(' ')
        .filter_map(|s| s.is_empty().not().then(|| s.to_string()))
        .collect();

    if item.is_empty() {
        return None;
    };

    if item.len() == 1 {
        let pids = item.get(0);
        return handle_arg(pids);
    }
    if item.len() >= 2 {
        let pids = item.get(1);
        return handle_arg(pids);
    }

    None
}

fn highlight<T>(present: &str, msg: T)
where
    T: Debug,
{
    eprintln!(
        "{}{}: {} {:?}",
        color::Fg(color::Green),
        present,
        color::Fg(color::Yellow),
        msg
    );
}

fn get_time(p: &Process) -> Result<String> {
    let time = NaiveDateTime::from_timestamp(i64::try_from(p.start_time())?, 0);
    // show time in utc
    let datetime_utc: DateTime<Utc> = DateTime::from_utc(time, Utc);
    Ok(datetime_utc.to_string())
}

fn info(pid: i32) -> Result<()> {
    let s = System::new_all();
    if let Some(p) = s.get_process(pid) {
        let lstart = get_time(p)?;
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
