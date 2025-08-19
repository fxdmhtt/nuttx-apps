pub fn demo_thread(secs: u64) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        for _ in 0..(secs / 5) {
            println!("Hello world from thread! {:?}", std::thread::current().id());
            std::thread::sleep(std::time::Duration::from_secs(5))
        }
    })
}

pub fn demo_tokio(secs: u64) {
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
