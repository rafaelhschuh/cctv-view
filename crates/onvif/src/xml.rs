use anyhow::{bail, ensure, Result};
use quick_xml::{events::Event, Reader};
#[derive(Clone, Default, Debug)]
pub(crate) struct Node {
    pub name: String,
    pub text: String,
    pub attributes: Vec<(String, String)>,
    pub children: Vec<Node>,
}
impl Node {
    pub fn child(&self, name: &str) -> Option<&Node> {
        self.children.iter().find(|n| n.name == name)
    }
    pub fn find(&self, name: &str) -> Option<&Node> {
        if self.name == name {
            return Some(self);
        }
        self.children.iter().find_map(|c| c.find(name))
    }
    pub fn value(&self, name: &str) -> String {
        self.find(name).map(|n| n.text.clone()).unwrap_or_default()
    }
    pub fn attr(&self, name: &str) -> String {
        self.attributes
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.clone())
            .unwrap_or_default()
    }
    pub fn all<'a>(&'a self, name: &str, out: &mut Vec<&'a Node>) {
        if self.name == name {
            out.push(self);
        }
        for c in &self.children {
            c.all(name, out);
        }
    }
}
pub(crate) fn parse(xml: &str) -> Result<Node> {
    ensure!(
        xml.len() <= 2 * 1024 * 1024,
        "Resposta ONVIF excede o limite."
    );
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut stack = vec![Node::default()];
    let mut nodes = 0;
    loop {
        match reader.read_event()? {
            Event::Start(e) | Event::Empty(e) => {
                let empty =
                    xml.as_bytes().get(reader.buffer_position() as usize - 2) == Some(&b'/');
                nodes += 1;
                ensure!(
                    nodes < 20000 && stack.len() < 64,
                    "XML ONVIF excede os limites."
                );
                let mut n = Node {
                    name: String::from_utf8_lossy(e.local_name().as_ref()).into(),
                    ..Default::default()
                };
                for attr in e.attributes() {
                    let a = attr?;
                    n.attributes.push((
                        String::from_utf8_lossy(a.key.local_name().as_ref()).into(),
                        a.decode_and_unescape_value(reader.decoder())?.into(),
                    ));
                }
                if empty {
                    if let Some(parent) = stack.last_mut() {
                        parent.children.push(n);
                    }
                } else {
                    stack.push(n);
                }
            }
            Event::End(_) => {
                ensure!(stack.len() > 1, "XML ONVIF inválido.");
                if let Some(n) = stack.pop() {
                    if let Some(p) = stack.last_mut() {
                        p.children.push(n);
                    }
                }
            }
            Event::Text(e) => {
                if let Some(n) = stack.last_mut() {
                    n.text
                        .push_str(&quick_xml::escape::unescape(&e.xml_content()?)?);
                }
            }
            Event::DocType(_) => bail!("DTD não permitido."),
            Event::GeneralRef(e) => {
                let text = if let Some(c) = e.resolve_char_ref()? {
                    c.to_string()
                } else {
                    match e.decode()?.as_ref() {
                        "amp" => "&",
                        "lt" => "<",
                        "gt" => ">",
                        "quot" => "\"",
                        "apos" => "'",
                        _ => bail!("Entidade XML não suportada."),
                    }
                    .to_string()
                };
                if let Some(n) = stack.last_mut() {
                    n.text.push_str(&text);
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    ensure!(stack.len() == 1, "XML ONVIF truncado.");
    stack.pop().ok_or_else(|| anyhow::anyhow!("XML vazio"))
}
pub(crate) fn escape(s: &str) -> String {
    quick_xml::escape::escape(s).into_owned()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn namespaces_and_entities() -> Result<()> {
        let n=parse("<s:Envelope><t:Profiles token=\"a&amp;b\"><t:Name>Entrada</t:Name><t:PTZConfiguration/></t:Profiles></s:Envelope>")?;
        let p = n.find("Profiles").unwrap();
        assert_eq!(p.attr("token"), "a&b");
        assert_eq!(p.value("Name"), "Entrada");
        assert!(p.find("PTZConfiguration").is_some());
        assert!(parse("<!DOCTYPE x SYSTEM 'file:///etc/passwd'><x/>").is_err());
        Ok(())
    }
}
