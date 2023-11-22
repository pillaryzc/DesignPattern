/*
绿色线程(Green threads)
绿色线程使用与操作系统相同的机制，为每个任务创建一个线程，设置一个堆栈，保存 CPU 状态，并通过“上下文切换”从一个任务(线程)跳转到另一个任务(线程)。

我们将控制权交给调度程序(在这样的系统中，调度程序是运行时的核心部分) ，然后调度程序继续运行不同的任务。

Rust曾经支持绿色线程，但他们它达到1.0之前被删除了， 执行状态存储在每个栈中，因此在这样的解决方案中不需要async,await,Futures 或者Pin。

典型的流程是这样的: 1. 运行一些非阻塞代码 2. 对某些外部资源进行阻塞调用 3. 跳转到main”线程，该线程调度一个不同的线程来运行，并“跳转”到该栈中 4. 在新线程上运行一些非阻塞代码，直到新的阻塞调用或任务完成 5. “跳转”回到“main”线程 ，调度一个新线程，这个新线程的状态已经是Ready,然后跳转到该线程

这些“跳转”被称为上下文切换，当你阅读这篇文章的时候，你的操作系统每秒钟都会做很多次。

优点:

栈大小可能需要增长,解决这个问题不容易,并且会有成本.[^go中的栈] [^go中的栈]: 栈拷贝,指针等问题
它不是一个零成本抽象(这也是Rust早期有绿色线程,后来删除的原因之一)
如果您想要支持许多不同的平台，就很难正确实现

*/
/*
 #![feature(asm, naked_functions)]
use std::ptr;

const DEFAULT_STACK_SIZE: usize = 1024 * 1024 * 2;
const MAX_THREADS: usize = 4;
static mut RUNTIME: usize = 0;

pub struct Runtime {
    threads: Vec<Thread>,
    current: usize,
}

  #[derive(PartialEq, Eq, Debug)]
enum State {
    Available,
    Running,
    Ready,
}

struct Thread {
    id: usize,
    stack: Vec<u8>,
    ctx: ThreadContext,
    state: State,
    task: Option<Box<dyn Fn()>>,
}

 #[derive(Debug, Default)]
 #[repr(C)]
struct ThreadContext {
    rsp: u64,
    r15: u64,
    r14: u64,
    r13: u64,
    r12: u64,
    rbx: u64,
    rbp: u64,
    thread_ptr: u64,
}

impl Thread {
    fn new(id: usize) -> Self {
        Thread {
            id,
            stack: vec![0_u8; DEFAULT_STACK_SIZE],
            ctx: ThreadContext::default(),
            state: State::Available,
            task: None,
        }
    }
}

impl Runtime {
    pub fn new() -> Self {
        let base_thread = Thread {
            id: 0,
            stack: vec![0_u8; DEFAULT_STACK_SIZE],
            ctx: ThreadContext::default(),
            state: State::Running,
            task: None,
        };

        let mut threads = vec![base_thread];
        threads[0].ctx.thread_ptr = &threads[0] as *const Thread as u64;
        let mut available_threads: Vec<Thread> = (1..MAX_THREADS).map(|i| Thread::new(i)).collect();
        threads.append(&mut available_threads);

        Runtime {
            threads,
            current: 0,
        }
    }

    pub fn init(&self) {
        unsafe {
            let r_ptr: *const Runtime = self;
            RUNTIME = r_ptr as usize;
        }
    }

    pub fn run(&mut self) -> ! {
        while self.t_yield() {}
        std::process::exit(0);
    }

    fn t_return(&mut self) {
        if self.current != 0 {
            self.threads[self.current].state = State::Available;
            self.t_yield();
        }
    }

    fn t_yield(&mut self) -> bool {
        let mut pos = self.current;
        while self.threads[pos].state != State::Ready {
            pos += 1;
            if pos == self.threads.len() {
                pos = 0;
            }
            if pos == self.current {
                return false;
            }
        }

        if self.threads[self.current].state != State::Available {
            self.threads[self.current].state = State::Ready;
        }

        self.threads[pos].state = State::Running;
        let old_pos = self.current;
        self.current = pos;

        unsafe {
            switch(&mut self.threads[old_pos].ctx, &self.threads[pos].ctx);
        }
        true
    }

    pub fn spawn<F: Fn() + 'static>(f: F){
        unsafe {
            let rt_ptr = RUNTIME as *mut Runtime;
            let available = (*rt_ptr)
                .threads
                .iter_mut()
                .find(|t| t.state == State::Available)
                .expect("no available thread.");

            let size = available.stack.len();
            let s_ptr = available.stack.as_mut_ptr();
            available.task = Some(Box::new(f));
            available.ctx.thread_ptr = available as *const Thread as u64;
            ptr::write(s_ptr.offset((size - 8) as isize) as *mut u64, guard as u64);
            ptr::write(s_ptr.offset((size - 16) as isize) as *mut u64, call as u64);
            available.ctx.rsp = s_ptr.offset((size - 16) as isize) as u64;
            available.state = State::Ready;
        }
    }
}

fn call(thread: u64) {
    let thread = unsafe { &*(thread as *const Thread) };
    if let Some(f) = &thread.task {
        f();
    }
}

 #[naked]
fn guard() {
    unsafe {
        let rt_ptr = RUNTIME as *mut Runtime;
        let rt = &mut *rt_ptr;
        println!("THREAD {} FINISHED.", rt.threads[rt.current].id);
        rt.t_return();
    };
}

pub fn yield_thread() {
    unsafe {
        let rt_ptr = RUNTIME as *mut Runtime;
        (*rt_ptr).t_yield();
    };
}

 #[naked]
 #[inline(never)]
unsafe fn switch(old: *mut ThreadContext, new: *const ThreadContext) {
    asm!("
        mov     %rsp, 0x00($0)
        mov     %r15, 0x08($0)
        mov     %r14, 0x10($0)
        mov     %r13, 0x18($0)
        mov     %r12, 0x20($0)
        mov     %rbx, 0x28($0)
        mov     %rbp, 0x30($0)

        mov     0x00($1), %rsp
        mov     0x08($1), %r15
        mov     0x10($1), %r14
        mov     0x18($1), %r13
        mov     0x20($1), %r12
        mov     0x28($1), %rbx
        mov     0x30($1), %rbp
        mov     0x38($1), %rdi
        ret
        "
    :
    : "r"(old), "r"(new)
    :
    : "alignstack"
    );
}
 #[cfg(not(windows))]
fn main() {
    let mut runtime = Runtime::new();
    runtime.init();
    Runtime::spawn(|| {
        println!("I haven't implemented a timer in this example.");
        yield_thread();
        println!("Finally, notice how the tasks are executed concurrently.");
    });
    Runtime::spawn(|| {
        println!("But we can still nest tasks...");
        Runtime::spawn(|| {
            println!("...like this!");
        })
    });
    runtime.run();
}
 #[cfg(windows)]
fn main() {
    let mut runtime = Runtime::new();
    runtime.init();
    Runtime::spawn(|| {
        println!("I haven't implemented a timer in this example.");
        yield_thread();
        println!("Finally, notice how the tasks are executed concurrently.");
    });
    Runtime::spawn(|| {
        println!("But we can still nest tasks...");
        Runtime::spawn(|| {
            println!("...like this!");
        })
    });
    runtime.run();
}



*/

