//全部内容来自:https://huonw.github.io/blog/2015/05/finding-closure-in-rust/

/* 一: what is closure ? the captures to closing over or capturing variables*/
#[test]
fn first_look_of_closure() {
    let option = Some(2);

    let new = option.map(|x| x + 6);
    dbg!(new);
}

/* 如果没有闭包,那么应该如何写map函数类似的功能呢? 下面的代码是不是很繁琐冗余? */
trait Transform<Input> {
    type Output;

    fn transform(self, input: Input) -> Self::Output;
}

fn map<X, Y, T>(option: Option<X>, transform: T) -> Option<Y>
where
    T: Transform<X, Output = Y>,
{
    match option {
        Some(x) => Some(transform.transform(x)),
        None => None,
    }
}

//下面是一点看上去和closure相同语法的代码实现
#[test]
fn same_semantics_closures() {
    // replacement for |val| val + x
    struct Adder {
        x: i32,
    }

    impl Transform<i32> for Adder {
        type Output = i32;

        // ignoring the `fn ... self`, this looks similar to |val| val + x
        fn transform(self, val: i32) -> i32 {
            val + self.x
        }
    }

    // replacement for |val| val * y
    struct Multiplier {
        y: i32,
    }

    impl Transform<i32> for Multiplier {
        type Output = i32;

        // looks similar to |val| val * y
        fn transform(self, val: i32) -> i32 {
            val * self.y
        }
    }

    fn main() {
        let option = Some(2);

        let x = 3;
        let new: Option<i32> = map(option, Adder { x: x });
        println!("{:?}", new); // Some(5)

        let y = 10;
        let new2 = map(option, Multiplier { y: y });
        println!("{:?}", new2); // Some(20)
    }

    main()
}

/*
             Implementer	                    Consumer
self	    Can move out and mutate	Can     only call method once
&mut self	Can’t move out, can mutate	    Can call many times, only with unique access
&self	    Can’t move out or mutate	    Can call many times, with no restrictions
*/

/* 二: How do real closures work?
    plus some flexibility and syntactic sugar ....  just like this:

    impl<X> Option<X> {
    pub fn map<Y, F: FnOnce(X) -> Y>(self, f: F) -> Option<Y> {
        match self {
            Some(x) => Some(f(x)),
            None => None
        }
    }
}

&self is Fn
&mut self is FnMut
self is FnOnce

闭包的编译器实现：当在Rust中写下 |args...| code... 形式的闭包时，
编译器会隐式地定义一个独特的新结构体类型来存储捕获的变量，
并实现上述某个特质，使用闭包的体作为实现。编译器还会重写提及捕获变量的任何部分，
使其通过闭包的环境来访问。这个结构体类型对用户是不可见的，仅在编译器内部使用。

运行时的闭包处理：程序在运行时遇到闭包定义时，会创建这个结构体的实例，
并将该实例传递给需要它的任何地方（就像前面提到的 map 函数例子中所做的那样）。
*/

/*
There’s two questions left:

1.how are variables captured? (what type are the fields of the environment struct?)
    struct T { ... }

    fn by_value(_: T) {}
    fn by_mut(_: &mut T) {}
    fn by_ref(_: &T) {}

    let x: T = ...;
    let mut y: T = ...;
    let mut z: T = ...;

    let closure = || {
        by_ref(&x);
        by_ref(&y);
        by_ref(&z);

        // forces `y` and `z` to be at least captured by `&mut` reference
        by_mut(&mut y);
        by_mut(&mut z);

        // forces `z` to be captured by value
        by_value(z);
    };

2.which trait is used? (what type of self is used?)
p

*/

/*
三: move and escape
下面的代码会报错
*/
/* /// Returns a closure that will add `x` to its argument.
    fn make_adder(x: i32) -> Box<Fn(i32) -> i32> {
        Box::new(|y| x + y)
    }

    fn main() {
        let f = make_adder(3);

        println!("{}", f(1)); // 4
        println!("{}", f(10)); // 13
    }

去糖后的代码如下:
    struct Closure<'a> {
        x: &'a i32   //这里也就是报错关键的原因,捕获的x,作用域会过期,不安全
    }

    /* impl of Fn for Closure */

    fn make_adder(x: i32) -> Box<Fn(i32) -> i32> {
        Box::new(Closure { x: &x })
    }

