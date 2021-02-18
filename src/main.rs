use psutil::process::processes;
use rand::seq::SliceRandom;
use rand::thread_rng;
use skim::prelude::*;
use std::{env, io::Cursor, ops::Not};
use structopt::StructOpt;
use sysinfo::{ProcessExt, Signal, System, SystemExt};

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    #[structopt(short = "p", long)]
    pid: Option<String>,
}

fn main() {
    let opt = Opt::from_args();
    dbg!(opt);
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
        .preview(Some(
            "echo {} |  sed 's/  */ /g' | cut -d' ' -f2 | xargs -I cmd rkill cmd",
        ))
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

    let item: Vec<String> = it
        .split(" ")
        .filter_map(|s| s.is_empty().not().then(|| s.to_string()))
        .collect();
    let pid = item.iter().nth(1).unwrap();

    let pid = pid.to_string();
    let pid = pid.parse().unwrap();
    println!("{}", pid);
    if let Some(_process) = s.get_process(pid) {
        // println!("{:?}", process);
        info(pid)
    }
}

fn info(pid: i32) {
    let s = System::new_all();

    if let Some(p) = s.get_process(pid) {
        println!("{}", p.name());
        println!("{}", p.status());
        println!("{}", p.start_time());
        println!("{}", p.cpu_usage());
        println!("{:?}", p.disk_usage());
        println!("{:?}", p.parent());
        println!("{:?}", p.exe());
        println!("{:?}", p.cmd());
        println!("{:?}", p.memory());
    }
}

// let args: Vec<String> = env::args().collect();
//     if args.len() > 1 {
//         let q = &args[1].to_string().to_owned();
//         println!("{}", q);
//         let s = q.parse().unwrap();

//         let query = &args[1].is_empty().not();
//         if *query {
//             info(s);
//             return;
//         }
//     }