/*
基于回调的方法背后的整个思想就是保存一个指向一组指令的指针，这些指令我们希望以后在以后需要的时候运行。 针对Rust，这将是一个闭包。 在下面的示例中，我们将此信息保存在HashMap中，但这并不是唯一的选项。

不涉及线程作为实现并发性的主要方法的基本思想是其余方法的共同点。 包括我们很快就会讲到的 Rust 今天使用的那个。

优点:
1. 大多数语言中易于实现
2. 没有上下文切换
3. 相对较低的内存开销(在大多数情况下)

缺点:

1.每个任务都必须保存它以后需要的状态，内存使用量将随着一系列计算中回调的数量线性增长
2.很难理解，很多人已经知道这就是“回调地狱”
3.这是一种非常不同的编写程序的方式，需要大量的重写才能从“正常”的程序流转变为使用“基于回调”的程序流
4.在 Rust 使用这种方法时，任务之间的状态共享是一个难题，因为它的所有权模型
一个极其简单的基于回调方法的例子是:
*/

use std::{marker::PhantomPinned, pin::Pin};

use crate::main;

#[test]
fn test_callback_programming() {
    fn program_main() {
        println!("So we start the program here!");
        set_timeout(200, || {
            println!("We create tasks with a callback that runs once the task finished!");
        });
        set_timeout(100, || {
            println!("We can even chain sub-tasks...");
            set_timeout(50, || {
                println!("...like this!");
            })
        });
        println!("While our tasks are executing we can do other stuff instead of waiting.");
    }

    fn main() {
        RT.with(|rt| rt.run(program_main));
    }

    use std::sync::mpsc::{channel, Receiver, Sender};
    use std::{cell::RefCell, collections::HashMap, thread};

    thread_local! {
        static RT: Runtime = Runtime::new();
    }

    struct Runtime {
        callbacks: RefCell<HashMap<usize, Box<dyn FnOnce() -> ()>>>,
        next_id: RefCell<usize>,
        evt_sender: Sender<usize>,
        evt_reciever: Receiver<usize>,
    }

    fn set_timeout(ms: u64, cb: impl FnOnce() + 'static) {
        RT.with(|rt| {
            let id = *rt.next_id.borrow();
            *rt.next_id.borrow_mut() += 1;
            rt.callbacks.borrow_mut().insert(id, Box::new(cb));
            let evt_sender = rt.evt_sender.clone();
            thread::spawn(move || {
                thread::sleep(std::time::Duration::from_millis(ms));
                evt_sender.send(id).unwrap();
            });
        });
    }

    impl Runtime {
        fn new() -> Self {
            let (evt_sender, evt_reciever) = channel();
            Runtime {
                callbacks: RefCell::new(HashMap::new()),
                next_id: RefCell::new(1),
                evt_sender,
                evt_reciever,
            }
        }

        fn run(&self, program: fn()) {
            program();
            for evt_id in &self.evt_reciever {
                let cb = self.callbacks.borrow_mut().remove(&evt_id).unwrap();
                cb();
                if self.callbacks.borrow().is_empty() {
                    break;
                }
            }
        }
    }
}

