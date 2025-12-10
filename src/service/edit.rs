use crate::api::edit::{
    EditAReq, EditAnalyzeReq, EditAnswerReq, EditBReq, EditCReq, EditDReq, EditEReq,
    EditKnowledgeReq, EditMentionReq, EditProcessReq, EditQuestionTypeReq, EditRateReq,
    EditRemarkReq, EditSelectReq, EditTagsReq, EditTitleReq,
};
use crate::constant::meta;
use crate::service::index;
use crate::util::{file, string};
use std::io::Error;

pub fn edit_question_type(meta_path: &str, req: EditQuestionTypeReq) -> Result<bool, Error> {
    let mut question_index_list =
        index::read_question_index(meta_path, &req.textbook_key, &req.catalog_key)?;
    for question_index in &mut question_index_list {
        if format!("{}_{}", question_index.id, question_index.left) == req.id {
            question_index.question_type = req.question_type;
            break;
        }
    }

    let _ = index::write_index(
        meta_path,
        &req.textbook_key,
        &req.catalog_key,
        &question_index_list,
    )?;

    Ok(true)
}

pub fn edit_tags(meta_path: &str, req: EditTagsReq) -> Result<bool, Error> {
    let mut question_index_list =
        index::read_question_index(meta_path, &req.textbook_key, &req.catalog_key)?;
    for question_index in &mut question_index_list {
        if format!("{}_{}", question_index.id, question_index.left) == req.id {
            question_index.tags = Some(req.tags);
            break;
        }
    }

    let _ = index::write_index(
        meta_path,
        &req.textbook_key,
        &req.catalog_key,
        &question_index_list,
    )?;

    Ok(true)
}

pub fn edit_rate(meta_path: &str, req: EditRateReq) -> Result<bool, Error> {
    let mut question_index_list =
        index::read_question_index(meta_path, &req.textbook_key, &req.catalog_key)?;
    for question_index in &mut question_index_list {
        if format!("{}_{}", question_index.id, question_index.left) == req.id {
            question_index.rate_val = Some(req.rate);
            break;
        }
    }

    let _ = index::write_index(
        meta_path,
        &req.textbook_key,
        &req.catalog_key,
        &question_index_list,
    )?;

    Ok(true)
}

pub fn edit_select(meta_path: &str, req: EditSelectReq) -> Result<bool, Error> {
    let mut question_index_list =
        index::read_question_index(meta_path, &req.textbook_key, &req.catalog_key)?;
    for question_index in &mut question_index_list {
        if format!("{}_{}", question_index.id, question_index.left) == req.id {
            question_index.show_select_val = Some(req.select);
            break;
        }
    }

    let _ = index::write_index(
        meta_path,
        &req.textbook_key,
        &req.catalog_key,
        &question_index_list,
    )?;

    Ok(true)
}

pub fn edit_title(meta_path: &str, req: EditTitleReq) -> Result<bool, Error> {
    let title_path = format!(
        "{}/{}/{}/{}/{}",
        meta_path,
        string::underline_to_slash(&req.textbook_key),
        string::underline_to_slash(&req.catalog_key),
        req.id,
        meta::QUESTION_TITLE_NAME
    );
    file::write_small_file(&title_path, &req.title)
}

pub fn edit_mention(meta_path: &str, req: EditMentionReq) -> Result<bool, Error> {
    let mention_path = format!(
        "{}/{}/{}/{}/{}",
        meta_path,
        string::underline_to_slash(&req.textbook_key),
        string::underline_to_slash(&req.catalog_key),
        req.id,
        meta::QUESTION_MENTION_NAME
    );
    file::write_small_file(&mention_path, &req.mention)
}

pub fn edit_a(meta_path: &str, req: EditAReq) -> Result<bool, Error> {
    let a_path = format!(
        "{}/{}/{}/{}/{}",
        meta_path,
        string::underline_to_slash(&req.textbook_key),
        string::underline_to_slash(&req.catalog_key),
        req.id,
        meta::QUESTION_A_NAME
    );
    file::write_small_file(&a_path, &req.a)
}

