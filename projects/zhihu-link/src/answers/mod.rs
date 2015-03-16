use std::fmt::{Display, Formatter};
use crate::ZhihuResult;

use scraper::{CaseSensitivity, Html, Node, Selector};
use ego_tree::NodeRef;

#[derive(Debug)]
pub struct ZhihuAnswer {
    title: String,
    content: String,
}

impl Display for ZhihuAnswer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "# {}\n\n{}", self.title, self.content)
    }
}

impl ZhihuAnswer {
    pub fn new() -> Self {
        Self { title: "".to_string(), content: "".to_string() }
    }
    pub fn parse(&mut self, html: &str) -> ZhihuResult<()> {
        let html = Html::parse_document(html);

        self.extract_title(&html)?;
        self.extract_description(&html)?;
        self.extract_content(&html)?;


        Ok(())
    }
    pub fn extract_title(&mut self, html: &Html) -> ZhihuResult<()> {
        // #root > div > main > div > div > div:nth-child(10) > div:nth-child(2) > div > div.QuestionHeader-content > div.QuestionHeader-main > h1
        let selector = Selector::parse("h1.QuestionHeader-title").expect("invalid title selector");
        let _: Option<_> = try {
            let node = html.select(&selector).next()?;
            let text = node.first_child()?.value().as_text()?;
            self.title = text.to_string();
        };
        Ok(())
    }
    pub fn extract_description(&mut self, html: &Html) -> ZhihuResult<()> {
        // #root > div > main > div > div > div:nth-child(10) > div:nth-child(2) > div > div.QuestionHeader-content > div.QuestionHeader-main > div:nth-child(4) > div > div > div > div > span > p
        let selector = Selector::parse("div.QuestionRichText").expect("invalid description selector");
        let _: Option<_> = try {
            for node in html.select(&selector) {
                let text = node.first_child()?.value().as_text()?;
                println!("text: {:?}", text);
            }
        };
        Ok(())
    }
    pub fn extract_content(&mut self, html: &Html) -> ZhihuResult<()> {
        // div.RichContent-inner
        let selector = Selector::parse("span.CopyrightRichText-richText").expect("invalid content selector");
        let _: Option<_> = try {
            let node = html.select(&selector).next()?;
            for child in node.children() {
                child.id();
                self.read_content_node(child).ok()?;
            }
        };
        Ok(())
    }
    fn read_content_node(&mut self, node: NodeRef<Node>) -> ZhihuResult<()> {

        match node.value() {
            Node::Document => { println!("document") }
            Node::Fragment => {
                println!("fragment");
            }
            Node::Doctype(_) => {
                println!("doctype");
            }
            Node::Comment(_) => {
                println!("comment");
            }
            Node::Text(t) => {
                self.content.push_str(t.trim());
            }
            Node::Element(e) => {
                match e.name() {
                    "p" => {
                        for child in node.children() {
                            self.read_content_node(child)?;
                        }
                        self.content.push_str("\n\n");
                    }
                    "span" => {
                        for child in e.classes() {
                            let script = Selector::parse("script").expect("invalid content selector");
                            node.select(&script);
                            if child.contains("ztext-math") {
                                match e.attr("data-tex") {
                                    Some(s) => {
                                        self.content.push_str(" $$");
                                        self.content.push_str(s);
                                        self.content.push_str("$$ ");
                                    },
                                    None => {}
                                }
                            }
                            else {
                                for child in node.children() {
                                    self.read_content_node(child)?;
                                }
                            }
                        }

                    }
                    unknown => panic!("unknown element: {unknown}"),
                }
            }
            Node::ProcessingInstruction(_) => {
                println!("processing instruction");
            }
        }
        Ok(())
    }
}