/* Waker 唤醒器
1.了解 Waker 对象是如何构造的
2.了解运行时如何知道leaf-future何时可以恢复
3.了解动态分发的基础知识和trait对象
*/

/* 1.了解Waker,先看胖指针
&dyn Trait:-----16   前8个字节指向trait 对象的data - 后八个字节指向trait对象的 vtable
&[&dyn Trait]:--16
Box<Trait>:-----16
&i32:-----------8
&[i32]:---------16   前8个字节是指向数组中第一个元素的实际指针(或 slice 引用的数组的一部分) - 第二个8字节是切片的长度
Box<i32>:-------8
&Box<i32>:------8
[&dyn Trait;4]:-64
[i32;4]:--------16
*/
#[test]
fn test_fat_prt() {
    use std::mem::size_of;

    trait SomeTrait {}

    fn main() {
        println!("======== The size of different pointers in Rust: ========");
        println!("&dyn Trait:-----{}", size_of::<&dyn SomeTrait>());
        println!("&[&dyn Trait]:--{}", size_of::<&[&dyn SomeTrait]>());
        println!("Box<Trait>:-----{}", size_of::<Box<dyn SomeTrait>>());
        println!("&i32:-----------{}", size_of::<&i32>());
        println!("&[i32]:---------{}", size_of::<&[i32]>());
        println!("Box<i32>:-------{}", size_of::<Box<i32>>());
        println!("&Box<i32>:------{}", size_of::<&Box<i32>>());
        println!("[&dyn Trait;4]:-{}", size_of::<[&dyn SomeTrait; 4]>());
        println!("[i32;4]:--------{}", size_of::<[i32; 4]>());
    }

    main()
}

