use psutil::process::processes;
use rand::seq::SliceRandom;
use rand::thread_rng;
use skim::prelude::*;
use std::{io::Cursor, ops::Not};
use sysinfo::{ProcessExt, Signal, System, SystemExt};

impl SkimItem for MyItem {
    fn text(&self) -> Cow<str> {
        Cow::Borrowed(&self.inner)
    }

    fn preview(&self, _context: PreviewContext) -> ItemPreview {
        if self.inner.starts_with("color") {
            ItemPreview::AnsiText(format!("\x1b[31mhello:\x1b[m\n{}", self.inner))
        } else {
            ItemPreview::Text(format!("hello:\n{}", self.inner))
        }
    }
}
struct MyItem {
    inner: String,
}
fn main() {
    let processes = processes().unwrap();

    let mut ps_names = Vec::new();

    for p in processes {
        let p = p.unwrap();
        let name: String = p.name().unwrap().chars().skip(0).take(25).collect();
        ps_names.push(format!("{:25}{:<2}", name, p.pid().to_string(),));
    }

    ps_names.shuffle(&mut thread_rng());

    let final_names = ps_names.join("\n");

    let options = SkimOptionsBuilder::default()
        .height(Some("30%"))
        .color(Some("molokai"))
        .reverse(true)
        .preview(Some("echo -e test"))
        .header(Some("Filter Processes:"))
        .build()
        .unwrap();
    let item_reader = SkimItemReader::default();
    let items = item_reader.of_bufread(Cursor::new(final_names));
    Skim::run_with(&options, Some(items)).map(|out| match out.final_key {
        Key::Enter => out
            .selected_items
            .iter()
            .for_each(|i| stop_process(&i.text())),
        _ => (),
    });
}

fn stop_process(item: &str) {
    let s = System::new_all();

    let item: Vec<String> = item
        .split(" ")
        .filter_map(|s| s.is_empty().not().then(|| s.to_string()))
        .collect();
    let pid = item.iter().nth(1).unwrap();

    let pid = pid.to_string();
    let pid = pid.parse().unwrap();
    println!("{}", pid);
    if let Some(process) = s.get_process(pid) {
        println!("ehere");
        println!("{:?}", process);
    }
}
