use crate::config::knowledge::get_knowledge;
use crate::config::publisher::get_publishers;
use crate::config::stage::get_stages;
use crate::config::subject::{get_subjects, Subject};
use crate::config::textbook::get_textbooks;
use std::io::Error;

pub fn get_guidance(meta_path: &str) -> Result<Vec<Subject>, Error> {
    let mut subjects = get_subjects(meta_path)?;

    for subject in subjects.iter_mut() {
        let publishers = get_publishers(meta_path, &subject.key)?;
        subject.children = publishers;
        for publisher in subject.children.iter_mut() {
            let stages = get_stages(meta_path, &publisher.key)?;
            publisher.children = stages;
            for stage in publisher.children.iter_mut() {
                let textbooks = get_textbooks(meta_path, &stage.key)?;
                stage.textbook_list = textbooks;
                let knowledge_list = get_knowledge(meta_path, &stage.key)?;
                stage.knowledge_list = knowledge_list;
            }
        }
    }

    Ok(subjects)
}
