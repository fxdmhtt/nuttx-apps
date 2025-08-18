extern crate serde;
extern crate serde_json;

use crate::runtime::{delay::delay, executor};
use std::ffi::c_void;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Person {
    name: String,
    age: u8,
}

mod binding;
mod runtime;

async fn task_template(id: u64) {
    println!("[Coroutine {id}] Task A Start");
    delay(1).await;
    println!("[Coroutine {id}] Task A Stop");

    delay(1).await;

    println!("[Coroutine {id}] Task B Start");
    delay(1).await;
    println!("[Coroutine {id}] Task B Stop");
}

fn demo_serde() {
    let john = Person {
        name: "John".to_string(),
        age: 30,
    };

    let json_str = serde_json::to_string(&john).unwrap();
    println!("{json_str}");

    let jane = Person {
        name: "Jane".to_string(),
        age: 25,
    };

    let json_str_jane = serde_json::to_string(&jane).unwrap();
    println!("{json_str_jane}");

    let json_data = r#"
        {
            "name": "Alice",
            "age": 28
        }"#;

    let alice: Person = serde_json::from_str(json_data).unwrap();
    println!("Deserialized: {} is {} years old", alice.name, alice.age);

    let pretty_json_str = serde_json::to_string_pretty(&alice).unwrap();
    println!("Pretty JSON:\n{pretty_json_str}");
}

fn demo_thread(secs: u64) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        for _ in 0..(secs / 5) {
            println!("Hello world from thread! {:?}", std::thread::current().id());
            std::thread::sleep(std::time::Duration::from_secs(5))
        }
    })
}

fn demo_tokio(secs: u64) {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            tokio::join!(
                async {
                    for _ in 0..(secs / 2) {
                        println!(
                            "Hello world from tokio 1! {:?}",
                            std::thread::current().id()
                        );
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    }
                },
                async {
                    for _ in 0..(secs / 3) {
                        println!(
                            "Hello world from tokio 2! {:?}",
                            std::thread::current().id()
                        );
                        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                    }
                }
            );
        });
}

#[no_mangle]
pub extern "C" fn demo_async_executor(ui_loop: *mut c_void) {
    assert!(!ui_loop.is_null());
    runtime::rust_register_loop(ui_loop);

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

#[allow(non_camel_case_types)]
type lv_event_code_t = u32;
#[allow(non_camel_case_types)]
type lv_event_t = c_void;

extern "C" {
    fn lv_event_get_code(e: *mut lv_event_t) -> lv_event_code_t;
    fn lv_event_get_target(e: *mut lv_event_t) -> *mut c_void;
}

event!(button_short_clicked_event_demo, e, async {
    let code = unsafe { lv_event_get_code(e) };
    let target = unsafe { lv_event_get_target(e) };

    println!("The async event {code:?} on {target:?} is invoking...");
    delay(1).await;
    println!("The async event {code:?} on {target:?} has been invoked!");
});

event!(button_long_pressed_event_demo, {
    println!("The long pressed event has been invoked!");
});

// Function hello_rust_cargo without manglng
// Bug: Cannot run twice because of random exceptions in tokio and infinite recursion stack overflow
#[no_mangle]
pub extern "C" fn hello_rust_cargo_main() {
    demo_serde();

    {
        let handle = demo_thread(30);
        demo_tokio(30);
        handle.join().unwrap();
    }
}
