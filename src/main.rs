use std::future::{self, Future};
use std::os::unix::process;
use std::pin::Pin;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use std::time::{Duration, Instant};


// struct TimerFuture {
//     start: Instant,
//     duration: Duration,
// }

// impl TimerFuture {
//     fn new(duration: Duration) -> Self {
//         Self { start: Instant::now(), duration }
//     }
// }

// impl Future for TimerFuture {
//     type Output = ();

//     fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
//         let elapsed = self.start.elapsed();

//         if elapsed >= self.duration {
//             println!("Timer completed after {:?}", elapsed);
//             Poll::Ready(())
//         } else {
//             println!("Timer not ready yet. Elapsed: {:?}, Need: {:?}", elapsed, self.duration);
//             Poll::Pending
//         }

//     }
// }

// struct CounterFuture {
//     count: u32,
//     target: u32,
// }

// impl CounterFuture {
//     fn new(target: u32) -> Self {
//         Self { count: 0, target }
//     }
// }

// impl Future for CounterFuture {
//     type Output = u32;

//     fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//         self.count += 1;
//         println!("Counter: {} / {}", self.count, self.target);

//         if self.count >= self.target {
//             Poll::Ready(self.count)
//         } else {
//             cx.waker().wake_by_ref();
//             Poll::Pending
//         }
        
//     }
// }

// executer 
fn block_on<F: Future>(mut future: F) -> F::Output {
    let mut future = unsafe {
        Pin::new_unchecked(&mut future)
    };

    let waker = dummy_waker();
    let mut context = Context::from_waker(&waker);

    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(value) => return value,
            Poll::Pending => {
                // in real executor, we would wait for a wake notification
                // for now, just spin (not efficent!)
                std::thread::sleep(Duration::from_millis(10));
            }
        }
    }
}

fn dummy_waker() -> Waker {
    fn no_op(_: *const() ) {}
    fn clone(_: *const ()) -> RawWaker {
        dummy_raw_waker()
    }

    fn dummy_raw_waker() -> RawWaker {
        RawWaker::new(
            std::ptr::null(),
            &RawWakerVTable::new(clone, no_op, no_op, no_op))
    }

    unsafe { Waker::from_raw(dummy_raw_waker())}
}


// Custom Join Handler 
struct Join<F1: Future, F2: Future> {
    future1: Option<F1>,
    future2: Option<F2>,
    output1: Option<<F1 as Future>::Output>,
    output2: Option<<F2 as Future>::Output>,
}

impl <F1: Future, F2: Future>Join<F1, F2> {
    fn new(future1: F1, future2: F2) -> Self {
        Self { future1: Some(future1), future2: Some(future2), output1: None, output2: None }
    }
}

impl <F1, F2 > Future for Join<F1, F2> 
where 
F1: Future,
F2: Future,
{
    type Output = (F1::Output, F2::Output);

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this: &mut Join<F1, F2> = unsafe {
            self.get_unchecked_mut()
        };

       // Poll first future if not complete
       if this.future1.is_some() {
        let f1 = this.future1.as_mut().unwrap();
        let pinned = unsafe {
            Pin::new_unchecked(f1)
        };

        if let Poll::Ready(output) = pinned.poll(cx) {
            this.output1 = Some(output);
            this.future1 = None;
        }
       }

       // Poll second future if not complete

       if this.future2.is_some() {
        let f2 = this.future2.as_mut().unwrap();
        let pinned = unsafe {
            Pin::new_unchecked(f2)
        };

        if let Poll::Ready(output) = pinned.poll(cx) {
            this.output2 = Some(output);
            this.future2 = None;
        }
       }


       // Both complete
       if this.output1.is_some() && this.output2.is_some() {
        let output1 = this.output1.take().unwrap();
        let output2 = this.output2.take().unwrap();
        Poll::Ready((output1, output2))
       } else {
        Poll::Pending
       }
        
    }
}


fn join<F1: Future, F2: Future>(f1: F1, f2: F2) -> Join<F1, F2> {
    Join::new(f1, f2)
}

async fn task(name: &str, duration_ms: u64) -> String {
    let start = Instant::now();
    println!("[{}] Starting", name);

    loop {
        if start.elapsed() >= Duration::from_millis(duration_ms) {
            break;
        }

        std::thread::sleep(Duration::from_millis(10));
    }

    let result = format!("{} completed in {:?}", name, start.elapsed());
    println!("[{}] Done!", name);
    result

}

async fn sequential_demo() {
    let start = Instant::now();

    let result1 = task("A", 100).await;
    let result2 = task("B", 100).await;
    let result3 = task("C", 100).await;

    println!("{}", result1);
    println!("{}",result2);
    println!("{}", result3);
    println!("total time taken: {:?}", start.elapsed());
}

async fn concurrent_demo() {
    let start = Instant::now();

    // Run task concurrently using our custom join
    let (result1, result2) = join(
        task("A", 100),
        task("B", 100)
    ).await;

    let result3 = task("c", 100).await;

    println!("{}", result1);
    println!("{}",result2);
    println!("{}", result3);

    println!("total time taken: {:?}", start.elapsed());
}

async fn nested_concurrent() {
    let start = Instant::now();

    let (result, _) = join(task("fast", 50), task("slow", 150)).await;

    println!("first result: {}", result);
    println!("total time: {:?}", start.elapsed());
}
// async await 
// async fn asyn_timer(duration: Duration) {
//     let start = Instant::now();

//     // await
//     loop {
//         if start.elapsed() >= duration {
//             break;
//         }
//     }

//     // yeild control
//     std::thread::sleep(Duration::from_millis(10));
// }


// async fn that await other async fn
// async fn fetch_data(id: i32) -> String {
//     println!("fetching data for id: {}", id);
//     asyn_timer(Duration::from_millis(100)).await;
//     format!("Data for ID: {}", id)
// }

// async fn process_data(data: String) -> String {
//     println!("Processing: {}", data);
//     asyn_timer(Duration::from_millis(50)).await;
//     format!("Processed: {}", data)
// }

// async fn save_data(data: String) {
//     println!("saving: {}", data);
//     asyn_timer(Duration::from_millis(30)).await;
//     println!("saved: {}", data);
// }

// async fn full_pipeline(id: i32) {
//     let data = fetch_data(id).await;
//     let processed = process_data(data).await;
//     save_data(processed).await;
// }

// async fn seq_operations() {
//     let start = Instant::now();

//     full_pipeline(1).await;
//     full_pipeline(2).await;
//     full_pipeline(3).await;

//     println!("sequential tool: {:?}", start.elapsed());
// }

fn main() {

    // println!("Simple Aysnc executor");
    // println!("Running a 2 second timer");
    // let result = block_on(TimerFuture::new(Duration::from_secs(2)));
    // let result = block_on(CounterFuture::new(10));
    // block_on(seq_operations());
    // println!("Result: {:?}\n", result);


    block_on(sequential_demo());

    block_on(concurrent_demo());
    
    block_on(nested_concurrent());
}
