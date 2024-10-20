// pub mod events;

// use std::{
//     any::{Any, TypeId},
//     cell::RefCell,
//     collections::HashMap,
//     marker::PhantomData,
// };

// pub struct Client {
//     listeners: Vec<StoredEventListener>,
// }

// impl Client {
//     pub fn new() -> Self {
//         Self {
//             listeners: Vec::new(),
//         }
//     }

//     pub fn add_listener<I, E: EventListener + 'static>(
//         &mut self,
//         listener: impl IntoEventListener<I, EventListener = E>,
//     ) -> &mut Self {
//         self.listeners
//             .push(Box::new(listener.into_event_listener()));
//         self
//     }
// }

// fn listener1() {}
// fn listener2(_: usize) {}
// fn listener3(_: usize, _: usize) {}

// fn test() {
//     let mut client = Client::new();
//     client
//         .add_listener(listener1)
//         .add_listener(listener2)
//         .add_listener(listener3);
// }

/*

struct Events {
    listeners: HashMap<Event, Vec<Box<dyn EventListener>>>
}

let mut events = Events::default();

struct TestEvent {
    a: u8,
}

fn on_test(
    event: Event<TestEvent>,
    res: Res<TestResource>,
    world: &mut World,
) {

}

events
    .add_listener(on_test)
    .trigger(TestEvent { a: 0 });

*/