/* 虚表的数据结构 */
#[test]
fn costom_fat_ptr() {
    // A reference to a trait object is a fat pointer: (data_ptr, vtable_ptr)
    trait Test {
        fn add(&self) -> i32;
        fn sub(&self) -> i32;
        fn mul(&self) -> i32;
    }

    // This will represent our home brewn fat pointer to a trait object
    #[repr(C)]
    struct FatPointer<'a> {
        /// A reference is a pointer to an instantiated `Data` instance
        data: &'a mut Data,
        /// Since we need to pass in literal values like length and alignment it's
        /// easiest for us to convert pointers to usize-integers instead of the other way around.
        vtable: *const usize,
    }

    // This is the data in our trait object. It's just two numbers we want to operate on.
    struct Data {
        a: i32,
        b: i32,
    }

    // ====== function definitions ======
    fn add(s: &Data) -> i32 {
        s.a + s.b
    }
    fn sub(s: &Data) -> i32 {
        s.a - s.b
    }
    fn mul(s: &Data) -> i32 {
        s.a * s.b
    }

    fn main() {
        let mut data = Data { a: 3, b: 2 };
        // vtable is like special purpose array of pointer-length types with a fixed
        // format where the three first values has a special meaning like the
        // length of the array is encoded in the array itself as the second value.
        let vtable = vec![
            0, // pointer to `Drop` (which we're not implementing here)
            6, // lenght of vtable
            8, // alignment
            // we need to make sure we add these in the same order as defined in the Trait.
            add as usize, // function pointer - try changing the order of `add`
            sub as usize, // function pointer - and `sub` to see what happens
            mul as usize, // function pointer
        ];

        let fat_pointer = FatPointer {
            data: &mut data,
            vtable: vtable.as_ptr(),
        };
        let test = unsafe { std::mem::transmute::<FatPointer, &dyn Test>(fat_pointer) };

        // And voalá, it's now a trait object we can call methods on
        println!("Add: 3 + 2 = {}", test.add());
        println!("Sub: 3 - 2 = {}", test.sub());
        println!("Mul: 3 * 2 = {}", test.mul());
    }

    main();
}

/* 生成器
1.理解 async / await 语法在底层是如何工作的
2/亲眼目睹(See first hand)我们为什么需要Pin
3.理解是什么让 Rusts 异步模型的内存效率非常高
*/
#[test]
fn yied_code_with_generator() {
    /*
    let mut gen = move || {
        let to_borrow = String::from("Hello");
        let borrowed = &to_borrow;
        yield borrowed.len();
        println!("{} world!", borrowed);
    };
    上述代码解糖后了类似下面代码
     */

    #![feature(never_type)] // Force nightly compiler to be used in playground
                            // by betting on it's true that this type is named after it's stabilization date...
    pub fn main() {
        let mut gen = GeneratorA::start();
        let mut gen2 = GeneratorA::start();

        if let GeneratorState::Yielded(n) = gen.resume() {
            println!("Got value {}", n);
        }

        std::mem::swap(&mut gen, &mut gen2); // <--- Big problem!

        if let GeneratorState::Yielded(n) = gen2.resume() {
            println!("Got value {}", n);
        }

        // This would now start gen2 since we swapped them.
        if let GeneratorState::Complete(()) = gen.resume() {
            ()
        };
    }
    enum GeneratorState<Y, R> {
        Yielded(Y),
        Complete(R),
    }

    trait Generator {
        type Yield;
        type Return;
        fn resume(&mut self) -> GeneratorState<Self::Yield, Self::Return>;
    }

    enum GeneratorA {
        Enter,
        Yield1 {
            to_borrow: String,
            borrowed: *const String,
        },
        Exit,
    }

    impl GeneratorA {
        fn start() -> Self {
            GeneratorA::Enter
        }
    }
    impl Generator for GeneratorA {
        type Yield = usize;
        type Return = ();
        fn resume(&mut self) -> GeneratorState<Self::Yield, Self::Return> {
            match self {
                GeneratorA::Enter => {
                    let to_borrow = String::from("Hello");
                    let borrowed = &to_borrow;
                    let res = borrowed.len();
                    *self = GeneratorA::Yield1 {
                        to_borrow,
                        borrowed: std::ptr::null(),
                    };

                    // We set the self-reference here
                    if let GeneratorA::Yield1 {
                        to_borrow,
                        borrowed,
                    } = self
                    {
                        *borrowed = to_borrow;
                    }

                    GeneratorState::Yielded(res)
                }

                GeneratorA::Yield1 { borrowed, .. } => {
                    let borrowed: &String = unsafe { &**borrowed };
                    println!("{} world", borrowed);
                    *self = GeneratorA::Exit;
                    GeneratorState::Complete(())
                }
                GeneratorA::Exit => panic!("Can't advance an exited generator!"),
            }
        }
    }

    main()
}

