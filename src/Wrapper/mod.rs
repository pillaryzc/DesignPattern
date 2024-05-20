use std::io::{BufReader, Cursor};



#[test]
fn test_wrapper(){
    BufReader::new(Cursor::new("Input data")); //BufReader就使用了装饰器设计模式，不动inner的基础上，增加buf缓冲功能
}