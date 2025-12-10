use crate::api;
use crate::service::index;
use crate::util;
use actix_multipart::Multipart;
use std::io::Error;

pub async fn upload_small_image(
    meta_path: &str,
    payload: Multipart,
    req_upload_file: api::file::UploadImageReq,
) -> Result<Vec<util::upload::UploadImageResp>, Error> {
    let textbook_key = req_upload_file.textbook_key.clone();
    let catalog_key = req_upload_file.catalog_key.clone();
    let id_opt = req_upload_file.id.clone();

    let upload_image_resp_list =
        util::upload::upload_small_image(meta_path, payload, req_upload_file).await?;

    match id_opt {
        Some(id) => {
            let mut find = false;

            let mut question_index_list =
                index::read_question_index(meta_path, &textbook_key, &catalog_key)?;
            for question_index in question_index_list.iter_mut() {
                if format!("{}_{}", question_index.id, question_index.left) == id {
                    find = true;

                    let mut image_names = question_index.image_names.clone().unwrap_or(vec![]);
                    for upload_image_resp in upload_image_resp_list.iter() {
                        image_names.push(upload_image_resp.name.clone());
                    }
                    question_index.image_names = Some(image_names);
                    break;
                }
            }

            if find {
                _ = index::write_index(
                    meta_path,
                    &textbook_key,
                    &catalog_key,
                    &question_index_list,
                )?;
            }

            Ok(upload_image_resp_list)
        }
        None => Ok(upload_image_resp_list),
    }
}

pub async fn delete_image(
    meta_path: &str,
    local_image_info: util::file::LocalImageInfo,
) -> Result<bool, Error> {
    let textbook_key = local_image_info.textbook_key.clone();
    let catalog_key = local_image_info.catalog_key.clone();
    let id_opt = local_image_info.id.clone();
    let filename = local_image_info.filename.clone();

    util::file::delete_image(meta_path, local_image_info).await?;

    match id_opt {
        Some(id) => {
            let mut find = false;

            let mut question_index_list =
                index::read_question_index(meta_path, &textbook_key, &catalog_key)?;
            for question_index in question_index_list.iter_mut() {
                if format!("{}_{}", question_index.id, question_index.left) == id {
                    find = true;

                    let mut image_names = question_index.image_names.clone().unwrap_or(vec![]);
                    if image_names.len() == 0 {
                        find = false;
                        break;
                    }
                    image_names.retain(|image_name| image_name != &filename);
                    question_index.image_names = Some(image_names);
                    break;
                }
            }

            if find {
                _ = index::write_index(
                    meta_path,
                    &textbook_key,
                    &catalog_key,
                    &question_index_list,
                )?;
            }

            Ok(true)
        }
        None => Ok(true),
    }
}