pub fn edit_b(meta_path: &str, req: EditBReq) -> Result<bool, Error> {
    let b_path = format!(
        "{}/{}/{}/{}/{}",
        meta_path,
        string::underline_to_slash(&req.textbook_key),
        string::underline_to_slash(&req.catalog_key),
        req.id,
        meta::QUESTION_B_NAME
    );
    file::write_small_file(&b_path, &req.b)
}

pub fn edit_c(meta_path: &str, req: EditCReq) -> Result<bool, Error> {
    let c_path = format!(
        "{}/{}/{}/{}/{}",
        meta_path,
        string::underline_to_slash(&req.textbook_key),
        string::underline_to_slash(&req.catalog_key),
        req.id,
        meta::QUESTION_C_NAME
    );
    file::write_small_file(&c_path, &req.c)
}

pub fn edit_d(meta_path: &str, req: EditDReq) -> Result<bool, Error> {
    let d_path = format!(
        "{}/{}/{}/{}/{}",
        meta_path,
        string::underline_to_slash(&req.textbook_key),
        string::underline_to_slash(&req.catalog_key),
        req.id,
        meta::QUESTION_D_NAME
    );
    file::write_small_file(&d_path, &req.d)
}

pub fn edit_e(meta_path: &str, req: EditEReq) -> Result<bool, Error> {
    let e_path = format!(
        "{}/{}/{}/{}/{}",
        meta_path,
        string::underline_to_slash(&req.textbook_key),
        string::underline_to_slash(&req.catalog_key),
        req.id,
        meta::QUESTION_E_NAME
    );
    file::write_small_file(&e_path, &req.e)
}

pub fn edit_answer(meta_path: &str, req: EditAnswerReq) -> Result<bool, Error> {
    let answer_path = format!(
        "{}/{}/{}/{}/{}",
        meta_path,
        string::underline_to_slash(&req.textbook_key),
        string::underline_to_slash(&req.catalog_key),
        req.id,
        meta::QUESTION_ANSWER_NAME
    );
    file::write_small_file(&answer_path, &req.answer)
}

pub fn edit_knowledge(meta_path: &str, req: EditKnowledgeReq) -> Result<bool, Error> {
    let knowledge_path = format!(
        "{}/{}/{}/{}/{}",
        meta_path,
        string::underline_to_slash(&req.textbook_key),
        string::underline_to_slash(&req.catalog_key),
        req.id,
        meta::QUESTION_KNOWLEDGE_NAME
    );
    file::write_small_file(&knowledge_path, &req.knowledge)
}

pub fn edit_analyze(meta_path: &str, req: EditAnalyzeReq) -> Result<bool, Error> {
    let analyze_path = format!(
        "{}/{}/{}/{}/{}",
        meta_path,
        string::underline_to_slash(&req.textbook_key),
        string::underline_to_slash(&req.catalog_key),
        req.id,
        meta::QUESTION_ANALYZE_NAME
    );
    file::write_small_file(&analyze_path, &req.analyze)
}

pub fn edit_process(meta_path: &str, req: EditProcessReq) -> Result<bool, Error> {
    let process_path = format!(
        "{}/{}/{}/{}/{}",
        meta_path,
        string::underline_to_slash(&req.textbook_key),
        string::underline_to_slash(&req.catalog_key),
        req.id,
        meta::QUESTION_PROCESS_NAME
    );
    file::write_small_file(&process_path, &req.process)
}

pub fn edit_remark(meta_path: &str, req: EditRemarkReq) -> Result<bool, Error> {
    let remark_path = format!(
        "{}/{}/{}/{}/{}",
        meta_path,
        string::underline_to_slash(&req.textbook_key),
        string::underline_to_slash(&req.catalog_key),
        req.id,
        meta::QUESTION_REMARK_NAME
    );
    file::write_small_file(&remark_path, &req.remark)
}