*/

/*
闭包的逃逸（Escaping）：在 make_adder 函数中，闭包通过引用捕获了变量 x。但由于闭包被装箱（boxed）并返回，
它有可能在 make_adder 函数的作用域结束后继续存在。这就造成了闭包中的 x 引用可能指向一个已经不再有效的栈内存位置。

悬挂引用（Dangling Reference）：由于 x 是 make_adder 函数的局部变量，当这个函数返回后，
x 的生命周期结束。然而，闭包试图保持对 x 的引用，这将导致悬挂引用。当闭包在 main 函数中被调用时，
它尝试访问一个不再存在的变量，违反了Rust的借用规则。

编译器报错：Rust编译器会检测到这种潜在的悬挂引用，并阻止这种不安全的行为。
因此，代码在尝试编译时会报错，指出闭包可能比 x 的生命周期活得更久。

为了解决这个问题，可以使用 move 关键字，使闭包通过值而不是通过引用捕获 x。修改后的 make_adder 函数如下：
*/
fn make_adder(x: i32) -> Box<dyn Fn(i32) -> i32> {
    Box::new(move |y| x + y)
}

fn main() {
    let f = make_adder(3);

    println!("{}", f(1)); // 4
    println!("{}", f(10)); // 13
}

/*
原因就是使用move关键字,则捕获的变量的形式如下:
    struct Environment {
        x: T,    //拿到的是所有权
        y: T,
        z: T
    }

*/

/* 下面代码展示了如何利用move关键字解决逃逸问题 */
#[test]
fn move_closure_keyword2() {
    #[derive(Debug)]
    // 定义一个简单的结构体T
    struct T(i32);

    // 一个函数，接受T的不可变引用
    fn by_ref(t: &T) {
        println!("by_ref: {}", t.0);
    }

    // 一个函数，接受T的可变引用
    fn by_mut(t: &mut T) {
        t.0 += 1;
        println!("by_mut: {}", t.0);
    }

    // 一个函数，通过值接受T
    fn by_value(t: T) {
        println!("by_value: {}", t.0);
    }

    fn make_closure() -> Box<dyn FnOnce()> {
        let x = T(10);
        let mut y = T(20);
        let mut z = T(30);

        // 定义一个闭包，使用move关键字捕获变量
        let return_closure = Box::new(move || {
            by_ref(&x); // 使用不可变引用
            by_ref(&y); // 使用不可变引用
            by_ref(&z); // 使用不可变引用

            by_mut(&mut y); // 使用可变引用
            by_mut(&mut z); // 使用可变引用

            by_value(z); // 通过值使用z
        });
        // println!("{:?}",y);  y,z,x have been moved
        return_closure
    }

    make_closure()(); //调用闭包
}

/* 下面代码编译器会隐式实现三个特性FnOnce ,Fn , FnMut */
fn test() {
    #[derive(Debug)]
    // 定义一个简单的结构体T
    struct T(i32);

    // 一个函数，接受T的不可变引用
    fn by_ref(t: &T) {
        println!("by_ref: {}", t.0);
    }

    // 一个函数，接受T的可变引用
    fn by_mut(t: &mut T) {
        t.0 += 1;
        println!("by_mut: {}", t.0);
    }

    // 一个函数，通过值接受T
    fn by_value(t: T) {
        println!("by_value: {}", t.0);
    }

    fn make_adder(x: i32) -> Box<dyn Fn(i32) -> i32> {
        Box::new(move |y| x + y)
    }

    fn test_closure() {
        let x = T(10);
        let mut y = T(20);
        let mut z = T(30);

        // 创建引用
        let x_ref: &T = &x;
        let y_mut: &mut T = &mut y;

        let closure = move || {
            by_ref(x_ref); // 使用不可变引用
            by_ref(&*y_mut); // 解引用后再获取不可变引用
            by_ref(&z); // 直接获取z的不可变引用

            by_mut(y_mut); // 使用可变引用
            by_mut(&mut z); // 获取z的可变引用

            by_value(z); // 通过值使用z
        };
        // 调用闭包
        closure();
    }
}

