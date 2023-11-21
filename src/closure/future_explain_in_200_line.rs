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
