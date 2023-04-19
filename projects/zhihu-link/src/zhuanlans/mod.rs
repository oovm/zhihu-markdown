use crate::{utils::select_text, ZhihuError, ZhihuResult};
use htmler::{Html, Node, NodeKind, Selector};
use std::{
    fmt::{Display, Formatter, Write},
    io::Write as _,
    path::Path,
    str::FromStr,
    sync::LazyLock,
};

#[derive(Debug)]
pub struct ZhihuArticle {
    title: String,
    content: String,
}

impl Default for ZhihuArticle {
    fn default() -> Self {
        Self { title: "".to_string(), content: "".to_string() }
    }
}

impl Display for ZhihuArticle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "# {}\n\n{}", self.title, self.content)
    }
}

impl FromStr for ZhihuArticle {
    type Err = ZhihuError;

    fn from_str(html: &str) -> Result<Self, Self::Err> {
        let mut empty = Self::default();
        empty.do_parse(html)?;
        Ok(empty)
    }
}
static SELECT_TITLE: LazyLock<Selector> = LazyLock::new(|| Selector::new("h1.Post-Title"));
static SELECT_CONTENT: LazyLock<Selector> = LazyLock::new(|| Selector::new("div.Post-RichTextContainer"));

impl ZhihuArticle {
    /// 通过问题 ID 和回答 ID 获取知乎回答, 并渲染为 markdown
    ///
    /// # Examples
    ///
    /// ```
    /// # use zhihu_link::ZhihuAnswer;
    /// let answer = ZhihuAnswer::new(58151047, 1).await?;
    /// ```
    pub async fn new(article: usize) -> ZhihuResult<Self> {
        let html = Self::request(article).await?;
        Ok(html.parse()?)
    }
    pub async fn request(article: usize) -> ZhihuResult<String> {
        let url = format!("https://zhuanlan.zhihu.com/p/{article}");
        let resp = reqwest::Client::new().get(url).send().await?;
        Ok(resp.text().await?)
    }
    pub fn save<P>(&self, path: P) -> ZhihuResult<()>
    where
        P: AsRef<Path>,
    {
        let mut file = std::fs::File::create(path)?;
        file.write_all(self.to_string().as_bytes())?;
        Ok(())
    }
    fn do_parse(&mut self, html: &str) -> ZhihuResult<()> {
        let html = Html::parse_document(html);
        self.extract_title(&html)?;
        self.extract_description(&html)?;
        self.extract_content(&html)?;
        Ok(())
    }

    fn extract_title(&mut self, html: &Html) -> ZhihuResult<()> {
        self.title = select_text(&html, &SELECT_TITLE).unwrap_or_default();
        Ok(())
    }
    fn extract_description(&mut self, html: &Html) -> ZhihuResult<()> {
        let selector = Selector::new("div.QuestionRichText");
        let _: Option<_> = try {
            for node in html.select(&selector) {
                let text = node.first_child()?.as_text()?;
                println!("text: {:?}", text);
            }
        };
        Ok(())
    }
    fn extract_content(&mut self, html: &Html) -> ZhihuResult<()> {
        // div.RichContent-inner
        let selector = Selector::new("span.CopyrightRichText-richText");
        let _: Option<_> = try {
            let node = html.select(&selector).next()?;
            for child in node.children() {
                self.read_content_node(child).ok()?;
            }
        };
        Ok(())
    }
    fn read_content_node(&mut self, node: Node) -> ZhihuResult<()> {
        match node.as_kind() {
            NodeKind::Document => {
                println!("document")
            }
            NodeKind::Fragment => {
                println!("fragment")
            }
            NodeKind::Doctype(_) => {
                println!("doctype")
            }
            NodeKind::Comment(_) => {
                println!("comment")
            }
            NodeKind::Text(t) => {
                self.content.push_str(t.trim());
            }
            NodeKind::Element(e) => {
                match e.name() {
                    "p" => {
                        for child in node.children() {
                            self.read_content_node(child)?;
                        }
                        self.content.push_str("\n\n");
                    }
                    "span" => {
                        // math mode
                        if e.has_class("ztext-math") {
                            match e.get_attribute("data-tex") {
                                Some(s) => {
                                    self.content.push_str(" $$");
                                    self.content.push_str(s);
                                    self.content.push_str("$$ ");
                                }
                                None => {}
                            }
                        }
                        // normal mode
                        else {
                            for child in node.children() {
                                self.read_content_node(child)?;
                            }
                        }
                    }
                    "br" => {
                        self.content.push_str("\n");
                    }
                    "figure" => {
                        for child in node.descendants().filter(|e| e.has_class("img")) {
                            let original = child.get_attribute("data-original");
                            if !original.is_empty() {
                                write!(self.content, "![]({})", original)?;
                                break;
                            }
                        }
                    }
                    unknown => panic!("unknown element: {unknown}"),
                }
            }
            NodeKind::ProcessingInstruction(_) => {
                println!("processing instruction");
            }
        }
        Ok(())
    }
}
