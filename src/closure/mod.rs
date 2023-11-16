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


/* 下面代码编译器回隐式实现三个特性FnOnce ,Fn , FnMut */
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

    fn test_closure(){
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
