use std::future::Future;
use std::os::unix::process;
use std::pin::Pin;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use std::time::{Duration, Instant};


struct TimerFuture {
    start: Instant,
    duration: Duration,
}

impl TimerFuture {
    fn new(duration: Duration) -> Self {
        Self { start: Instant::now(), duration }
    }
}

impl Future for TimerFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let elapsed = self.start.elapsed();

        if elapsed >= self.duration {
            println!("Timer completed after {:?}", elapsed);
            Poll::Ready(())
        } else {
            println!("Timer not ready yet. Elapsed: {:?}, Need: {:?}", elapsed, self.duration);
            Poll::Pending
        }

    }
}

struct CounterFuture {
    count: u32,
    target: u32,
}

impl CounterFuture {
    fn new(target: u32) -> Self {
        Self { count: 0, target }
    }
}

impl Future for CounterFuture {
    type Output = u32;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.count += 1;
        println!("Counter: {} / {}", self.count, self.target);

        if self.count >= self.target {
            Poll::Ready(self.count)
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
        
    }
}

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



// async await 
async fn asyn_timer(duration: Duration) {
    let start = Instant::now();

    // await
    loop {
        if start.elapsed() >= duration {
            break;
        }
    }

    // yeild control
    std::thread::sleep(Duration::from_millis(10));
}


// async fn that await other async fn
async fn fetch_data(id: i32) -> String {
    println!("fetching data for id: {}", id);
    asyn_timer(Duration::from_millis(100)).await;
    format!("Data for ID: {}", id)
}

async fn process_data(data: String) -> String {
    println!("Processing: {}", data);
    asyn_timer(Duration::from_millis(50)).await;
    format!("Processed: {}", data)
}

async fn save_data(data: String) {
    println!("saving: {}", data);
    asyn_timer(Duration::from_millis(30)).await;
    println!("saved: {}", data);
}

async fn full_pipeline(id: i32) {
    let data = fetch_data(id).await;
    let processed = process_data(data).await;
    save_data(processed).await;
}

async fn seq_operations() {
    let start = Instant::now();

    full_pipeline(1).await;
    full_pipeline(2).await;
    full_pipeline(3).await;

    println!("sequential tool: {:?}", start.elapsed());
}

fn main() {

    println!("Simple Aysnc executor");
    // println!("Running a 2 second timer");
    // let result = block_on(TimerFuture::new(Duration::from_secs(2)));
    // let result = block_on(CounterFuture::new(10));
    block_on(seq_operations());
    // println!("Result: {:?}\n", result);
}
