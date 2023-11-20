/* 下面内容来自于https://huonw.github.io/blog/2015/05/defaulting-to-thread-safety/ */



/*
这段文本讨论了 Rust 语言中的 "Opt-In Built-In Trait"（OIBIT）模型，特别是关于 `Send` trait 的实现与优化。`Send` trait 是 Rust 中的一个自动实现的 trait，表示类型的值可以安全地在不同线程之间传递。这里的关键点是：

1. **自动实现和手动实现**：
   - 1.1 当一个类型的所有内容都实现了 `Send`，那么这个类型也会自动实现 `Send`。
   - 1.2 然而，有些情况下这种自动实现不是完美的。例如，即使一个类型只包含原始类型（通常是线程安全的），也可能使用 `unsafe` 代码创建一个线程不安全的类型。
   - 1.3 反过来，也可能有一个由非 `Send` 类型构成的类型，通过在数据操作上施加额外约束而实际上是线程安全的。

2. **强制退出和进入 (`opt-out` 和 `opt-in`)**：
   - 2.1 OIBIT 提案的两个重要部分是：填补默认实现没有覆盖的“缺口”，以及强制退出默认实现。
   - 退出（`opt-out`）是通过负实现（negative implementations）完成的，即通过实现 `!Trait` 来表明不实现某个 trait。例如，`Rc<T>` 明确声明它不实现 `Send`。
   - 进入（`opt-in`）则是指在包含非 `Send` 类型的新结构体中显式实现 `Send`，尽管这通常不会成功，因为这可能导致未定义行为（如数据竞争）。

3. **安全性和闭包**：
   - Rust 通常默认安全性（有时是保守的），特别是在低级情况下。例如，标准库中的原始指针类型不实现 `Send`，因此含有这些类型的数据结构需要显式声明它们实际上是线程安全的。
   - 闭包的类型是不可命名的，因此无法退出默认的 trait（如 `Send`）。这意味着即使闭包只捕获 `Send` 类型，它也可能使用 `unsafe` 代码变得线程不安全。不过，这种情况很少发生，因为标准库中的负实现通常能捕获大多数情况。

总的来说，这段文本讨论了 Rust 中处理并发和线程安全时的一些高级特性和实践，特别是如何通过 `Send` trait 和相关机制来确保类型在多线程环境中的安全使用。

*/

use std::{sync::Arc, thread, time::Duration};

/* 1.1 下面代码将rc move 会报错,因为rc 实现了!Send 不能move */
#[test]
fn test_oibit() {
    use std::rc::Rc;

    // can only be used with `Send` types.
    fn check_send<T: Send>(_: T) {}

    let x: i32 = 1;
    let vec: Vec<Option<String>> = vec![None, None, None];

    let f = move || {
        let _p = (x, vec); // make sure they're captured
    };
    check_send(f);

    let pointer: Rc<i32> = Rc::new(1);
    let g = move || {
        let _ = pointer; // make sure it is captured
    };
    check_send(g);
}

/* 1.2 然而，有些情况下这种自动实现不是完美的。例如，即使一个类型只包含原始类型（通常是线程安全的），也可能使用 `unsafe` 代码创建一个线程不安全的类型。nightly版本 */
/* #[test]
fn test_unsafe_auto() {
    use std::sync::Arc;
    use std::thread;

    // 简单的结构体，只包含一个原始类型（通常是线程安全的）。
    struct ThreadUnsafe {
        data: i32,
    }

    unsafe impl Send for ThreadUnsafe {}

    fn main() {
        // 创建一个新的 ThreadUnsafe 实例。
        let unsafe_instance = ThreadUnsafe { data: 42 };

        // 将该实例封装在 Arc 中以便跨线程共享。
        let shared_instance = Arc::new(unsafe_instance);

        let clone_for_thread = Arc::clone(&shared_instance);

        // 创建一个新线程。
        let new_thread = thread::spawn(move || {
            // 在新线程中访问 shared_instance。
            println!("Data in new thread: {}", clone_for_thread.data);
        });

        // 在主线程中改变数据。
        unsafe {
            let unsafe_shared_instance = Arc::get_mut_unchecked(&mut shared_instance);
            unsafe_shared_instance.data = 100;
        }

        // 等待新线程结束。
        new_thread.join().unwrap();

        // 打印主线程中的数据。
        println!("Data in main thread: {}", shared_instance.data);
    }
} 
 */


/* 1.3 反过来，也可能有一个由非 `Send` 类型构成的类型，通过在数据操作上施加额外约束而实际上是线程安全的。下面代码在1.73版本中编译报错,说明对send等限制变严格了 */
/* #[test]
fn test_unsafe_closure_mutex() {
    use std::rc::Rc;
    use std::sync::{Arc, Mutex};
    use std::thread;

    // 定义一个包含非 Send 类型（Rc<T>）的结构体
    struct NonSendType {
        data: Rc<i32>,
    }

    // 实现一个新的结构体，它包含 NonSendType 但是使用 Mutex 来保证线程安全
    struct SafeWrapper {
        inner: Mutex<NonSendType>,
    }

    impl SafeWrapper {
        fn new(data: i32) -> SafeWrapper {
            SafeWrapper {
                inner: Mutex::new(NonSendType {
                    data: Rc::new(data),
                }),
            }
        }

        fn do_something(&self) {
            let mut data = self.inner.lock().unwrap();
            // 在这里可以安全地操作 data
        }
    }


    fn main() {
        let safe_instance = Arc::new(SafeWrapper::new(42));

        let threads: Vec<_> = (0..10)
            .map(|_| {
                let safe_instance_clone = Arc::clone(&safe_instance);
                thread::spawn(move || {
                    safe_instance_clone.do_something();
                })
            })
            .collect();

        for thread in threads {
            thread.join().unwrap();
        }
    }
}
*/



/* We’ve been focusing on Send above, but there’s another trait that’s important for thread-safety: Sync.
This trait represents values that are safe to be accessed by multiple threads at once, that is, sharing.
*/
#[test]
fn test_closure_safe_sharing(){
    fn upto<F>(f:F,n:usize)
    where F:Send + 'static + Fn(usize) + Sync
    {
        let func = Arc::new(f);
        for i in 0..n{
            let clone_func = func.clone();
            thread::spawn(move || {
                clone_func(i)
            });
        }
    }

    fn main() {
        let message: &'static str = "hello";
        upto(move |i| println!("thread #{}: {}", i, message),10);
    
        // as above, don't let `main` finish
        thread::sleep(Duration::from_millis(100));
    }
    main()
    
}