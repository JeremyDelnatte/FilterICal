use std::{collections::HashSet, fs, io};

use icalendar::{Calendar, CalendarComponent, Component};
use regex::Regex;
use serde_json::Value;

const ICAL_URL: &str = "https://hplanning2024.umons.ac.be/Telechargements/ical/Edt__MAB1___Sc__informatiques__FA.ics?version=2024.0.7.0&icalsecurise=559E8BCDD98FE0A63A41A0A820A78B92FCD331DF4E802E22EC79CA53D02369ED2254A2E15B06499B1C8409E71728D20C&param=643d5b312e2e36325d2666683d3126663d3131303030";

fn fetch_ical() -> Result<String, Box<dyn std::error::Error>> {
    let response = reqwest::blocking::get(ICAL_URL)?;
    Ok(response.text()?)
}

fn save_ics(ics_content: &str) -> io::Result<()> {
    fs::write("filtered_calendar.ics", ics_content)?;
    Ok(())
}

fn filter_event(ical_content: &str, whitelist: HashSet<String>) -> Result<String, Box<dyn std::error::Error>> {
    let calendar = ical_content.parse::<Calendar>()?;
    let mut filtered_calendar = Calendar::new();
    filtered_calendar.properties = calendar.properties.clone();

    let re = Regex::new(r"([A-Z]-[A-Z]*-\d*)").unwrap();

    calendar.components.iter()
        .filter_map(|component| {
            match component {
                CalendarComponent::Event(event) => event.get_description().map(|description| (event, description)),
                _ => None,
            }
        })
        .for_each(|(event, description)| {
            if let Some(course_code) = re.captures(description).and_then(|caps| caps.get(1)) {
                if whitelist.contains(course_code.as_str()) {
                    filtered_calendar.push(event.clone());
                }
            } else {
                filtered_calendar.push(event.clone());
            }
        });

    Ok(filtered_calendar.to_string())
}

fn print_all_courses(ical_content: &str) -> Result<(), Box<dyn std::error::Error>> {
    let calendar = ical_content.parse::<Calendar>()?;
    let mut all_courses = HashSet::new();
    let re = Regex::new(r"([A-Z]-[A-Z]*-\d*)").unwrap();

    calendar.components.iter()
        .filter_map(|component| {
            match component {
                CalendarComponent::Event(event) => event.get_description(),
                _ => None,
            }
        })
        .filter_map(|description| {
            re.captures(description).and_then(|caps| {
                caps.get(1)
            })
        })
        .for_each(|course_name| {
            all_courses.insert(course_name.as_str());
        });

    println!("{all_courses:#?}");
    Ok(())
}

fn load_whitelist() -> Result<HashSet<String>, Box<dyn std::error::Error>> {
    let file_content = fs::read_to_string("whitelist.json")?;
    let json_value: Value = serde_json::from_str(&file_content)?;
    let whitelist: HashSet<String> = serde_json::from_value(json_value)?;
    Ok(whitelist)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ical_content = fetch_ical()?;
    let whitelist = load_whitelist()?;
    let ics_content = filter_event(&ical_content, whitelist)?;
    save_ics(&ics_content)?;
    Ok(())
}
