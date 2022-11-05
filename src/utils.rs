use std::{
    fs::File,
    io::{self, BufRead, Write},
    path::Path,
};

use colored::*;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use linked_hash_map::LinkedHashMap;
use rand::Rng;
use url::Url;

use crate::{config::structs::Config, RANDOM_CHARSET};

/// prints informative messages/non critical errors
pub fn info<S: Into<String>, T: std::fmt::Display>(
    config: &Config,
    id: usize,
    progress_bar: &ProgressBar,
    word: S,
    msg: T,
) {
    if config.verbose > 0 {
        progress_bar.println(format!(
            "{} [{}] {}",
            color_id(id),
            word.into().yellow(),
            msg
        ));
    }
}

/// prints errors. Progress_bar may be null in case the error happened too early (before requests)
pub fn error<T: std::fmt::Display>(msg: T, url: Option<&str>, progress_bar: Option<&ProgressBar>) {
    let message = if url.is_none() {
        format!("{} {}", "[#]".red(), msg)
    } else {
        format!("{} [{}] {}", "[#]".red(), url.unwrap(), msg)
    };

    if progress_bar.is_none() {
        writeln!(io::stdout(), "{}", message).ok();
    } else {
        progress_bar.unwrap().println(message);
    }
}

/// initialize progress bars for every url set
pub fn init_progress(config: &Config) -> Vec<(ProgressBar, Vec<String>)> {
    let mut urls_to_progress = Vec::new();
    let m = MultiProgress::new();

    // we're creating an empty progress bar to make one empty line between progress bars and the tool's output
    let empty_line = m.add(ProgressBar::new(128));
    let sty = ProgressStyle::with_template(" ").unwrap();
    empty_line.set_style(sty);
    empty_line.inc(1);
    urls_to_progress.push((empty_line, vec![String::new()]));

    // in case --one-worker-per-host option is provided -- each url set contains urls with one host
    // otherwise it's just url sets with one url
    let urls = if config.one_worker_per_host {
        order_urls(&config.urls)
    } else {
        config.urls.iter().map(|x| vec![x.to_owned()]).collect()
    };

    // append progress bars one after another and push them to urls_to_progress
    for url_set in urls {
        let pb = m.insert_from_back(
            0,
            if config.disable_progress_bar || config.verbose < 1 {
                ProgressBar::new(128)
            } else {
                ProgressBar::hidden()
            },
        );
        urls_to_progress.push((pb.clone(), url_set));
    }

    urls_to_progress
}

/// read wordlist with parameters
pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

/// read parameters from stdin
pub fn read_stdin_lines() -> Vec<String> {
    let stdin = io::stdin();
    stdin.lock().lines().filter_map(|x| x.ok()).collect()
}

/// generate random word of RANDOM_CHARSET chars
pub fn random_line(size: usize) -> String {
    (0..size)
        .map(|_| {
            let idx = rand::thread_rng().gen_range(0, RANDOM_CHARSET.len());
            RANDOM_CHARSET[idx] as char
        })
        .collect()
}

/// returns colored id when > 1 url is being tested in the same time
pub fn color_id(id: usize) -> String {
    if id % 7 == 0 {
        id.to_string().white()
    } else if id % 6 == 0 {
        id.to_string().bright_red()
    } else if id % 5 == 0 {
        id.to_string().bright_cyan()
    } else if id % 4 == 0 {
        id.to_string().bright_blue()
    } else if id % 3 == 0 {
        id.to_string().yellow()
    } else if id % 2 == 0 {
        id.to_string().bright_green()
    } else if id % 1 == 0 {
        id.to_string().magenta()
    } else {
        unreachable!()
    }
    .to_string()
}

/// moves urls with different hosts to different vectors
pub fn order_urls(urls: &Vec<String>) -> Vec<Vec<String>> {
    // LinkedHashMap instead of hashmap for preserving the order
    // LinkedHashMap<HOST, Vec<URL>>
    let mut sorted_urls: LinkedHashMap<String, Vec<String>> = LinkedHashMap::new();
    let mut ordered_urls: Vec<Vec<String>> = Vec::new();

    for url in urls.iter() {
        let parsed_url = Url::parse(url).unwrap();
        let host = parsed_url.host_str().unwrap();

        if sorted_urls.contains_key(host) {
            sorted_urls.get_mut(host).unwrap().push(url.to_owned());
        } else {
            sorted_urls.insert(host.to_owned(), vec![url.to_owned()]);
        }
    }

    for host in sorted_urls.clone().keys() {
        ordered_urls.push(sorted_urls[host].clone())
    }

    ordered_urls
}
