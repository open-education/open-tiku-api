use crate::api::question::{
    QuestionInfoReq, QuestionInfoResp, QuestionListReq, QuestionListResp, QuestionUploadReq,
    QuestionUploadResp,
};
use crate::constant::meta;
use crate::service::{index, similar};
use crate::util::{file, md, string, time};
use log::{error, warn};
use std::io::{Error, ErrorKind};

fn get_question_left(title: &str, n: usize) -> String {
    (&format!("{:x}", md5::compute(title))[..n]).to_string()
}

pub async fn upload_question(
    meta_path: &str,
    req: QuestionUploadReq,
) -> Result<QuestionUploadResp, Error> {
    if !string::has_content(&req.title_val) {
        return Err(Error::new(ErrorKind::Other, "标题不能为空"));
    }

    let textbook_key = req.textbook_key.clone();
    let catalog_key = req.catalog_key.clone();
    let source_id = req.source_id.clone();

    // init base info
    let mut question_index_list =
        index::read_question_index(meta_path, &req.textbook_key, &req.catalog_key)?;
    let max_id = index::get_question_index_max_id(&question_index_list);
    let next_max_id = max_id + 1;
    let question_left = get_question_left(&req.title_val, meta::QUESTION_INDEX_LENGTH);

    if let Some(_find_index) = question_index_list
        .iter()
        .find(|item| item.left == question_left)
    {
        return Err(Error::new(ErrorKind::Other, "这个标题已被添加过"));
    }

    let mut question_index = index::QuestionIndex {
        id: next_max_id,
        left: question_left.clone(),
        question_type: req.question_type.clone(),
        tags: req.tags.clone(),
        rate_val: req.rate_val.clone(),
        image_names: req.image_names.clone(),
        show_image_val: req.show_image_val.clone(),
        show_select_val: req.show_select_val.clone(),
        create_time: None,
        update_time: None,
        author: None,
    };

    // upload question file
    match md::write_markdown_files(meta_path, req, &question_index).await {
        Ok(_) => {}
        Err(e) => {
            error!("Failed to upload question to markdown file: {}", e);
            Err(Error::new(ErrorKind::Other, "题目上传失败".to_string()))?
        }
    }

    let current_time = time::get_beijing_time_info();
    question_index.create_time = Some(current_time.0);
    question_index.update_time = Some(current_time.0);
    let id = index::append_write_index(
        meta_path,
        &textbook_key,
        &catalog_key,
        &mut question_index_list,
        question_index,
    )?;

    // related source_id
    match source_id {
        Some(source_id) => {
            let _ = related_source_id(meta_path, &textbook_key, &catalog_key, &source_id, &id)?;
        }
        None => {}
    }

    Ok(QuestionUploadResp {
        id: format!("{}", id),
    })
}

pub fn related_source_id(
    meta_path: &str,
    textbook_key: &str,
    catalog_key: &str,
    source_id: &str,
    target_id: &str,
) -> Result<bool, Error> {
    if !string::has_content(source_id) {
        return Ok(true);
    }

    let mut similar_index_map =
        similar::read_question_similar_index(meta_path, textbook_key, catalog_key)?;
    let mut index_list = match similar_index_map.get(source_id) {
        Some(index_list) => index_list.clone(),
        _ => vec![],
    };
    index_list.push(target_id.to_string());
    similar_index_map.insert(source_id.to_string(), index_list);

    similar::write_similar_index(meta_path, textbook_key, catalog_key, &similar_index_map)
}

pub fn get_question_info(meta_path: &str, req: QuestionInfoReq) -> Result<QuestionInfoResp, Error> {
    let textbook_key = req.textbook_key.clone();
    let catalog_key = req.catalog_key.clone();
    let question_index_list = index::read_question_index(meta_path, &textbook_key, &catalog_key)?;
    let question_index = index::get_question_index(&req.id, question_index_list)?;

    read_question_info(meta_path, req, &question_index)
}

fn read_question_info(
    meta_path: &str,
    req: QuestionInfoReq,
    index: &index::QuestionIndex,
) -> Result<QuestionInfoResp, Error> {
    let question_path = req.id.clone();
    let textbook_key = req.textbook_key.clone();
    let catalog_key = req.catalog_key.clone();

    let mut question_info_resp = QuestionInfoResp {
        id: req.id,
        textbook_key: req.textbook_key,
        catalog_key: req.catalog_key,
        question_type: index.question_type.clone(),
        tags: index.tags.clone(),
        rate_val: index.rate_val.clone(),
        title_val: "".to_string(),
        mention_val: None,
        image_names: index.image_names.clone(),
        show_image_val: index.show_image_val.clone(),
        a_val: None,
        b_val: None,
        c_val: None,
        d_val: None,
        e_val: None,
        show_select_val: index.show_select_val.clone(),
        answer_val: None,
        knowledge_val: None,
        analyze_val: None,
        process_val: None,
        remark_val: None,
    };

    // read req file
    let mut ext_list = Vec::new();
    match req.ext {
        Some(mut req_ext) => ext_list.append(&mut req_ext),
        None =>
        // read all file
        {
            ext_list = vec![
                "title".to_string(),
                "mention".to_string(),
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "d".to_string(),
                "e".to_string(),
                "answer".to_string(),
                "knowledge".to_string(),
                "analyze".to_string(),
                "process".to_string(),
                "remark".to_string(),
            ]
        }
    }

    match read_ext_list_content(
        &format!(
            "{}/{}/{}/{}",
            meta_path,
            string::underline_to_slash(&textbook_key),
            string::underline_to_slash(&catalog_key),
            question_path
        ),
        ext_list,
        &mut question_info_resp,
    ) {
        Ok(_) => Ok(question_info_resp),
        Err(e) => {
            error!("Failed to load question info from file: {}", e);
            Err(Error::new(ErrorKind::Other, "读取题目信息出错"))
        }
    }
}

