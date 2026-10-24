//! Native picture layers and matrix transfer, mirroring SkiaDrawingCanvas's ZIndex canvases.
use crate::{
    Size,
    native::{Api, Handle, Ptr},
};
use anyhow::{Result, bail};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    rc::{Rc, Weak},
    sync::Arc,
};
thread_local! { static SESSIONS: RefCell<std::collections::HashMap<usize,Weak<RefCell<State>>>> = RefCell::new(std::collections::HashMap::new()); }
struct Layer {
    recorder: Handle,
    canvas: Ptr,
}
struct State {
    api: Arc<Api>,
    size: Size,
    current: i32,
    semantic_id: i32,
    layers: BTreeMap<i32, Layer>,
    managed_matrix: Option<crate::native::Matrix>,
    matrix_stack: Vec<crate::native::Matrix>,
}
pub(crate) struct CanvasSession {
    state: Rc<RefCell<State>>,
    pub ptr: Ptr,
}
impl Layer {
    fn new(api: &Arc<Api>, size: Size) -> Result<Self> {
        unsafe {
            let recorder = Handle::new(
                (api.questpdf_skia_picture_recorder_create)(),
                api.questpdf_skia_picture_recorder_delete,
                api,
            )?;
            let canvas = (api.questpdf_skia_picture_recorder_begin_recording)(
                recorder.ptr,
                size.width,
                size.height,
            );
            if canvas.is_null() {
                bail!("Cannot begin native canvas layer");
            }
            Ok(Self { recorder, canvas })
        }
    }
}
impl CanvasSession {
    pub fn new(api: &Arc<Api>, size: Size) -> Result<Self> {
        let layer = Layer::new(api, size)?;
        let ptr = layer.canvas;
        let mut layers = BTreeMap::new();
        layers.insert(0, layer);
        let state = Rc::new(RefCell::new(State {
            api: api.clone(),
            size,
            current: 0,
            semantic_id: 0,
            layers,
            managed_matrix: None,
            matrix_stack: vec![],
        }));
        SESSIONS.with(|s| s.borrow_mut().insert(ptr as usize, Rc::downgrade(&state)));
        Ok(Self { state, ptr })
    }
    pub fn enable_managed_matrix(&self, matrix: crate::native::Matrix) {
        self.state.borrow_mut().managed_matrix = Some(matrix);
        unsafe {
            (self.state.borrow().api.questpdf_skia_canvas_set_matrix9)(route(self.ptr), &matrix);
        }
    }
    pub fn finish(self) -> Result<Vec<Handle>> {
        Ok(self
            .finish_snapshot()?
            .into_iter()
            .map(|(_, p)| p)
            .collect())
    }
    pub fn finish_snapshot(self) -> Result<Vec<(i32, Handle)>> {
        SESSIONS.with(|s| s.borrow_mut().remove(&(self.ptr as usize)));
        let mut state = self.state.borrow_mut();
        let layers = std::mem::take(&mut state.layers);
        let api = &state.api;
        layers
            .into_iter()
            .map(|(index, layer)| unsafe {
                Ok((
                    index,
                    Handle::new(
                        (api.questpdf_skia_picture_recorder_end_recording)(layer.recorder.ptr),
                        api.questpdf_skia_picture_unref,
                        api,
                    )?,
                ))
            })
            .collect()
    }
}
pub(crate) enum Transform {
    Save,
    Restore,
    Translate(f32, f32),
    Scale(f32, f32),
    Rotate(f32),
    Set(crate::native::Matrix),
}
#[allow(clippy::assign_op_pattern)] // Keep the Matrix4x4 multiplication operand order.
pub(crate) fn transform(canvas: Ptr, operation: Transform) -> bool {
    let state = SESSIONS.with(|s| s.borrow().get(&(canvas as usize)).and_then(Weak::upgrade));
    let Some(state) = state else {
        return false;
    };
    let mut state = state.borrow_mut();
    let Some(mut m) = state.managed_matrix else {
        return false;
    };
    match operation {
        Transform::Save => state.matrix_stack.push(m),
        Transform::Restore => {
            m = state
                .matrix_stack
                .pop()
                .expect("balanced canvas save/restore")
        }
        Transform::Set(matrix) => m = matrix,
        Transform::Translate(x, y) => {
            m.translate_x = ((x * m.scale_x) + (y * m.skew_x)) + m.translate_x;
            m.translate_y = ((x * m.skew_y) + (y * m.scale_y)) + m.translate_y;
        }
        Transform::Scale(x, y) => {
            m.scale_x *= x;
            m.skew_y *= x;
            m.skew_x *= y;
            m.scale_y *= y;
        }
        Transform::Rotate(angle) => {
            let angle = std::f32::consts::PI * angle / 180.;
            let (sin, cos) = angle.sin_cos();
            let old = m;
            m.scale_x = cos * old.scale_x + sin * old.skew_x;
            m.skew_y = cos * old.skew_y + sin * old.scale_y;
            m.skew_x = -sin * old.scale_x + cos * old.skew_x;
            m.scale_y = -sin * old.skew_y + cos * old.scale_y;
        }
    }
    state.managed_matrix = Some(m);
    unsafe {
        (state.api.questpdf_skia_canvas_set_matrix9)(state.layers[&state.current].canvas, &m);
    }
    true
}
pub(crate) fn draw_snapshot(canvas: Ptr, layers: &[(i32, Handle)], api: &Arc<Api>) -> Result<()> {
    let state = SESSIONS.with(|s| s.borrow().get(&(canvas as usize)).and_then(Weak::upgrade));
    let Some(state) = state else {
        return Ok(());
    };
    let mut state = state.borrow_mut();
    let identity = crate::native::Matrix {
        scale_x: 1.,
        scale_y: 1.,
        perspective3: 1.,
        ..Default::default()
    };
    for (index, picture) in layers {
        if !state.layers.contains_key(index) {
            let layer = Layer::new(api, state.size)?;
            state.layers.insert(*index, layer);
        }
        let target = state.layers[index].canvas;
        unsafe {
            (api.questpdf_skia_canvas_save)(target);
            (api.questpdf_skia_canvas_set_matrix9)(target, &identity);
            (api.questpdf_skia_canvas_draw_picture)(target, picture.ptr);
            (api.questpdf_skia_canvas_restore)(target);
        }
    }
    Ok(())
}
impl Drop for CanvasSession {
    fn drop(&mut self) {
        SESSIONS.with(|s| s.borrow_mut().remove(&(self.ptr as usize)));
    }
}
pub(crate) fn route(canvas: Ptr) -> Ptr {
    SESSIONS.with(|s| {
        s.borrow()
            .get(&(canvas as usize))
            .and_then(Weak::upgrade)
            .map(|s| {
                let s = s.borrow();
                s.layers[&s.current].canvas
            })
            .unwrap_or(canvas)
    })
}
pub(crate) fn z_index(canvas: Ptr) -> i32 {
    SESSIONS.with(|s| {
        s.borrow()
            .get(&(canvas as usize))
            .and_then(Weak::upgrade)
            .map(|s| s.borrow().current)
            .unwrap_or(0)
    })
}
pub(crate) fn set_z_index(canvas: Ptr, index: i32) -> Result<()> {
    let state = SESSIONS.with(|s| s.borrow().get(&(canvas as usize)).and_then(Weak::upgrade));
    let Some(state) = state else {
        return Ok(());
    };
    let mut state = state.borrow_mut();
    let api = state.api.clone();
    let mut matrix = crate::native::Matrix::default();
    unsafe {
        (api.questpdf_skia_canvas_get_matrix9)(state.layers[&state.current].canvas, &mut matrix);
    }
    if !state.layers.contains_key(&index) {
        let layer = Layer::new(&api, state.size)?;
        unsafe {
            (api.questpdf_skia_canvas_set_semantic_node_id)(layer.canvas, state.semantic_id);
        }
        state.layers.insert(index, layer);
    }
    state.current = index;
    unsafe {
        (api.questpdf_skia_canvas_set_matrix9)(state.layers[&index].canvas, &matrix);
    }
    Ok(())
}
pub(crate) fn semantic_id(canvas: Ptr) -> i32 {
    SESSIONS.with(|s| {
        s.borrow()
            .get(&(canvas as usize))
            .and_then(Weak::upgrade)
            .map(|s| s.borrow().semantic_id)
            .unwrap_or(0)
    })
}
pub(crate) fn set_semantic_id(canvas: Ptr, id: i32) {
    let state = SESSIONS.with(|s| s.borrow().get(&(canvas as usize)).and_then(Weak::upgrade));
    if let Some(state) = state {
        let mut state = state.borrow_mut();
        state.semantic_id = id;
        for layer in state.layers.values() {
            unsafe {
                (state.api.questpdf_skia_canvas_set_semantic_node_id)(layer.canvas, id);
            }
        }
    }
}