/* Pinning和自引用结构 */

/* Pin解决的问题如下 */
fn problem_code() {
    use std::pin::Pin;

    #[derive(Debug)]
    struct Test {
        a: String,
        b: *const String,
    }

    impl Test {
        fn new(txt: &str) -> Self {
            let a = String::from(txt);
            Test {
                a,
                b: std::ptr::null(),
            }
        }

        fn init(&mut self) {
            let self_ref: *const String = &self.a;
            self.b = self_ref;
        }

        fn a(&self) -> &str {
            &self.a
        }

        fn b(&self) -> &String {
            unsafe { &*(self.b) }
        }
    }

    fn ok_main() {
        let mut test1 = Test::new("test1");
        test1.init();
        let mut test2 = Test::new("test2");
        test2.init();

        println!("a: {}, b: {}", test1.a(), test1.b());
        println!("a: {}, b: {}", test2.a(), test2.b());
    }

    fn not_ok_main() {
        let mut test1 = Test::new("test1");
        test1.init();
        let mut test2 = Test::new("test2");
        test2.init();

        println!("a: {}, b: {}", test1.a(), test1.b());
        std::mem::swap(&mut test1, &mut test2);
        println!("a: {}, b: {}", test2.a(), test2.b());
    }
}

#[test]
fn test_pin() {
    use std::marker::PhantomPinned;
    use std::mem;
    use std::pin::Pin;

    #[derive(Debug)]
    struct Test {
        a: String,
        b: *const String,
        _marker: PhantomPinned,
    }

    impl Test {
        fn new(txt: &str) -> Self {
            Test {
                a: String::from(txt),
                b: std::ptr::null(),
                _marker: PhantomPinned, // This makes our type `!Unpin`
            }
        }
        fn init<'a>(self: Pin<&'a mut Self>) {
            let self_ptr: *const String = &self.a;
            let this = unsafe { self.get_unchecked_mut() };
            this.b = self_ptr;
        }

        fn a<'a>(self: Pin<&'a Self>) -> &'a str {
            &self.get_ref().a
        }

        fn b<'a>(self: Pin<&'a Self>) -> &'a String {
            unsafe { &*(self.b) }
        }
    }
    fn main() {
        let mut test1 = Test::new("test1");
        let mut test1_pin = unsafe { Pin::new_unchecked(&mut test1) };
        Test::init(test1_pin.as_mut());
        drop(test1_pin);

        let mut test2 = Test::new("test2");
        mem::swap(&mut test1, &mut test2);
        println!("Not self referential anymore: {:?}", test1.b);
    }
    // main()

    //下面代码会编译不通过，因为被pin在栈上的数据不应许被swap
    //变量阴影（Shadowing）和错误的使用
    // pub fn main_1() {
    //     let mut test1 = Test::new("test1");
    //     let mut test1 = unsafe { Pin::new_unchecked(&mut test1) };
    //     Test::init(test1.as_mut());

    //     let mut test2 = Test::new("test2");
    //     let mut test2 = unsafe { Pin::new_unchecked(&mut test2) };
    //     Test::init(test2.as_mut());

    //     println!("a: {}, b: {}", Test::a(test1.as_ref()), Test::b(test1.as_ref()));
    //     std::mem::swap(test1.get_mut(), test2.get_mut());
    //     println!("a: {}, b: {}", Test::a(test2.as_ref()), Test::b(test2.as_ref()));
    // }
    // main_1();
}

