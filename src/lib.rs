use rayon::ThreadPoolBuilder;
use std::sync::mpsc::channel;
use scraper::{ElementRef, Html, Selector};
use reqwest::blocking;
use regex::Regex;
use std::collections::BTreeMap;
use port_scanner::scan_port_addr;
use std::time::{Duration, Instant};
use std::thread;
use ctrlc;
use std::process;
use std::sync::{Arc,Mutex};

#[derive(Debug)]
pub struct Threads {
    pub thread_name: String,
    pub cumulative_user_cpu_s: String,
    pub cumulative_kernel_cpu_s: String,
    pub cumulative_iowait_cpu_s: String,
    pub stack: String,
}

pub fn sample_servers(
    hostname_vec: &Vec<&str>,
    port_vec: &Vec<&str>,
    update_interval: u64,
) {
    let mut threads: Arc<Mutex<BTreeMap<String, u64>>> = Default::default();

    {
        let threads = Arc::clone(&threads);
        ctrlc::set_handler(move || {
            //println!("{:#?}", &*(threads.lock().unwrap()));
            for (stack, nr) in &*(threads.lock().unwrap()) {
                println!("{} {}", stack, nr);
            };
            process::exit(0);
        }).expect("Error setting ctrl-c handler");
    }

    //loop {
    for _ in 0..20 {
        let wait_time_ms = Duration::from_millis(update_interval);
        let start_time = Instant::now();
        perform_threads_snapshot(&hostname_vec, &port_vec, 1, &mut threads );
        let time_to_wait= wait_time_ms.checked_sub(start_time.elapsed()).unwrap_or(wait_time_ms);
        //print!(".");
        thread::sleep( time_to_wait);
    }

    for (stack, nr) in &*(threads.lock().unwrap()) {
        println!("{} {}", stack, nr);
    }
    //println!("{:#?}", threads);
}

fn perform_threads_snapshot(
    hostname_vec: &Vec<&str>,
    port_vec: &Vec<&str>,
    parallel: usize,
    threads_hashmap: &mut Arc<Mutex<BTreeMap<String, u64>>>,
) {
    let pool = ThreadPoolBuilder::new().num_threads(parallel).build().unwrap();
    let (tx, rx) = channel();

    pool.scope(move |s| {
        for hostname in hostname_vec {
            for port in port_vec {
                let tx = tx.clone();
                s.spawn(move |_| {
                    let threads = read_threads(format!("{}:{}", &hostname, &port).as_str());
                    tx.send((format!("{}:{}", hostname, port), threads)).expect("failed sending to channel");
                });
            }
        }
    });

    for (hostname_port, threads) in rx {
        for thread in threads {
            let key=format!("{};{}", hostname_port, thread.stack);
            let mut threads_hashmap = threads_hashmap.lock().unwrap();
            let counter = threads_hashmap.entry(key).or_insert(0);
            *counter+=1;
        }
    }
}

fn read_threads(
    hostname: &str
) -> Vec<Threads> {
    if ! scan_port_addr( hostname ) {
        println!("Warning: hostname:port {} cannot be reached, skipping", hostname.to_string());
        return Vec::new();
    }
    if let Ok(data_from_http) = blocking::get(format!("http://{}/threadz?group=all", hostname.to_string())) {
        parse_threads(data_from_http.text().unwrap())
    } else {
        parse_threads(String::from(""))
    }
}

fn parse_threads(
    http_data: String
) -> Vec<Threads> {
    let mut threads: Vec<Threads> = Vec::new();
    let function_regex = Regex::new(r"@\s+0x[[:xdigit:]]+\s+(\S+)\n").unwrap();
    if let Some(table) = find_table(&http_data) {
        let (headers, rows) = table;

        let try_find_header = |target| headers.iter().position(|h| h == target);
        // mind "Thread name": name doesn't start with a capital, unlike all other headings
        let thread_name_pos = try_find_header("Thread name");
        let cumul_user_cpus_pos = try_find_header("Cumulative User CPU(s)");
        let cumul_kernel_cpus_pos = try_find_header("Cumulative Kernel CPU(s)");
        let cumul_iowaits_pos = try_find_header("Cumulative IO-wait(s)");

        let take_or_missing =
            |row: &mut [String], pos: Option<usize>| match pos.and_then(|pos| row.get_mut(pos)) {
                Some(value) => std::mem::take(value),
                None => "<Missing>".to_string(),
            };

        let mut stack_from_table = String::from("Initial value: this should not be visible");
        for mut row in rows {
            stack_from_table = if row.len() == 5 {
                std::mem::take(&mut row[4])
            } else {
                //   empty_stack_from_table.to_string()
                stack_from_table.to_string()
            };
            let mut st = Vec::new();
            for c in function_regex.captures_iter(&stack_from_table) {
                st.push(c[1].to_string().clone());
            };
            st.reverse();
            let mut final_stack = String::from("");
            for function in &st {
                final_stack.push_str(&function );
                final_stack.push_str(";");
            }
            final_stack.pop();
            threads.push(Threads {
                thread_name: take_or_missing(&mut row, thread_name_pos),
                cumulative_user_cpu_s: take_or_missing(&mut row, cumul_user_cpus_pos),
                cumulative_kernel_cpu_s: take_or_missing(&mut row, cumul_kernel_cpus_pos),
                cumulative_iowait_cpu_s: take_or_missing(&mut row, cumul_iowaits_pos),
                //stack: stack_from_table.clone(),
                stack: final_stack,
            });
        }
    }
    threads
}

fn find_table(http_data: &str) -> Option<(Vec<String>, Vec<Vec<String>>)> {
    let css = |selector| Selector::parse(selector).unwrap();
    let get_cells = |row: ElementRef, selector| {
        row.select(&css(selector))
            .map(|cell| cell.inner_html().trim().to_string())
            .collect()
    };
    let html = Html::parse_fragment(http_data);
    let table = html.select(&css("table")).next()?;
    let tr = css("tr");
    let mut rows = table.select(&tr);
    let headers = get_cells(rows.next()?, "th");
    let rows: Vec<_> = rows.map(|row| get_cells(row, "td")).collect();
    Some((headers, rows))
}