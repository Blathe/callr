use crate::model::history::HistoryEntry;
use crate::model::request::RequestFile;
use crate::model::response::ResponseData;

#[derive(Debug)]
pub enum AppMsg {
    HttpResult {
        request: Box<RequestFile>,
        result: Result<ResponseData, String>,
    },
    HistoryLoaded(Result<Vec<HistoryEntry>, String>),
}
