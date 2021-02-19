use chrono::prelude::*;
use chrono::{DateTime, Utc};

use psutil::process::processes;
use rand::seq::SliceRandom;
use rand::thread_rng;
use skim::prelude::*;
use std::{io::Cursor, ops::Not};
use structopt::StructOpt;
use sysinfo::{ProcessExt, System, SystemExt};
use termion::color;

#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "rkill")]
struct Opt {
    #[structopt(short = "p", long, help = "Process PID in digits")]
    pid: Option<String>,
}

fn main() {
    let opt = Opt::from_args();

    if let Some(pid) = opt.pid {
        let pid = get_pid(pid.into());
        match pid {
            Some(pid) => return info(pid),
            None => return,
        }
    }

    let processes = processes().unwrap();
    let mut ps_names = Vec::new();

    for p in processes {
        let p = p.unwrap();
        let name: String = p.name().unwrap().chars().skip(0).take(23).collect();
        ps_names.push(format!("{:25}{:<2}", name, p.pid().to_string(),));
    }

    ps_names.shuffle(&mut thread_rng());

    let final_names = ps_names.join("\n");

    let options = SkimOptionsBuilder::default()
        .height(Some("70%"))
        .color(Some("molokai"))
        .preview(Some("rkill -p {}"))
        .preview_window(Some("right:60%:wrap"))
        .header(Some("Filter Processes(ctrl+c to exit):"))
        .build()
        .unwrap();
    let item_reader = SkimItemReader::default();
    let items = item_reader.of_bufread(Cursor::new(final_names));
    Skim::run_with(&options, Some(items)).map(|out| match out.final_key {
        Key::Enter => out.selected_items.iter().for_each(|i| stop_process(&i)),
        _ => (),
    });
}

fn stop_process(item: &Arc<dyn SkimItem>) {
    let s = System::new_all();
    let it = item.text();
    let pid = get_pid(it);

    match pid {
        Some(pid) => {
            if let Some(_) = s.get_process(pid) {
                info(pid)
            }
        }
        None => return,
    }

    // if let Some(_process) = s.get_process(pid.unwrap()) {
    //     info(pid);
    // }
}

// take care of this
fn get_pid(it: Cow<str>) -> Option<i32> {
    let item: Vec<String> = it
        .split(" ")
        .filter_map(|s| s.is_empty().not().then(|| s.to_string()))
        .collect();
    dbg!(&item);

    if item.is_empty() {
        return None;
    };

    if item.len() == 1 {
        println!("1 {:?}", item);
        let its = item.iter().nth(0).unwrap().to_string();
        return Some(its.parse().unwrap());
    }
    if item.len() >= 2 {
        println!("2 {:?}", item);

        let pid = item.iter().nth(1).unwrap().to_string();
        let pid = pid.parse().unwrap();
        return Some(pid);
    } else {
        println!("Unable to get");
        return None;
    }
}

fn info(pid: i32) {
    let s = System::new_all();

    if let Some(p) = s.get_process(pid) {
        let time = NaiveDateTime::from_timestamp(p.start_time() as i64, 0);
        let datetime_utc: DateTime<Utc> = DateTime::from_utc(time, Utc);
        let lstart: DateTime<Local> = DateTime::from(datetime_utc);
        let lstart = lstart.format("%a, %b %e %Y %T").to_string();
        println!(
            "{}Name: {} {}",
            color::Fg(color::Green),
            color::Fg(color::LightYellow),
            p.name()
        );
        println!(
            "{}Pid: {} {}",
            color::Fg(color::Green),
            color::Fg(color::LightYellow),
            p.pid()
        );
        println!(
            "{}Status: {} {}",
            color::Fg(color::Green),
            color::Fg(color::LightYellow),
            p.status()
        );
        println!(
            "{}Executable: {} {:?}",
            color::Fg(color::Green),
            color::Fg(color::Yellow),
            p.exe()
        );
        println!(
            "{}Cmd: {} {:?}",
            color::Fg(color::Green),
            color::Fg(color::Yellow),
            p.cmd()
        );
        println!(
            "{}Start Time: {} {:?}",
            color::Fg(color::Green),
            color::Fg(color::Yellow),
            lstart
        );
    }
}
