<p align="center">
	<a href="https://github.com/jameender/arcanya">hi</a>
</p>

<h1 align="center">Arcanya</h1>

A little _magical_ language 🧙‍♂️

## What is Arcanya?

Arcanya is a interpreted LISP like language built in **Rust**.

It is a hobby project of mine that I created to learn more about the LISP family. I was frustrated with the syntax, so I created my own.

It supports:

-   Integers
-   Floats
-   Strings
-   Symbols
-   Functions
-   Builtins
-   Mapping
-   Folding (or reducing)
-   Filtering
-   Partial function application 😍
-   and more..

To try it out, just run

```bash
cargo run
```

Which starts a interactive Arcanya session.

## Install

The only way for now to install is to:

```bash
git clone https://github.com/jameender/arcanya

cd arcanya

cargo run
```

## Example code

You can print any variable using `print`

```lisp
(print "hello world")
```

Defining global variables can be done with `define`

```lisp
(define x 5)
(define y 7)

(+ x y)
; 12
```

Defining local variables can be done with `let`

```lisp
(let x 5
	(let y 7
		(+ x y)
	)
)
; 12
```

Or by using let multiple with `let*`

```lisp
(let* (
	(x 5)
	(y 7)
) (+ x y))
; 12
```

You can map over lists with `map`

```lisp
(map
	(function (x) (* x 2))
	(1 2 3)
)
; (2 4 6)
```

And fold over lists with `fold`

```lisp
(fold + 0 (1 2 3))
; 6
```

Concat strings with `concat`

```lisp
(concat
	"Arcanya "
	"is "
	"gorgeous" "!"
)
; you can figure that out ;)
```
