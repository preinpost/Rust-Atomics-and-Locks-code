pub mod unsafe_channel {
    use std::cell::UnsafeCell;
    use std::mem::MaybeUninit;
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
    use std::thread;

    pub struct Channel<T> {
        message: UnsafeCell<MaybeUninit<T>>,
        in_use: AtomicBool,
        ready: AtomicBool,
    }

    unsafe impl<T> Sync for Channel<T> where T: Send {}

    impl<T> Channel<T> {
        pub const fn new() -> Self {
            Self {
                message: UnsafeCell::new(MaybeUninit::uninit()),
                in_use: AtomicBool::new(false),
                ready: AtomicBool::new(false),
            }
        }

        pub fn send(&self, message: T) {
            if self.in_use.swap(true, Relaxed) {
                panic!("can't send more than one message!");
            }

            unsafe { (*self.message.get()).write(message) };
            self.ready.store(true, Release);
        }

        pub fn is_ready(&self) -> bool {
            self.ready.load(Relaxed)
        }

        pub fn receive(&self) -> T {
            if !self.ready.swap(false, Acquire) {
                panic!("no message available!");
            }
            unsafe { (*self.message.get()).assume_init_read() }
        }
    }

    impl<T> Drop for Channel<T> {
        fn drop(&mut self) {
            if *self.ready.get_mut() {
                unsafe { self.message.get_mut().assume_init_drop() }
            }
        }
    }

    pub fn run1() {
        let channel = Channel::new();
        let t = thread::current();
        thread::scope(|s| {
            s.spawn(|| {
                channel.send("hello world!");
                t.unpark();
            });
            while !channel.is_ready() {
                thread::park();
            }
        });
        assert_eq!(channel.receive(), "hello world!");
    }
}

pub mod safe_channel {
    use std::cell::UnsafeCell;
    use std::mem::MaybeUninit;
    use std::sync::Arc;
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
    use std::thread;

    struct Channel<T> {
        message: UnsafeCell<MaybeUninit<T>>,
        ready: AtomicBool,
    }

    pub struct Sender<T> {
        channel: Arc<Channel<T>>,
    }

    pub struct Receiver<T> {
        channel: Arc<Channel<T>>,
    }

    unsafe impl<T> Sync for Channel<T> where T: Send {}

    pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
        let a = Arc::new(Channel {
            message: UnsafeCell::new(MaybeUninit::uninit()),
            ready: AtomicBool::new(false),
        });
        (Sender { channel: a.clone() }, Receiver { channel: a })
    }

    impl<T> Sender<T> {
        pub fn send(self, message: T) {
            unsafe { (*self.channel.message.get()).write(message) };
            self.channel.ready.store(true, Release);
        }
    }

    impl<T> Receiver<T> {
        pub fn is_ready(&self) -> bool {
            self.channel.ready.load(Relaxed)
        }

        pub fn receive(self) -> T {
            if !self.channel.ready.swap(false, Acquire) {
                panic!("no message available!");
            }
            unsafe { (*self.channel.message.get()).assume_init_read() }
        }
    }

    impl<T> Drop for Channel<T> {
        fn drop(&mut self) {
            if *self.ready.get_mut() {
                unsafe { self.message.get_mut().assume_init_drop() }
            }
        }
    }

    pub fn run() {
        thread::scope(|s| {
            let (sender, receiver) = channel();
            let t = thread::current();
            s.spawn(move || {
                sender.send("hello world!");
                t.unpark();
            });
            while !receiver.is_ready() {
                thread::park();
            }

            println!("{}", receiver.receive());
            // assert_eq!(receiver.receive(), "hello world!");
        });
    }
}


pub mod safe_channel_without_arc {
    use std::cell::UnsafeCell;
    use std::mem::MaybeUninit;
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
    use std::thread;

    pub struct Channel<T> {
        message: UnsafeCell<MaybeUninit<T>>,
        ready: AtomicBool,
    }

    unsafe impl<T> Sync for Channel<T> where T: Send {}