/* 感受一下闭包的语法，和去糖后的语法 */
/* fn simple_explicit_closure() {
    let mut v = vec![];

    // nice form
    let closure = || v.push(1);

    // explicit form
    struct Environment<'v> {
        v: &'v mut Vec<i32>,
    }

    // let's try implementing `Fn` not pass but FnMut or FnOnce is ok    tips: fllowing code need nightly and #![feature(fn_traits)] 和 #![feature(unboxed_closures)]。
    impl<'v> FnMut() for Environment<'v> {
        fn call_mut(&mut self) {
            self.v.push(1) // error: cannot borrow data mutably
        }
    }

    let closure = Environment { v: &mut v };
}
 */

/*
 四：闭包捕获到的环境在什么情况下分配在栈上，什么情况下分配在堆？ （内容不一定正确）

1.以下是一些可能导致闭包使用堆分配的情况：

1.1长生命周期的捕获：
如果闭包需要在其定义的作用域之外使用，例如作为函数返回值或存储在全局变量中，那么闭包及其捕获的环境需要在堆上分配。
这是因为栈上的变量在其作用域结束时会被销毁，而堆上的数据可以跨作用域存活。

1.2捕获大型数据：
如果闭包捕获了大型数据（如大数组或大型结构体），并且这些数据的大小超出了栈的容量限制，那么这些数据可能会在堆上分配。
栈空间通常有限，不适合存储大量数据。

1.3动态环境捕获：
当闭包动态决定捕获哪些变量时（例如，基于运行时条件），这些变量可能在堆上分配，特别是当它们的总大小在编译时无法确定时。

1.4递归闭包：
在某些情况下，例如递归闭包，闭包可能需要在堆上分配，以避免栈溢出。递归闭包在每次递归调用时都可能捕获新的环境。

1.5跨线程共享：
如果闭包需要在多个线程之间共享，例如作为多线程环境中的任务，它们通常需要在堆上分配。这是因为不同线程的栈空间是隔离的，而堆空间是线程之间共享的。

1.6闭包捕获 Box 或其他智能指针：
如果闭包捕获了 Box、Rc、Arc 等智能指针类型的变量，那么这些变量本身已经在堆上分配。

总之，闭包是否在堆上分配取决于其捕获的数据类型、数据大小、生命周期以及如何使用这些闭包。Rust 的闭包设计旨在最大化性能和灵活性，使得在多数常见情况下闭包可以有效地存储在栈上。


2.分配在栈上的情况通常具有以下特点，这些特点使得栈成为存储数据的理想选择：

有限的生命周期：
栈上的数据通常具有有限的生命周期，即它们只在声明它们的特定作用域（如函数）内存在。一旦作用域结束，栈上的数据就会被自动清理掉。

大小在编译时已知：
栈上的变量通常需要在编译时就确定其大小。这意味着像数组这样的数据结构，如果要存储在栈上，其大小必须在编译时已知。

小型数据：
由于栈空间相对有限，通常只适用于存储小型数据。对于大型数据结构，更倾向于在堆上分配。

非递归和无需跨作用域共享：
栈上的数据不适合用于递归场景，也不适合在作用域之间共享，因为这可能导致栈溢出或生命周期问题。

值类型数据：
栈通常用于存储基本数据类型和小型结构体，这些类型在分配和销毁时开销较小。

快速分配和释放：
栈内存的分配和释放非常快速。它不涉及复杂的内存管理算法，仅仅是移动栈指针。

无需动态扩展：
栈上的数据不适用于动态扩展的场景，因为栈的大小在程序启动时就已经确定。

线程局部存储：
每个线程都有自己的栈，因此栈上的数据是线程局部的。这意味着在多线程环境中，栈上的数据自然地避免了共享状态的复杂性。

在 Rust 中，这些特点使得栈成为许多变量（特别是闭包捕获的变量）的理想存储位置，因为它们满足了快速、简单和安全的内存管理的需求。

  */



  /* n! 闭包递归版本实现 */
fn dfs_closure_test() {
    use std::cell::RefCell;
    use std::rc::Rc;

    fn main() {
        let factorial: Rc<RefCell<Box<dyn Fn(i32) -> i32>>> =
            Rc::new(RefCell::new(Box::new(|_| 0)));
        let factorial_clone = factorial.clone();

        *factorial.borrow_mut() = Box::new(move |n| {
            if n <= 1 {
                1
            } else {
                n * factorial_clone.borrow()(n - 1)
            }
        });

        let result = factorial.borrow()(5);
        println!("Factorial of 5 is {}", result);
    }
}
