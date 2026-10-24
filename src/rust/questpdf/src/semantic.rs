//! Logical PDF structure tree and native tag ownership, corresponding to Drawing/SemanticTreeManager.
use crate::{
    elements::Context,
    native::{self, Api, Handle, Ptr},
};
use anyhow::{Result, anyhow};
use std::{cell::RefCell, ffi::CString, ptr, sync::Arc};
#[derive(Clone, Debug)]
pub enum SemanticAttributeValue {
    Text(String),
    Integer(i32),
    Float(f32),
    FloatArray(Vec<f32>),
    NodeIds(Vec<i32>),
}
#[derive(Clone, Debug)]
pub struct SemanticAttribute {
    pub owner: String,
    pub name: String,
    pub value: SemanticAttributeValue,
}
#[derive(Clone, Debug)]
pub struct SemanticInfo {
    pub kind: String,
    pub alternative_text: Option<String>,
    pub language: Option<String>,
    pub attributes: Vec<SemanticAttribute>,
}
impl SemanticInfo {
    pub fn new(kind: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            alternative_text: None,
            language: None,
            attributes: vec![],
        }
    }
}
pub(crate) struct Node {
    pub info: SemanticInfo,
    pub children: Vec<usize>,
}
pub(crate) struct SemanticTree {
    pub nodes: Vec<Node>,
    pub stack: Vec<usize>,
    pub counter: usize,
    pub artifacts: usize,
}
impl SemanticTree {
    pub fn new() -> Self {
        Self {
            nodes: vec![Node {
                info: SemanticInfo::new("Document"),
                children: vec![],
            }],
            stack: vec![1],
            counter: 1,
            artifacts: 0,
        }
    }
    pub fn register(&mut self, info: SemanticInfo, initial: bool) -> usize {
        self.counter += 1;
        let id = self.counter;
        if initial {
            self.nodes.push(Node {
                info,
                children: vec![],
            });
            let parent = *self.stack.last().unwrap_or(&1);
            self.nodes[parent - 1].children.push(id);
        }
        id
    }
    pub fn paragraph_suppressed(&self) -> bool {
        self.artifacts > 0
            || self.stack.iter().any(|id| {
                matches!(
                    self.nodes.get(id - 1).map(|n| n.info.kind.as_str()),
                    Some("H" | "H1" | "H2" | "H3" | "H4" | "H5" | "H6" | "P" | "Lbl")
                )
            })
    }
    pub fn native(&self, api: &Arc<Api>) -> Result<NativeSemanticTree> {
        let mut strings = vec![];
        fn text(value: &str, strings: &mut Vec<CString>) -> *const std::ffi::c_char {
            strings.push(native::cstr(value));
            strings.last().unwrap().as_ptr()
        }
        fn create(
            tree: &SemanticTree,
            id: usize,
            api: &Arc<Api>,
            strings: &mut Vec<CString>,
        ) -> Result<Handle> {
            let node = tree
                .nodes
                .get(id - 1)
                .ok_or_else(|| anyhow!("Invalid semantic node id {id}"))?;
            let kind = native::cstr(&node.info.kind);
            let alt = node.info.alternative_text.as_ref().map(|v| native::cstr(v));
            let lang = node.info.language.as_ref().map(|v| native::cstr(v));
            unsafe {
                let element = Handle::new(
                    (api.questpdf_skia_pdf_structure_element_create)(
                        id as i32,
                        kind.as_ptr(),
                        alt.as_ref().map(|s| s.as_ptr()).unwrap_or(ptr::null()),
                        lang.as_ref().map(|s| s.as_ptr()).unwrap_or(ptr::null()),
                    ),
                    api.questpdf_skia_pdf_structure_element_delete,
                    api,
                )?;
                let children = node
                    .children
                    .iter()
                    .map(|id| create(tree, *id, api, strings))
                    .collect::<Result<Vec<_>>>()?;
                if !children.is_empty() {
                    let pointers: Vec<_> = children.iter().map(|c| c.ptr).collect();
                    (api.questpdf_skia_pdf_structure_element_set_children)(
                        element.ptr,
                        pointers.as_ptr(),
                        pointers.len() as i32,
                    );
                    for child in children {
                        child.into_raw();
                    }
                }
                for attribute in &node.info.attributes {
                    let owner = text(&attribute.owner, strings);
                    let name = text(&attribute.name, strings);
                    match &attribute.value {
                        SemanticAttributeValue::Text(v) => (api
                            .questpdf_skia_pdf_structure_element_add_attribute_text)(
                            element.ptr,
                            owner,
                            name,
                            text(v, strings),
                        ),
                        SemanticAttributeValue::Integer(v) => (api
                            .questpdf_skia_pdf_structure_element_add_attribute_integer)(
                            element.ptr,
                            owner,
                            name,
                            *v,
                        ),
                        SemanticAttributeValue::Float(v) => (api
                            .questpdf_skia_pdf_structure_element_add_attribute_float)(
                            element.ptr,
                            owner,
                            name,
                            *v,
                        ),
                        SemanticAttributeValue::FloatArray(v) => (api
                            .questpdf_skia_pdf_structure_element_add_attribute_float_array)(
                            element.ptr,
                            owner,
                            name,
                            v.as_ptr(),
                            v.len() as i32,
                        ),
                        SemanticAttributeValue::NodeIds(v) => (api
                            .questpdf_skia_pdf_structure_element_add_attribute_node_ids)(
                            element.ptr,
                            owner,
                            name,
                            v.as_ptr(),
                            v.len() as i32,
                        ),
                    }
                }
                Ok(element)
            }
        }
        let root = create(self, 1, api, &mut strings)?;
        Ok(NativeSemanticTree {
            root,
            _strings: strings,
        })
    }
}
pub(crate) struct NativeSemanticTree {
    pub root: Handle,
    _strings: Vec<CString>,
}
pub(crate) struct Scope<'a> {
    tree: &'a RefCell<Option<SemanticTree>>,
    canvas: Ptr,
    previous: i32,
    artifact: bool,
}
impl<'a> Scope<'a> {
    pub fn node(c: &'a Context<'_>, canvas: Ptr, id: usize) -> Self {
        let previous = crate::canvas::semantic_id(canvas);
        if let Some(tree) = c.semantic_tree.borrow_mut().as_mut() {
            tree.stack.push(id);
        }
        crate::canvas::set_semantic_id(canvas, id as i32);
        Self {
            tree: c.semantic_tree,
            canvas,
            previous,
            artifact: false,
        }
    }
    pub fn artifact(c: &'a Context<'_>, canvas: Ptr, id: i32) -> Self {
        let previous = crate::canvas::semantic_id(canvas);
        if let Some(tree) = c.semantic_tree.borrow_mut().as_mut() {
            tree.artifacts += 1;
        }
        crate::canvas::set_semantic_id(canvas, id);
        Self {
            tree: c.semantic_tree,
            canvas,
            previous,
            artifact: true,
        }
    }
}
impl Drop for Scope<'_> {
    fn drop(&mut self) {
        if let Some(tree) = self.tree.borrow_mut().as_mut() {
            if self.artifact {
                tree.artifacts -= 1;
            } else {
                tree.stack.pop();
            }
        }
        crate::canvas::set_semantic_id(self.canvas, self.previous);
    }
}
pub(crate) struct CounterScope<'a> {
    tree: &'a RefCell<Option<SemanticTree>>,
    previous: Option<usize>,
}
impl<'a> CounterScope<'a> {
    pub fn new(c: &'a Context<'_>, snapshots: &mut Vec<usize>, index: usize) -> Self {
        let mut tree = c.semantic_tree.borrow_mut();
        let mut previous = None;
        if let Some(tree) = tree.as_mut() {
            let original = tree.counter;
            if let Some(snapshot) = snapshots.get(index) {
                tree.counter = *snapshot;
            } else {
                snapshots.push(original);
            }
            if c.total > 0 {
                previous = Some(original);
            }
        }
        Self {
            tree: c.semantic_tree,
            previous,
        }
    }
}
impl Drop for CounterScope<'_> {
    fn drop(&mut self) {
        if let Some(previous) = self.previous
            && let Some(tree) = self.tree.borrow_mut().as_mut()
        {
            tree.counter = previous;
        }
    }
}
