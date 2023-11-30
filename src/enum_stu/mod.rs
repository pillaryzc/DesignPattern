//一：下面内容来自于：https://smallcultfollowing.com/babysteps/blog/2015/05/05/where-rusts-enum-shines/


//1.作者先表述rust中enum的灵活

/* cpp
enum ErrorCode {
    FileNotFound,
    UnexpectedChar
};

class Error {
  public:
    Error(ErrorCode ec) : errorCode(ec) { }
    const ErrorCode errorCode;
};

class FileNotFoundError : public Error {    
  public:
    FileNotFound() : Error(FileNotFound);
};

class UnexpectedChar : public ErrorCode {
  public:
    UnexpectedChar(char expected, char found)
      : Error(UnexpectedChar),
        expected(expected),
        found(found)
    { }
    
    const char expected;
    const char found;
};

*/

/* 
enum ErrorCode {
    FileNotFound,
    UnexpectedChar
}

fn parse_file(file_name: String) -> ErrorCode;

==========================================================

enum ErrorCode {
    FileNotFound,
    UnexpectedChar { expected: Vec<String>, found: char }
}

fn parse_file(file_name: String) -> ErrorCode;

*/


//二：Virtual Structs Part 2: Classes strike back : https://smallcultfollowing.com/babysteps/blog/2015/05/29/classes-strike-back/
