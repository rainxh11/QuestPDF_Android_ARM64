//! Page-aware dynamic components and state rollback, corresponding to Elements/Dynamic.cs.
use crate::{
    Container, Element, Size,
    elements::{Context, Environment},
};
use anyhow::{Result, anyhow};
use std::any::Any;

pub trait IDynamicComponent: 'static {
    type State: Clone + 'static;
    fn state(&self) -> Self::State;
    fn set_state(&mut self, state: Self::State);
    fn compose(
        &mut self,
        context: &DynamicContext<'_, '_>,
    ) -> Result<DynamicComponentComposeResult>;
}
pub struct DynamicContext<'a, 'fonts> {
    pub page_number: usize,
    pub total_pages: usize,
    pub available_size: Size,
    pub(crate) context: &'a Context<'fonts>,
    pub(crate) environment: &'a Environment,
}
impl DynamicContext<'_, '_> {
    pub fn get_content_captured_positions(&self, id: &str) -> Vec<crate::PageElementLocation> {
        self.context
            .captured_positions
            .borrow()
            .iter()
            .filter(|p| p.id == id)
            .cloned()
            .collect()
    }
    pub fn create_element(&self, content: impl FnOnce(Container)) -> Result<DynamicElement> {
        let container = Container::new();
        content(container.clone());
        container.0.reset(false);
        let size = container
            .0
            .measure(Size::MAX, self.context, self.environment)?
            .size;
        Ok(DynamicElement {
            size,
            element: container.0,
        })
    }
}
#[derive(Clone)]
pub struct DynamicElement {
    pub size: Size,
    pub(crate) element: Element,
}
pub struct DynamicComponentComposeResult {
    pub content: Option<DynamicElement>,
    pub has_more_content: bool,
}
impl DynamicComponentComposeResult {
    pub fn new(content: DynamicElement, has_more_content: bool) -> Self {
        Self {
            content: Some(content),
            has_more_content,
        }
    }
}
pub(crate) trait ErasedComponent {
    fn snapshot(&self) -> Box<dyn Any>;
    fn restore(&mut self, state: &dyn Any) -> Result<()>;
    fn reset(&mut self);
    fn compose(
        &mut self,
        context: &DynamicContext<'_, '_>,
    ) -> Result<DynamicComponentComposeResult>;
}
struct Component<T: IDynamicComponent> {
    component: T,
    initial: T::State,
}
impl<T: IDynamicComponent> ErasedComponent for Component<T> {
    fn snapshot(&self) -> Box<dyn Any> {
        Box::new(self.component.state())
    }
    fn restore(&mut self, state: &dyn Any) -> Result<()> {
        self.component.set_state(
            state
                .downcast_ref::<T::State>()
                .ok_or_else(|| anyhow!("Invalid dynamic component state"))?
                .clone(),
        );
        Ok(())
    }
    fn reset(&mut self) {
        self.component.set_state(self.initial.clone());
    }
    fn compose(
        &mut self,
        context: &DynamicContext<'_, '_>,
    ) -> Result<DynamicComponentComposeResult> {
        self.component.compose(context)
    }
}
pub(crate) fn host<T: IDynamicComponent>(component: T) -> Box<dyn ErasedComponent> {
    let initial = component.state();
    Box::new(Component { component, initial })
}
