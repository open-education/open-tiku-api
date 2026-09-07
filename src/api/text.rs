use crate::api::req::question::CreateQuestionReq;
use crate::api::req::text::QuestionSnippetReq;
use crate::app::conf::AppState;
use crate::service::question_upload;
use crate::util::response::ApiResponse;
use actix_web::{post, web};
// 文本片段解析工具

#[post("/question/snippet")]
pub async fn question_snippet(
    app_state: web::Data<AppState>,
    req: web::Json<QuestionSnippetReq>,
) -> ApiResponse<CreateQuestionReq> {
    ApiResponse::response(
        question_upload::parse_question_snippet(&app_state, req.into_inner()).await,
    )
}
