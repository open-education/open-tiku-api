use crate::api::edit::{
    EditAReq, EditAnalyzeReq, EditAnswerReq, EditBReq, EditCReq, EditDReq, EditEReq,
    EditKnowledgeReq, EditMentionReq, EditProcessReq, EditQuestionTypeReq, EditRateReq,
    EditRemarkReq, EditSelectReq, EditTagsReq, EditTitleReq,
};
use crate::constant::meta;
use crate::service::index;
use crate::util::{file, string};
use std::io::Error;

pub fn edit_question_type(req: EditQuestionTypeReq) -> Result<bool, Error> {
    let mut question_index_list = index::read_question_index(&req.textbook_key, &req.catalog_key)?;
    for question_index in &mut question_index_list {
        if format!("{}_{}", question_index.id, question_index.left) == req.id {
            question_index.question_type = req.question_type;
            break;
        }
    }

    let _ = index::write_index(&req.textbook_key, &req.catalog_key, &question_index_list)?;

    Ok(true)
}

pub fn edit_tags(req: EditTagsReq) -> Result<bool, Error> {
    let mut question_index_list = index::read_question_index(&req.textbook_key, &req.catalog_key)?;
    for question_index in &mut question_index_list {
        if format!("{}_{}", question_index.id, question_index.left) == req.id {
            question_index.tags = Some(req.tags);
            break;
        }
    }

    let _ = index::write_index(&req.textbook_key, &req.catalog_key, &question_index_list)?;

    Ok(true)
}

pub fn edit_rate(req: EditRateReq) -> Result<bool, Error> {
    let mut question_index_list = index::read_question_index(&req.textbook_key, &req.catalog_key)?;
    for question_index in &mut question_index_list {
        if format!("{}_{}", question_index.id, question_index.left) == req.id {
            question_index.rate_val = Some(req.rate);
            break;
        }
    }

    let _ = index::write_index(&req.textbook_key, &req.catalog_key, &question_index_list)?;

    Ok(true)
}

pub fn edit_select(req: EditSelectReq) -> Result<bool, Error> {
    let mut question_index_list = index::read_question_index(&req.textbook_key, &req.catalog_key)?;
    for question_index in &mut question_index_list {
        if format!("{}_{}", question_index.id, question_index.left) == req.id {
            question_index.show_select_val = Some(req.select);
            break;
        }
    }

    let _ = index::write_index(&req.textbook_key, &req.catalog_key, &question_index_list)?;

    Ok(true)
}

pub fn edit_title(req: EditTitleReq) -> Result<bool, Error> {
    let title_path = format!(
        "{}/{}/{}/{}/{}",
        meta::META_PATH,
        string::underline_to_slash(&req.textbook_key),
        string::underline_to_slash(&req.catalog_key),
        req.id,
        meta::QUESTION_TITLE_NAME
    );
    file::write_small_file(&title_path, &req.title)
}

pub fn edit_mention(req: EditMentionReq) -> Result<bool, Error> {
    let mention_path = format!(
        "{}/{}/{}/{}/{}",
        meta::META_PATH,
        string::underline_to_slash(&req.textbook_key),
        string::underline_to_slash(&req.catalog_key),
        req.id,
        meta::QUESTION_MENTION_NAME
    );
    file::write_small_file(&mention_path, &req.mention)
}

pub fn edit_a(req: EditAReq) -> Result<bool, Error> {
    let a_path = format!(
        "{}/{}/{}/{}/{}",
        meta::META_PATH,
        string::underline_to_slash(&req.textbook_key),
        string::underline_to_slash(&req.catalog_key),
        req.id,
        meta::QUESTION_A_NAME
    );
    file::write_small_file(&a_path, &req.a)
}

pub fn edit_b(req: EditBReq) -> Result<bool, Error> {
    let b_path = format!(
        "{}/{}/{}/{}/{}",
        meta::META_PATH,
        string::underline_to_slash(&req.textbook_key),
        string::underline_to_slash(&req.catalog_key),
        req.id,
        meta::QUESTION_B_NAME
    );
    file::write_small_file(&b_path, &req.b)
}

pub fn edit_c(req: EditCReq) -> Result<bool, Error> {
    let c_path = format!(
        "{}/{}/{}/{}/{}",
        meta::META_PATH,
        string::underline_to_slash(&req.textbook_key),
        string::underline_to_slash(&req.catalog_key),
        req.id,
        meta::QUESTION_C_NAME
    );
    file::write_small_file(&c_path, &req.c)
}

pub fn edit_d(req: EditDReq) -> Result<bool, Error> {
    let d_path = format!(
        "{}/{}/{}/{}/{}",
        meta::META_PATH,
        string::underline_to_slash(&req.textbook_key),
        string::underline_to_slash(&req.catalog_key),
        req.id,
        meta::QUESTION_D_NAME
    );
    file::write_small_file(&d_path, &req.d)
}

pub fn edit_e(req: EditEReq) -> Result<bool, Error> {
    let e_path = format!(
        "{}/{}/{}/{}/{}",
        meta::META_PATH,
        string::underline_to_slash(&req.textbook_key),
        string::underline_to_slash(&req.catalog_key),
        req.id,
        meta::QUESTION_E_NAME
    );
    file::write_small_file(&e_path, &req.e)
}

pub fn edit_answer(req: EditAnswerReq) -> Result<bool, Error> {
    let answer_path = format!(
        "{}/{}/{}/{}/{}",
        meta::META_PATH,
        string::underline_to_slash(&req.textbook_key),
        string::underline_to_slash(&req.catalog_key),
        req.id,
        meta::QUESTION_ANSWER_NAME
    );
    file::write_small_file(&answer_path, &req.answer)
}

pub fn edit_knowledge(req: EditKnowledgeReq) -> Result<bool, Error> {
    let knowledge_path = format!(
        "{}/{}/{}/{}/{}",
        meta::META_PATH,
        string::underline_to_slash(&req.textbook_key),
        string::underline_to_slash(&req.catalog_key),
        req.id,
        meta::QUESTION_KNOWLEDGE_NAME
    );
    file::write_small_file(&knowledge_path, &req.knowledge)
}

pub fn edit_analyze(req: EditAnalyzeReq) -> Result<bool, Error> {
    let analyze_path = format!(
        "{}/{}/{}/{}/{}",
        meta::META_PATH,
        string::underline_to_slash(&req.textbook_key),
        string::underline_to_slash(&req.catalog_key),
        req.id,
        meta::QUESTION_ANALYZE_NAME
    );
    file::write_small_file(&analyze_path, &req.analyze)
}

pub fn edit_process(req: EditProcessReq) -> Result<bool, Error> {
    let process_path = format!(
        "{}/{}/{}/{}/{}",
        meta::META_PATH,
        string::underline_to_slash(&req.textbook_key),
        string::underline_to_slash(&req.catalog_key),
        req.id,
        meta::QUESTION_PROCESS_NAME
    );
    file::write_small_file(&process_path, &req.process)
}

pub fn edit_remark(req: EditRemarkReq) -> Result<bool, Error> {
    let remark_path = format!(
        "{}/{}/{}/{}/{}",
        meta::META_PATH,
        string::underline_to_slash(&req.textbook_key),
        string::underline_to_slash(&req.catalog_key),
        req.id,
        meta::QUESTION_REMARK_NAME
    );
    file::write_small_file(&remark_path, &req.remark)
}
