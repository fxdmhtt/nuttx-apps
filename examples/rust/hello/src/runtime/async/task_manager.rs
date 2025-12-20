use std::{cell::RefCell, rc::Rc, time::Duration};

use async_executor::Task;
use futures::{future::LocalBoxFuture, FutureExt};

use crate::runtime::Delay;

#[derive(Debug, Default)]
pub struct TaskManager {
    tasks: Rc<RefCell<Vec<Task<()>>>>,
    _gc_task: Option<Task<()>>,
}

impl TaskManager {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn new_with_auto_gc(spawner: impl Fn(LocalBoxFuture<'static, ()>) -> Task<()>) -> Self {
        let tasks = Rc::new(RefCell::new(Vec::<Task<()>>::new()));

        let _gc_task = Some({
            let tasks = Rc::downgrade(&tasks);
            spawner(
                async move {
                    while let Some(tasks) = tasks.upgrade() {
                        tasks.borrow_mut().retain(|t| !t.is_finished());
                        // println!("TaskManager: {} tasks remaining!", tasks.borrow().len());
                        let _ = Delay::new(Duration::from_millis(1000)).await;
                    }
                }
                .boxed_local(),
            )
        });

        Self { tasks, _gc_task }
    }

    pub fn attach(&self, task: Task<()>) -> usize {
        self.tasks.borrow_mut().push(task);
        self.count()
    }

    pub fn gc(&self) -> usize {
        self.tasks.borrow_mut().retain(|t| !t.is_finished());
        self.count()
    }

    pub fn count(&self) -> usize {
        self.tasks.borrow().len()
    }

    pub fn cancel_all(&self) {
        // executor.spawn(join_all(self.tasks.borrow_mut().drain(..).map(|task| task.cancel())))
        self.tasks.borrow_mut().clear()
    }
}
