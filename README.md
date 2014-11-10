# Coki

Coki is a simple language written in Rust.  Right now Coki is
interpreted, but fairly soon it will be compiled via LLVM.

This is mostly just a toy language for me to:

* write stuff in Rust!
* demonstrate my [peruse](https://github.com/DanSimon/peruse.git) parser-combinator library
* try writing a LLVM front-end

## "Features"

* the only first-class type is signed integer
* all variables are global (scope coming soon!)
* the only reserved keywords right now are `if`, `else`, `while`, and `out`


Here's a program to output the first 30 Fibbonacci numbers.
```
n1 = 1
n2 = 1
n = n1 + n2
i = 0

while i < 30 {
  out n
  n2 = n1
  n1 = n
  n = n1 + n2
  i = i + 1
}

```

The name comes from Coki Beach in St. Thomas, US VI, which is where I was when
I decided this project needed a name.
