use std::{collections::{HashMap, HashSet}, fs, io};

use icalendar::{Calendar, CalendarComponent, Component};
use regex::Regex;
use serde_json::Value;

fn fetch_ical(ical_url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let response = reqwest::blocking::get(ical_url)?;
    Ok(response.text()?)
}

fn save_ics(ics_content: &str, name: &str) -> io::Result<()> {
    fs::write(format!("{}.ics", name), ics_content)?;
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

fn load_whitelist() -> Result<HashSet<String>, Box<dyn std::error::Error>> {
    let file_content = fs::read_to_string("whitelist.json")?;
    let json_value: Value = serde_json::from_str(&file_content)?;
    let whitelist: HashSet<String> = serde_json::from_value(json_value)?;
    Ok(whitelist)
}

fn load_ical_urls() -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let file_content = fs::read_to_string("ical_urls.json")?;
    let json_value: Value = serde_json::from_str(&file_content)?;
    let whitelist: HashMap<String, String> = serde_json::from_value(json_value)?;
    Ok(whitelist)
} 

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ical_urls = load_ical_urls()?;

    for (name, ical_url) in ical_urls {
        let ical_content = match fetch_ical(&ical_url) {
            Ok(ical_content) => ical_content,
            _ => return Err(Box::new(io::Error::new(io::ErrorKind::InvalidData, format!("Couldn't fetch the ics file for {}", ical_url)))),
        };

        let whitelist = load_whitelist()?;
        let ics_content = filter_event(&ical_content, whitelist)?;
        save_ics(&ics_content, &name)?;
    }

    Ok(())
}
