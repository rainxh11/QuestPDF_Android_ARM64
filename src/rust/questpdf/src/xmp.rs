//! Counterpart of Qpdf/XmpMetadataExtension.cs, preserving RDF namespaces.
use anyhow::{Result, anyhow, bail};
use xmltree::{Element, EmitterConfig, ParserConfig, XMLNode};
const RDF: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
const EXT: &str = "http://www.aiim.org/pdfa/ns/extension/";
fn named(e: &Element, name: &str, namespace: &str) -> bool {
    e.name == name && e.namespace.as_deref() == Some(namespace)
}
fn find<'a>(e: &'a mut Element, name: &str, namespace: &str) -> Option<&'a mut Element> {
    if named(e, name, namespace) {
        return Some(e);
    }
    for child in &mut e.children {
        if let XMLNode::Element(e) = child
            && let Some(found) = find(e, name, namespace)
        {
            return Some(found);
        }
    }
    None
}
fn parse(bytes: &[u8]) -> Result<Vec<XMLNode>> {
    if bytes.windows(9).any(|v| v == b"<!DOCTYPE") {
        bail!("XMP metadata cannot contain a DTD");
    }
    Ok(Element::parse_all_with_config(
        bytes,
        ParserConfig::new()
            .trim_whitespace(false)
            .whitespace_to_characters(true)
            .ignore_comments(false),
    )?)
}
pub(crate) fn extend(bytes: &[u8], contents: &[String]) -> Result<Vec<u8>> {
    let mut nodes = parse(bytes)?;
    let root = nodes
        .iter_mut()
        .find_map(XMLNode::as_mut_element)
        .ok_or_else(|| anyhow!("XMP has no document element"))?;
    let rdf = find(root, "RDF", RDF)
        .ok_or_else(|| anyhow!("The XMP metadata does not contain rdf:RDF"))?;
    for content in contents {
        let wrapper = format!("<rdf:RDF xmlns:rdf=\"{RDF}\">{content}</rdf:RDF>");
        let mut parsed = parse(wrapper.as_bytes())?;
        let parsed = parsed.iter_mut().find_map(XMLNode::as_mut_element).unwrap();
        for mut node in std::mem::take(&mut parsed.children) {
            if let XMLNode::Element(description) = &mut node {
                let schema_index = description
                    .children
                    .iter()
                    .position(|n| n.as_element().is_some_and(|e| named(e, "schemas", EXT)));
                if let Some(index) = schema_index {
                    let new_bag = description.children[index].as_mut_element().and_then(|s| {
                        s.children
                            .iter_mut()
                            .filter_map(XMLNode::as_mut_element)
                            .find(|e| named(e, "Bag", RDF))
                    });
                    if let Some(new_bag) = new_bag
                        && let Some(schemas) = find(rdf, "schemas", EXT)
                        && let Some(existing_bag) = schemas
                            .children
                            .iter_mut()
                            .filter_map(XMLNode::as_mut_element)
                            .find(|e| named(e, "Bag", RDF))
                    {
                        existing_bag.children.extend(
                            new_bag
                                .children
                                .iter()
                                .filter(|n| matches!(n, XMLNode::Element(_)))
                                .cloned(),
                        );
                        description.children.remove(index);
                        if !description
                            .children
                            .iter()
                            .any(|n| matches!(n, XMLNode::Element(_)))
                        {
                            continue;
                        }
                    }
                }
            }
            rdf.children.push(node);
        }
    }
    let mut output = vec![];
    for node in nodes {
        match node {
            XMLNode::Element(e) => e.write_with_config(
                &mut output,
                EmitterConfig::new()
                    .perform_indent(false)
                    .write_document_declaration(false),
            )?,
            XMLNode::ProcessingInstruction(name, value) => output.extend_from_slice(
                format!(
                    "<?{name}{}?>",
                    value.map(|v| format!(" {v}")).unwrap_or_default()
                )
                .as_bytes(),
            ),
            XMLNode::Comment(v) => output.extend_from_slice(format!("<!--{v}-->").as_bytes()),
            XMLNode::Text(v) => output.extend_from_slice(v.as_bytes()),
            _ => {}
        }
    }
    Ok(output)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn merges_extension_schema_bags_and_preserves_packet() {
        let packet = format!(
            "<?xpacket begin=\"\"?><x:xmpmeta xmlns:x=\"adobe:ns:meta/\"><rdf:RDF xmlns:rdf=\"{RDF}\"><rdf:Description xmlns:e=\"{EXT}\"><e:schemas><rdf:Bag><rdf:li>one</rdf:li></rdf:Bag></e:schemas></rdf:Description></rdf:RDF></x:xmpmeta><?xpacket end=\"w\"?>"
        );
        let fragment = format!(
            "<rdf:Description xmlns:e=\"{EXT}\"><e:schemas><rdf:Bag><rdf:li>two</rdf:li></rdf:Bag></e:schemas></rdf:Description><rdf:Description xmlns:test=\"urn:test\"><test:value>ok</test:value></rdf:Description>"
        );
        let output = extend(packet.as_bytes(), &[fragment]).unwrap();
        let text = String::from_utf8(output.clone()).unwrap();
        assert!(text.contains("xpacket"));
        assert!(text.contains("urn:test"));
        let mut nodes = parse(&output).unwrap();
        let root = nodes.iter_mut().find_map(XMLNode::as_mut_element).unwrap();
        let bag = find(root, "Bag", RDF).unwrap();
        assert_eq!(
            bag.children
                .iter()
                .filter(|n| n.as_element().is_some())
                .count(),
            2
        );
        assert!(extend(b"<!DOCTYPE x><x/>", &[]).is_err());
        assert!(extend(b"<x/>", &["<broken>".into()]).is_err());
    }
}
