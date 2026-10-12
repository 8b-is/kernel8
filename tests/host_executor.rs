extern crate alloc;
#[path = "../kernel/src/task/mod.rs"]
mod task;
use std::{cell::RefCell, rc::Rc};
use task::{simple_executor::SimpleExecutor, yield_now::yield_now, Task};

#[test]
fn tasks_interleave_and_complete_once() {
    let events = Rc::new(RefCell::new(Vec::new()));
    let mut executor = SimpleExecutor::new();
    for name in ["a", "b"] {
        let events = events.clone();
        executor.spawn(Task::new(async move {
            events.borrow_mut().push((name, 1));
            yield_now().await;
            events.borrow_mut().push((name, 2));
        }));
    }
    executor.run();
    assert_eq!(*events.borrow(), [("a", 1), ("b", 1), ("a", 2), ("b", 2)]);
    executor.run();
    assert_eq!(events.borrow().len(), 4, "completed tasks must not run again");
}

#[test]
fn empty_executor_returns() {
    SimpleExecutor::new().run();
}
