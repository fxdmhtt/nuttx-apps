use crate::runtime::{delay::delay, executor};

async fn task_template(id: u64) {
    println!("[Coroutine {id}] Task A Start");
    delay(1).await;
    println!("[Coroutine {id}] Task A Stop");

    delay(1).await;

    println!("[Coroutine {id}] Task B Start");
    delay(1).await;
    println!("[Coroutine {id}] Task B Stop");
}

#[no_mangle]
extern "C" fn demo_async_executor() {
    let task1 = executor().spawn(async {
        println!("[Coroutine 1] Begin");
        task_template(1).await;
        println!("[Coroutine 1] End");
    });

    let task2 = executor().spawn(async {
        println!("[Coroutine 2] Begin");
        task_template(2).await;
        println!("[Coroutine 2] End");
    });

    let task3 = executor().spawn(async {
        println!("[Coroutine 3] Begin");

        let task1 = executor().spawn(async { task_template(3).await });
        let task2 = executor().spawn(async { task_template(3).await });

        futures::future::join(task1, task2).await;
        println!("[Coroutine 3] End");
    });

    let tasks = [task1, task2, task3];
    tasks.into_iter().for_each(|task| task.detach());
    executor().try_tick_all();
}
