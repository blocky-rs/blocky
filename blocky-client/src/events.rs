use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
    marker::PhantomData,
};

pub type Resources = HashMap<TypeId, RefCell<Box<dyn Any>>>;

pub trait EventListener {
    // TODO: add resources
    fn run(&mut self, resources: &mut Resources);
}

pub struct FunctionEventListener<I, F> {
    f: F,
    marker: PhantomData<fn() -> I>,
}

pub trait IntoEventListener<I> {
    type EventListener: EventListener;

    fn into_event_listener(self) -> Self::EventListener;
}

