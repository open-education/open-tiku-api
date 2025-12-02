use crate::config::publisher::{Publisher, Subject, get_publisher_by_key, get_publishers};
use crate::config::stage::get_stages;
use crate::config::textbook::get_textbooks;
use crate::util::string;
use std::io::Error;

pub fn get_guidance() -> Result<Vec<Publisher>, Error> {
    let mut publishers = get_publishers()?;

    for publisher in publishers.iter_mut() {
        for subject in publisher.children.iter_mut() {
            let stages = get_stages(&subject.key)?;
            subject.children = stages;

            for subject in subject.children.iter_mut() {
                let textbooks = get_textbooks(&subject.key)?;
                subject.children = textbooks;
            }
        }
    }

    Ok(publishers)
}

pub fn get_guidance_by_publisher(key: &str) -> Result<Vec<Subject>, Error> {
    let publisher_name = string::get_first_part(key)?;
    let publisher = get_publisher_by_key(publisher_name)?;
    let mut subjects = publisher.children;
    for subject in subjects.iter_mut() {
        let stages = get_stages(&subject.key)?;
        subject.children = stages;
        for stage in subject.children.iter_mut() {
            let textbooks = get_textbooks(&stage.key)?;
            stage.children = textbooks;
        }
    }

    Ok(subjects)
}