    pub struct Sender<'a, T> {
        channel: &'a Channel<T>,
    }

    pub struct Receiver<'a, T> {
        channel: &'a Channel<T>,
    }

    impl<T> Channel<T> {
        pub const fn new() -> Self {
            Self {
                message: UnsafeCell::new(MaybeUninit::uninit()),
                ready: AtomicBool::new(false),
            }
        }

        pub fn split<'a>(&'a mut self) -> (Sender<'a, T>, Receiver<'a, T>) {
            *self = Self::new();
            (Sender { channel: self }, Receiver { channel: self })
        }
    }

    impl<T> Sender<'_, T> {
        pub fn send(self, message: T) {
            unsafe { (*self.channel.message.get()).write(message) };
            self.channel.ready.store(true, Release);
        }
    }

    impl<T> Receiver<'_, T> {
        pub fn is_ready(&self) -> bool {
            self.channel.ready.load(Relaxed)
        }

        pub fn receive(self) -> T {
            if !self.channel.ready.swap(false, Acquire) {
                panic!("no message available!");
            }
            unsafe { (*self.channel.message.get()).assume_init_read() }
        }
    }

    impl<T> Drop for Channel<T> {
        fn drop(&mut self) {
            if *self.ready.get_mut() {
                unsafe { self.message.get_mut().assume_init_drop() }
            }
        }
    }

    pub fn run3() {
        let mut channel = Channel::new();
        thread::scope(|s| {
            let (sender, receiver) = channel.split();
            let t = thread::current();
            s.spawn(move || {
                sender.send("hello world!");
                t.unpark();
            });
            while !receiver.is_ready() {
                thread::park();
            }

            assert_eq!(receiver.receive(), "hello world!");
        });
    }
}

pub mod ch5_6 {
    use std::cell::UnsafeCell;
    use std::marker::PhantomData;
    use std::mem::MaybeUninit;
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
    use std::thread;
    use std::thread::Thread;

    pub struct Channel<T> {
        message: UnsafeCell<MaybeUninit<T>>,
        ready: AtomicBool,
    }

    unsafe impl<T> Sync for Channel<T> where T: Send {}

    pub struct Sender<'a, T> {
        channel: &'a Channel<T>,
        receiving_thread: Thread,
    }

    pub struct Receiver<'a, T> {
        channel: &'a Channel<T>,
        _no_send: PhantomData<*const ()>,
    }

    impl<T> Sender<'_, T> {
        pub fn send(self, message: T) {
            unsafe { (*self.channel.message.get()).write(message) };
            self.channel.ready.store(true, Release);
            self.receiving_thread.unpark();
        }
    }

    impl<T> Receiver<'_, T> {
        pub fn is_ready(&self) -> bool {
            self.channel.ready.load(Relaxed)
        }

        pub fn receive(self) -> T {
            if !self.channel.ready.swap(false, Acquire) {
                thread::park();
            }
            unsafe { (*self.channel.message.get()).assume_init_read() }
        }
    }

    impl<T> Channel<T> {
        pub const fn new() -> Self {
            Self {
                message: UnsafeCell::new(MaybeUninit::uninit()),
                ready: AtomicBool::new(false),
            }
        }

        pub fn split(&mut self) -> (Sender<T>, Receiver<T>) {
            *self = Self::new();
            (
                Sender {
                    channel: self,
                    receiving_thread: thread::current(),
                },
                Receiver {
                    channel: self,
                    _no_send: PhantomData,
                }
            )
        }
    }

    impl<T> Drop for Channel<T> {
        fn drop(&mut self) {
            if *self.ready.get_mut() {
                unsafe { self.message.get_mut().assume_init_drop() }
            }
        }
    }

    pub fn run_5_6() {
        let mut channel = Channel::new();
        thread::scope(|s| {
            let (sender, receiver) = channel.split();
            s.spawn(move || {
                sender.send("hello world!");
            });
            assert_eq!(receiver.receive(), "hello world!");
        })
    }
}