// use axum::{
//     debug_handler,
//     extract::{Query, State},
//     Json,
// };

// use crate::{
//     app_state::AppState,
//     errors::{Error, E_SUCCESS},
//     proto::{ActivateReq, ActivateResp},
//     JsonResult,
// };

// use async_session::SessionStore;

// #[debug_handler(state = AppState)]
// pub async fn activate(
//     State(state): State<AppState>,
//     Query(param): Query<ActivateReq>,
// ) -> JsonResult<ActivateResp> {
//     let user_id = {
//         let active_id = urlencoding::decode(&param.activate)
//             .map_err(|e| Error::Custom(format!("invalid link: {}", e)))?
//             .to_string();
//         let session = state
//             .store
//             .load_session(active_id.clone())
//             .await
//             .map_err(|e| Error::Custom(format!("activate session not found: {}", e)))?;
//         if session.is_none() {
//             return Err(Error::Custom(format!(
//                 "activate session not found: {}",
//                 param.activate
//             )));
//         }
//         let session = session.unwrap();

//         let user_id: i64 = session
//             .get("user_id")
//             .ok_or(Error::Custom(format!("activate session not found")))?;

//         state
//             .store
//             .destroy_session(session)
//             .await
//             .map_err(|e| Error::Custom(format!("destroy session error: {}", e)))?;
//         user_id
//     };

//     state.repo.activate_user(user_id).await?;

//     let resp = ActivateResp {
//         error: E_SUCCESS,
//         message: "success".to_string(),
//     };
//     Ok(Json(resp))
// }
