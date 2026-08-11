pub mod json_tree;
pub mod xml_tree;

use serde_json::Value;

pub enum BodyView {
    Json(Value),
    Xml(String),
    Text,
    Binary,
}

/// Picks how to render a response body: prefer the declared Content-Type,
/// fall back to sniffing the body's first non-whitespace character, and
/// finally fall back further to whichever parser actually succeeds.
pub fn detect(content_type: Option<&str>, body: &str, is_valid_utf8: bool) -> BodyView {
    if !is_valid_utf8 {
        return BodyView::Binary;
    }

    let trimmed = body.trim_start();
    let looks_json = trimmed.starts_with('{') || trimmed.starts_with('[');
    let looks_xml = trimmed.starts_with('<');

    let content_type = content_type.unwrap_or_default();
    let prefer_json = content_type.contains("json") || (content_type.is_empty() && looks_json);
    let prefer_xml = content_type.contains("xml") || (content_type.is_empty() && looks_xml);

    if prefer_json && serde_json::from_str::<Value>(body).is_ok() {
        return BodyView::Json(serde_json::from_str(body).expect("checked above"));
    }
    if prefer_xml && roxmltree::Document::parse(body).is_ok() {
        return BodyView::Xml(body.to_string());
    }
    if let Ok(value) = serde_json::from_str::<Value>(body) {
        return BodyView::Json(value);
    }
    if roxmltree::Document::parse(body).is_ok() {
        return BodyView::Xml(body.to_string());
    }
    BodyView::Text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_json_by_content_type() {
        let view = detect(Some("application/json"), r#"{"a":1}"#, true);
        assert!(matches!(view, BodyView::Json(_)));
    }

    #[test]
    fn detects_json_by_sniffing_when_no_content_type() {
        let view = detect(None, r#"{"a":1}"#, true);
        assert!(matches!(view, BodyView::Json(_)));
    }

    #[test]
    fn detects_xml_by_content_type() {
        let view = detect(Some("application/xml"), "<root><a>1</a></root>", true);
        assert!(matches!(view, BodyView::Xml(_)));
    }

    #[test]
    fn detects_xml_by_sniffing_when_no_content_type() {
        let view = detect(None, "<root><a>1</a></root>", true);
        assert!(matches!(view, BodyView::Xml(_)));
    }

    #[test]
    fn falls_back_to_text_for_plain_body() {
        let view = detect(Some("text/plain"), "just some log lines", true);
        assert!(matches!(view, BodyView::Text));
    }

    #[test]
    fn invalid_utf8_is_always_binary_regardless_of_content_type() {
        let view = detect(Some("application/json"), "garbage", false);
        assert!(matches!(view, BodyView::Binary));
    }

    #[test]
    fn json_tree_index_covers_nested_and_top_level_paths() {
        let value: Value = serde_json::from_str(r#"{"user":{"name":"Ada","tags":["admin","staff"]}}"#).unwrap();
        let tree = json_tree::build(&value);
        assert_eq!(tree.items.len(), 1, "one top-level key");
        assert!(tree.index.iter().any(|(_, text)| text.contains("Ada")));
        assert!(tree.index.iter().any(|(_, text)| text.contains("admin")));
        let ada_path = tree
            .index
            .iter()
            .find(|(_, text)| text.contains("Ada"))
            .map(|(path, _)| path.clone())
            .unwrap();
        assert_eq!(ada_path, vec![0, 0]);
    }

    #[test]
    fn xml_tree_index_covers_nested_elements() {
        let doc = roxmltree::Document::parse("<root><child>value</child></root>").unwrap();
        let tree = xml_tree::build(&doc);
        assert_eq!(tree.items.len(), 1, "single root element");
        assert!(tree.index.iter().any(|(_, text)| text.contains("child")));
    }
}
