#![allow(clippy::uninlined_format_args)]

extern crate serde;
extern crate serde_json;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Person {
    name: String,
    age: u8,
}

mod delay;

async fn delay(secs: u64) {
    delay::Delay::new(std::time::Duration::from_secs(secs)).await;
}

async fn task_template(id: u64) {
    println!("[Coroutine {id}] Task A Start");
    delay(1).await;
    println!("[Coroutine {id}] Task A Stop");

    delay(1).await;

    println!("[Coroutine {id}] Task B Start");
    delay(1).await;
    println!("[Coroutine {id}] Task B Stop");
}

static EXECUTOR: async_executor::StaticExecutor = async_executor::StaticExecutor::new();

fn demo_serde() {
    let john = Person {
        name: "John".to_string(),
        age: 30,
    };

    let json_str = serde_json::to_string(&john).unwrap();
    println!("{}", json_str);

    let jane = Person {
        name: "Jane".to_string(),
        age: 25,
    };

    let json_str_jane = serde_json::to_string(&jane).unwrap();
    println!("{}", json_str_jane);

    let json_data = r#"
        {
            "name": "Alice",
            "age": 28
        }"#;

    let alice: Person = serde_json::from_str(json_data).unwrap();
    println!("Deserialized: {} is {} years old", alice.name, alice.age);

    let pretty_json_str = serde_json::to_string_pretty(&alice).unwrap();
    println!("Pretty JSON:\n{}", pretty_json_str);
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

fn demo_async_executor() {
    let task1 = EXECUTOR.spawn(async {
        println!("[Coroutine 1] Begin");
        task_template(1).await;
        println!("[Coroutine 1] End");
    });

    let task2 = EXECUTOR.spawn(async {
        println!("[Coroutine 2] Begin");
        task_template(2).await;
        println!("[Coroutine 2] End");
    });

    let task3 = EXECUTOR.spawn(async {
        println!("[Coroutine 3] Begin");

        let task1 = EXECUTOR.spawn(async { task_template(3).await });
        let task2 = EXECUTOR.spawn(async { task_template(3).await });

        futures::future::join(task1, task2).await;
        println!("[Coroutine 3] End");
    });

    let tasks = [task1, task2, task3];
    while !tasks.iter().all(|task| task.is_finished()) {
        while EXECUTOR.try_tick() {}
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

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

    demo_async_executor();
}
