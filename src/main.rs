use std::future::Future;
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


fn main() {

    println!("Simple Aysnc executor");
    // println!("Running a 2 second timer");
    // let result = block_on(TimerFuture::new(Duration::from_secs(2)));
    let result = block_on(CounterFuture::new(10));
    println!("Result: {:?}\n", result);
}
