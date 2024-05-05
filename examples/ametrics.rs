use anyhow::Result;
use rand::Rng;
use rust_concurrency::AmapMetrics;
use std::thread;

const N: usize = 2;
const M: usize = 4;

fn main() -> Result<()> {
    let metrics = AmapMetrics::new(&[
        "call.thread.task_worker.0",
        "call.thread.task_worker.1",
        "req.page.0",
        "req.page.1",
        "req.page.2",
        "req.page.3",
    ]);
    println!("{}", metrics);

    for idx in 0..N {
        task_worker(idx, metrics.clone())?; //Metrics{Arc::clone(&metrics.data)}
    }

    for _ in 0..M {
        request_worker(metrics.clone())?; //Metrics{Arc::clone(&metrics.data)}
    }

    loop {
        thread::sleep(std::time::Duration::from_secs(5));
        println!("{}", metrics);
    }
    #[allow(unreachable_code)]
    Ok(())
}

fn task_worker(idx: usize, metrics: AmapMetrics) -> Result<()> {
    thread::spawn(move || {
        loop {
            let mut rng = rand::thread_rng();
            thread::sleep(std::time::Duration::from_millis(rng.gen_range(100..5000)));
            metrics.inc(format!("call.thread.task_worker.{}", idx))?;
        }
        #[allow(unreachable_code)]
        Ok::<_, anyhow::Error>(())
    });
    Ok(())
}

fn request_worker(metrics: AmapMetrics) -> Result<()> {
    thread::spawn(move || {
        loop {
            let mut rng = rand::thread_rng();
            thread::sleep(std::time::Duration::from_millis(rng.gen_range(50..800)));
            let page = rng.gen_range(0..5);
            metrics.inc(format!("req.page.{}", page))?;
        }
        #[allow(unreachable_code)]
        Ok::<_, anyhow::Error>(())
    });
    Ok(())
}
