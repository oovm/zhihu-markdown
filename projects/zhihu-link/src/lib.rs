#![feature(try_blocks)]
#![feature(lazy_cell)]

mod answers;
mod auto;
mod bilibili;
mod errors;
pub mod utils;
mod zhuanlans;
mod dispatch;

pub use crate::{answers::ZhihuAnswer, auto::AutoMarkdown, bilibili::article::BilibiliArticle, zhuanlans::ZhihuArticle, dispatch::UrlDispatcher};
pub use errors::{MarkResult, ZhihuError};
