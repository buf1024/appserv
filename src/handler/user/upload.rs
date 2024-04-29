use std::{fs, path::Path};

use crate::{
    app_state::AppState,
    auth_user::AuthUser,
    config::CONFIG,
    errors::{Error, E_SUCCESS},
    handler::ok_with_trace,
    proto::UploadRsp,
    JsonResult,
};
use axum::{
    debug_handler,
    extract::{Multipart, State},
};
use nanoid::nanoid;
#[debug_handler(state = AppState)]
pub async fn upload(
    State(_state): State<AppState>,
    _auth_user: AuthUser,
    mut multipart: Multipart,
) -> JsonResult<UploadRsp> {
    let mut avatar_path = String::from("");
    while let Some(field) = multipart.next_field().await.unwrap() {
        let file_name = field
            .file_name()
            .ok_or(Error::Internal(String::from("fail to get file name")))?
            .to_string();
        let data = field.bytes().await.map_err(|e| {
            Error::Internal(format!("fail to get file name, error: {}", e.to_string()))
        })?;

        let file_ext: Vec<_> = file_name.split(".").collect();

        let mut ext = String::from("");
        if let Some(f_ext) = file_ext.last() {
            ext.push_str(".");
            ext.push_str(*&f_ext);
        }
        let alphabet: [char; 16] = [
            '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 'a', 'b', 'c', 'd', 'e', 'f',
        ];
        let name = nanoid!(32, &alphabet);
        let file_name = format!("{}{}", name, ext);

        let path_str = format!("{}/{}", &CONFIG.avatar_path, file_name);
        let path = Path::new(&path_str);

        fs::write(path, data).map_err(|e| {
            Error::Internal(format!(
                "fail to write file: {}, error: {}",
                path_str,
                e.to_string()
            ))
        })?;
        avatar_path = file_name;
        break;
    }

    let rsp = UploadRsp {
        error: E_SUCCESS,
        message: "success".into(),
        avatar_path,
    };

    ok_with_trace(rsp)
}