/* 变量阴影（Shadowing）的解决办法：
1.避免在栈上创建并返回自引用结构：
如果你需要创建自引用结构，考虑在堆上创建。例如，使用 Box 来分配堆内存，然后通过 Pin 来固定它。这样可以在函数返回后保持对象的有效性。

2.正确管理固定的对象：
当使用 Pin 固定对象时，需要谨慎地管理这个对象。确保不再对原始对象进行操作，尤其是在它被固定之后。

3.使用类型系统防止错误：
考虑使用 Rust 的类型系统来强制执行正确的用法。例如，可以设计类型使得只有固定（Pinned）的版本可以被初始化。
*/

//1.Pinning to the heap + 始化Pin类型
fn pinning_to_heap() {
    use std::marker::PhantomPinned;
    use std::pin::Pin;

    #[derive(Debug)]
    struct Test {
        a: String,
        b: *const String,
        _marker: PhantomPinned,
    }

    impl Test {
        fn new(txt: &str) -> Pin<Box<Self>> {
            let t = Test {
                a: String::from(txt),
                b: std::ptr::null(),
                _marker: PhantomPinned,
            };
            let mut boxed = Box::pin(t);
            let self_ptr: *const String = &boxed.as_ref().a;
            unsafe { boxed.as_mut().get_unchecked_mut().b = self_ptr };

            boxed
        }

        fn a<'a>(self: Pin<&'a Self>) -> &'a str {
            &self.get_ref().a
        }

        fn b<'a>(self: Pin<&'a Self>) -> &'a String {
            unsafe { &*(self.b) }
        }
    }

    pub fn main() {
        let mut test1 = Test::new("test1");
        let mut test2 = Test::new("test2");

        println!("a: {}, b: {}", test1.as_ref().a(), test1.as_ref().b());
        println!("a: {}, b: {}", test2.as_ref().a(), test2.as_ref().b());
    }
}

// todo!()
#[test]
fn test_pin_in_heap() {
    use std::pin::Pin;

    struct Test {
        to_borrow: String,
        borrow: *const String,
        _marker: PhantomPinned,
    }
    impl Test {
        fn new(s: &str) -> Pin<Box<Self>> {
            let mut t = Test {
                to_borrow: String::from(s),
                borrow: std::ptr::null(),
                _marker: PhantomPinned,
            };
            let mut boxed = Box::pin(t);
            let self_ptr: *const String = &boxed.as_ref().to_borrow;
            unsafe {
                boxed.as_mut().get_unchecked_mut().borrow = self_ptr;
            }
            boxed
        }

        fn a<'a>(self: Pin<&'a Self>) -> &'a str {
            &self.get_ref().to_borrow
        }

        fn b<'a>(self: Pin<&'a Self>) -> &'a String {
            unsafe { &*(self.borrow) }
        }
    }

    fn main() {
        let mut test1 = Test::new("test1");
        let mut test2 = Test::new("test2");

        println!("a: {}, b: {}", test1.as_ref().a(), test1.as_ref().b());
        std::mem::swap(&mut test1, &mut test2);
        println!("a: {}, b: {}", test2.as_ref().a(), test2.as_ref().b());
    }

    main()
}

