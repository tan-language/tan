# Tan Language

Tan is a language for Humans and Machines.

## Example

Here is an example of Tan:

```tan
; Compute the fibonacci function.

(let fib (Func x
    (if (< x 3)
        1
        (else (+ (fib (- x 1)) (fib (- x 2))))
    )
))

(echo (fib 10))
```

Check out more [Tan examples](https://github.com/tan-language/examples) to get a
feeling of Tan.

## Status

This is an _experimental_ project, not intended for production use. However, the
project is under active development.

## Contributing

Pull requests, issues, and comments are welcome! Make sure to add tests for new
features and bug fixes.

## License

This work is licensed under the Apache-2.0 License with LLVM Exceptions. See
[LICENSE.txt](LICENSE.txt) or <https://spdx.org/licenses/LLVM-exception.html>
for details.

## Copyright

Copyright © 2022 [Georgios Moschovitis](https://gmosx.com).
