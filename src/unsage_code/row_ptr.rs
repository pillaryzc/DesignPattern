/* 

非安全Rust比安全Rust可以多做的事情只有以下几个：
    1解引用裸指针
    2调用非安全函数（包括C语言函数，编译器内联函数，还有直接内存分配等）
    3实现非安全trait
    4访问或修改可变静态变量

*/

// 1解引用裸指针
#[test]
fn test_row_ptr(){
    let mut num = 3;

    let ptr_n1 = &num as *const i32;
    let ptr_n2 = &mut num as *mut i32;

    unsafe{
        println!("{}",*ptr_n1);
        println!("{}",*ptr_n2);
    }
}
 