fn read_ext_list_content(
    file_path: &str,
    ext_list: Vec<String>,
    resp: &mut QuestionInfoResp,
) -> Result<bool, Error> {
    for ext in ext_list {
        match ext.as_str() {
            "title" => {
                resp.title_val = file::read_small_file(
                    format!("{}/{}", file_path, meta::QUESTION_TITLE_NAME),
                    false,
                )?;
            }
            "mention" => {
                resp.mention_val = Some(file::read_small_file(
                    format!("{}/{}", file_path, meta::QUESTION_MENTION_NAME),
                    false,
                )?);
            }
            "a" => {
                resp.a_val = Some(file::read_small_file(
                    format!("{}/{}", file_path, meta::QUESTION_A_NAME),
                    false,
                )?);
            }
            "b" => {
                resp.b_val = Some(file::read_small_file(
                    format!("{}/{}", file_path, meta::QUESTION_B_NAME),
                    false,
                )?);
            }
            "c" => {
                resp.c_val = Some(file::read_small_file(
                    format!("{}/{}", file_path, meta::QUESTION_C_NAME),
                    false,
                )?);
            }
            "d" => {
                resp.d_val = Some(file::read_small_file(
                    format!("{}/{}", file_path, meta::QUESTION_D_NAME),
                    false,
                )?);
            }
            "e" => {
                resp.e_val = Some(file::read_small_file(
                    format!("{}/{}", file_path, meta::QUESTION_E_NAME),
                    false,
                )?);
            }
            "answer" => {
                resp.answer_val = Some(file::read_small_file(
                    format!("{}/{}", file_path, meta::QUESTION_ANSWER_NAME),
                    false,
                )?);
            }
            "knowledge" => {
                resp.knowledge_val = Some(file::read_small_file(
                    format!("{}/{}", file_path, meta::QUESTION_KNOWLEDGE_NAME),
                    false,
                )?);
            }
            "analyze" => {
                resp.analyze_val = Some(file::read_small_file(
                    format!("{}/{}", file_path, meta::QUESTION_ANALYZE_NAME),
                    false,
                )?);
            }
            "process" => {
                resp.process_val = Some(file::read_small_file(
                    format!("{}/{}", file_path, meta::QUESTION_PROCESS_NAME),
                    false,
                )?);
            }
            "remark" => {
                resp.remark_val = Some(file::read_small_file(
                    format!("{}/{}", file_path, meta::QUESTION_REMARK_NAME),
                    false,
                )?);
            }
            _ => {
                warn!("Unknown extension: {}", ext);
            }
        }
    }

    Ok(true)
}

pub fn get_question_list(meta_path: &str, req: QuestionListReq) -> Result<QuestionListResp, Error> {
    let textbook_key = req.textbook_key.clone();
    let catalog_key = req.catalog_key.clone();
    let question_index_list = index::read_question_index(meta_path, &textbook_key, &catalog_key)?;
    // 本期不考虑排序, 正常默认就是升序

    let total = question_index_list.len();
    let mut question_list_resp = QuestionListResp {
        page_no: req.page_no,
        page_size: req.page_size,
        total,
        data: vec![],
    };

    // 计算分页看需要获取多少个题目索引片段
    let start_index = (req.page_no - 1) * req.page_size;
    let end_index = req.page_no * req.page_size;
    let get_question_index_list = question_index_list
        .get(start_index..end_index.min(total))
        .unwrap_or(&[]);

    let mut question_info_list: Vec<QuestionInfoResp> = Vec::new();
    for question_index in get_question_index_list {
        let question_info = read_question_info(
            meta_path,
            QuestionInfoReq {
                textbook_key: textbook_key.clone(),
                catalog_key: catalog_key.clone(),
                id: format!(
                    "{}_{}",
                    question_index.id.clone(),
                    question_index.left.clone()
                ),
                ext: Some(vec![
                    "title".to_string(),
                    "mention".to_string(),
                    "a".to_string(),
                    "b".to_string(),
                    "c".to_string(),
                    "d".to_string(),
                    "e".to_string(),
                ]),
            },
            question_index,
        )?;
        question_info_list.push(question_info);
    }
    if question_info_list.len() > 0 {
        question_list_resp.data.append(&mut question_info_list);
    }
    Ok(question_list_resp)
}