/* 学习了pin,现在可以来解决之前生成器的自引用问题了 */
fn pin_fix_generator() {
    #![feature(auto_traits, negative_impls)] // needed to implement `!Unpin`
    use std::pin::Pin;

    pub fn main() {
        let gen1 = GeneratorA::start();
        let gen2 = GeneratorA::start();
        // Before we pin the data, this is safe to do
        // std::mem::swap(&mut gen, &mut gen2);

        // constructing a `Pin::new()` on a type which does not implement `Unpin` is
        // unsafe. An object pinned to heap can be constructed while staying in safe
        // Rust so we can use that to avoid unsafe. You can also use crates like
        // `pin_utils` to pin to the stack safely, just remember that they use
        // unsafe under the hood so it's like using an already-reviewed unsafe
        // implementation.

        let mut pinned1 = Box::pin(gen1);
        let mut pinned2 = Box::pin(gen2);

        // Uncomment these if you think it's safe to pin the values to the stack instead
        // (it is in this case). Remember to comment out the two previous lines first.
        //let mut pinned1 = unsafe { Pin::new_unchecked(&mut gen1) };
        //let mut pinned2 = unsafe { Pin::new_unchecked(&mut gen2) };

        if let GeneratorState::Yielded(n) = pinned1.as_mut().resume() {
            println!("Gen1 got value {}", n);
        }

        if let GeneratorState::Yielded(n) = pinned2.as_mut().resume() {
            println!("Gen2 got value {}", n);
        };

        // This won't work:
        // std::mem::swap(&mut gen, &mut gen2);
        // This will work but will just swap the pointers so nothing bad happens here:
        // std::mem::swap(&mut pinned1, &mut pinned2);

        let _ = pinned1.as_mut().resume();
        let _ = pinned2.as_mut().resume();
    }

    enum GeneratorState<Y, R> {
        Yielded(Y),
        Complete(R),
    }

    trait Generator {
        type Yield;
        type Return;
        fn resume(self: Pin<&mut Self>) -> GeneratorState<Self::Yield, Self::Return>;
    }

    enum GeneratorA {
        Enter,
        Yield1 {
            to_borrow: String,
            borrowed: *const String,
        },
        Exit,
    }

    impl GeneratorA {
        fn start() -> Self {
            GeneratorA::Enter
        }
    }

    // This tells us that this object is not safe to move after pinning.
    // In this case, only we as implementors "feel" this, however, if someone is
    // relying on our Pinned data this will prevent them from moving it. You need
    // to enable the feature flag `#![feature(optin_builtin_traits)]` and use the
    // nightly compiler to implement `!Unpin`. Normally, you would use
    // `std::marker::PhantomPinned` to indicate that the struct is `!Unpin`.
    // impl !Unpin for GeneratorA {}

    impl Generator for GeneratorA {
        type Yield = usize;
        type Return = ();
        fn resume(self: Pin<&mut Self>) -> GeneratorState<Self::Yield, Self::Return> {
            // lets us get ownership over current state
            let this = unsafe { self.get_unchecked_mut() };
            match this {
                GeneratorA::Enter => {
                    let to_borrow = String::from("Hello");
                    let borrowed = &to_borrow;
                    let res = borrowed.len();
                    *this = GeneratorA::Yield1 {
                        to_borrow,
                        borrowed: std::ptr::null(),
                    };

                    // Trick to actually get a self reference. We can't reference
                    // the `String` earlier since these references will point to the
                    // location in this stack frame which will not be valid anymore
                    // when this function returns.
                    if let GeneratorA::Yield1 {
                        to_borrow,
                        borrowed,
                    } = this
                    {
                        *borrowed = to_borrow;
                    }

                    GeneratorState::Yielded(res)
                }

                GeneratorA::Yield1 { borrowed, .. } => {
                    let borrowed: &String = unsafe { &**borrowed };
                    println!("{} world", borrowed);
                    *this = GeneratorA::Exit;
                    GeneratorState::Complete(())
                }
                GeneratorA::Exit => panic!("Can't advance an exited generator!"),
            }
        }
    }
}
