# FML language and runtime

FML is an imperative, object-oriented toy language based on Feeny. FML has
ML-like syntax. The features are selected to make the language small and quick
to implement, but to make the runtime implementation non-trivial.

Features:
  * object-oriented
  * inheritence
  * dynamically typed
  * overrideable operators
  * dynamic dispatch
  * pass by reference

The name FML is accidental, but turned out to be archetypal nevertheless.

# Installation guide

FML is a rust project hosted on Github.

## Prerequisites

Getting Rust (https://www.rust-lang.org/tools/install):

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Getting the code

```
git clone https://github.com/kondziu/FML.git
```

## Compilation

```
cd FML;
cargo build --release
```

Running:

```
cargo run --release
```

or

```
target/release/fml
```

If you're going to use the interpreter, I recommend using it through rlwrap.

```
rlwrap cargo run --release
```

or 

```
rlwrap target/release/fml
```

## Usage

Run an FML program:

```
fml run examples/hello_world.fml
```

## Full stack

The execution always parses the FML source code, compiles it into bytecode, and then executes the bytecode. These steps can be done separately to get intermediate results.

Parse the FML source into an AST in the form of a LISP S-expression:

```
fml parse examples/hello_world.fml -o examples/hello_world.ast --format=lisp       
```

Other formats include JSON and YAML.

Compile LISP S-expression into bytecode:

```
fml compile examples/hello_world.ast -o examples/hello_world.bc --input-format=lisp --output-format=bytes
```

Execute bytecode:

```
target/release/fml execute examples/hello_world.bc
```

# Language elements

A run down of all language elements in some sort of order.

## Comments

FML has C-style end-of-line comments and block comments.

End of line comments start with `//` end end with an end of line:

```fml
// Here, I will write all my secrets!
```

Block comments start with `/*` and end with `*/` and can span multiple lines. Comments do not nest.

```fml
/* in Naniwa Bay, 
   now the flowers are blossoming. 
   after lying dormant all winter, 
   now the spring has come 
   and the flowers are blossoming. */
```

Comments support UTF-8.

```fml
// 👍 
```

## Types

FML has five types: integer, boolean, unit, objects, and arrays.

**Integers** represent numbers. Integers are always signed 32 bit values.

```fml
-1;
42
```

*Note:* The semicolon `;` is used to separate two expressions, it's not a part
of the literal.

Integers have the following operators defined on them: 

```fml
5  + 7;   //    12
5  - 7;   //    -2
5  * 7;   //    35
5  / 7;   //     0
5  % 7;   //     5
5 <= 7;   //  true
5 >= 7;   // false
5  < 7;   //  true
5  > 7;   // false
5 == 7;   // false
5 != 7;   //  true
```

Comparison of an integer to a value of a different type with `==` and `!=` is
also possible and it results in `false` and `true` respectively.

*Note:* FML does not have floating point numbers, just integers.

**Boolean** type represents logical expressions. Booleans can have the following two values:

```fml
true;
false
```

Booleans support the following operators:

```fml
true  & false;   // false
true  | false;   //  true
true == false;   // false
true != false;   //  true
```

Comparison of a boolean to a value of a different type with `==` and `!=` is
also possible and it results in `false` and `true` respectively.

*Note:* Feeny did not have booleans.

**Unit** represents things that don't have a value or where we don't care
about the value. It is represented with the literal:

```fml
null
```

Unit supports comparisons via `==` and `!=` to itself and other types.

*Note:* We will refer this as null and unit interchangeably, which is a
misnomer.  The term unit is used for a separate type that means "nothing is
expected to be here, ever", whereas null is a value that can be assigned to any
type and means "there is no value here right now".  As FML is a dynamic
language, this distinction is blurred in practice and unit often plays both
roles.

**Objects** are programmer-defined structures containing data and associated
methods and operators that can be executed on them. They are described in a
separate section below.

**Arrays** are contain collections of multiple values of any type. They are
described in a separate section below.

## Print

FML has a print instruction, for, well, printing things on screen.

```fml
print("ahoj przygodo!\n")
```

*Note:* FML does not have a string type, so that string literal is just a
feature of `print`.

The printer has a limited formatter where each placeholder `~` is replaced by
consecutive arguments to `print`:

```fml
print("~ and ~ and ~ and ~\n", 1, 2, 3, 4)  // prints: 1 and 2 and 3 and 4
```

The result of `print` is unit. 

## Variables

FML uses the `let` keyword and `=` for declaration. A variable is given a name
and an initial value. 

```fml
let answer = 42;
```

A variable cannot be created without an initial value, which is where
nulls/units come in handy.

```fml
let butts = null;
```

A variable can be assigned the result of any expression.

```fml
let result = some_function();
```

A variable declaration returns (a reference to) the initial value;

```fml
let a = let b = 2;
print("~ ~\n", a, b)    // 2 2
```

By the way, the language has parens that can be used to disambiguate expressions:

```fml
let a = (let b = 2); 
print("~ ~\n", a, b)    // 2 2
```

Since variables are variable, their value can be changed with the asssignment
operator `<-`.

```fml
let answer = null;
answer <- 42;
```



## Blocks of expressions

In FML a block of expression is defined between a `begin` and an `end` with
semicolons separating statements inside:

```fml
begin
  let x = 1;
  let y = 2;
  print("~ ~\n", x, y)
end
```

A block can be used anywhere instead of a single statement. It returns the
result of the last executed statement inside.

```fml
let x = begin 1; 2; 3 end
print("~\n", x)                 // 3
```

Blocks inherit the environment of their parent, but everything defined inside
is limited to the scope of the environment:

```fml
let x = 1;
let y = 1;
begin
  let y = 1;
  let z = 1;
  x <- 2;
  y <- 2;
  z <- 2;
end;
print("~\n", x);        // 2
print("~\n", y);        // 1
print("~\n", z);        // no such global: z
```

Environments can be nested.

## Conditional expressions

FML has two principle mechanisms of controlling the flow of execution:
conditionals and loops.

A conditional expressions us the keywords `if`, `then`, `else` representing the
condition, the consequent branch, and the alternative branch. `if` is followed
by an expression that should evaluate to boolean. `then` and `else` are
followed by expressions that execute depending on the result of the condition.
It's all pretty standard, really.

```fml
if true 
then print("true")
else print("false") 
```

If you need more than one expression in any of the branches, use a block:

```fml
if true
then begin
  print("true");
  print("true");
end
else begin
  print("false");
  print("false");
end
```

The conditional statement returns the result of the last executed expression:

```fml
let p = true;
let q = false;
let x = if p & q then 1 else 0;
print("p & q = ~\n", x);         // p & q = 0
```

The `else` block is optional:

```fml
if true
  then print("true\n")
```

If the `else` block is missing, and the alternative branch is invoked, the
conditional returns unit:

```fml
print("~\n", if true  then true)   // true
print("~\n", if false then true)   // null
```

The condition, and both brancheds is integrated into the parent scope:

```fml
if let x = true then let y = true else let z = true;
print("~\n", x);                    // true
print("~\n", y);                    // true
print("~\n", z);                    // no such global: z
```

**Note to self**: I could fix this, but then y'all would have to fix this too.

## Loops

FML has while loops and nothign else. It has a condition that is re-evaluated
on each iteration and an expression for a body.

```fml
while false
do print("forever\n")
```

A loop printing numbers from 0 to 10.

```fml
let i = 0;
while i < 10 
do begin
  print("~\n", i);
  i <- i + 1;
end
```

Loops can be used as expressions but always return unit.

## Functions

Another control flow abstraction we have in FML is functions, which execute a
snippet of code and return to the callsite with a result. Functions are named
and have a list of zero or more parameters. A function returns the result of
the statement sconsittuting its body.

```fml
function f(x,y) -> 2 * x + y;
```

Functions are called (applied) with a list of arguments that are assigned to
parameters positionally:

```fml
let x = f(2, 1);     
print("~\n", x)      // 2 * 2 + 1 = 5
```

If a function's body requires more than one expression, use a block:

function ff(x, y) -> 
begin
  let a = 2 * x;
  let b = y;
  a + b
end;

FML functions pass arguments in by reference (or rather, the reference is
copied)---it's explained later where it matters.

*Note:* There's no `return` statement in FML. It simplifies some things.

Functions can only be defined anywhere in the top level.

## Arrays

Arrays are structures that hold indexable references to multiple other
entities. Arrays in FML have constant sizes, but have mutable elements. They
have a single dimension. The simplest way to construct an array is by
specifying its size and an initial value for all the elements:

```fml
array(7, null)        // [null, null, null, null, null, null, null, ]
```

The size can be specified using any expression:

```fml
array(let size = 7, null);
print("size: ~\n", size);         // size: 7
```

The initial value can also be specified using an expression, however, this
expression will be re-executed for every element. This means that a sort of
array comprehension is possible:

```fml
let i = 1;
let a = array(10, begin let x = i; i <- i + 1; x; end);
print("~\n", a)                   // [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, ]
```

*Note:* You can thank/curse your colleagues from the beta edition of this
course for introducing this feature.

Array elements can be accessed by a get operator `[..]`. Arrays are 0-indexed.

```fml
print("~..~\n", a[0], a[9]);      // 1..10
```

Array elements can be modified with a `[..] <-` operator.

```fml
a[5] <- -1;
print("~\n", a)                   // [1, 2, 3, 4, 5, -1, 7, 8, 9, 10, ]
```

Remember only references to arrays get passed around:

```fml
let a = array(3, null);
let b = a;
b[1] <- 42;
print("~\n", a);                // [null, -42, null, ] 
print("~\n", b);                // [null, -42, null, ]
```

And this:

```fml
function f (arr) -> 
begin 
  arr[1] <- 42
end    

let a = array(3, null);
f(a);
print("~\n", a);                // [null, -42, null, ] 
```

## Objects 

Objects are declared by defining an anonymous instance with optional
inheritence declarations and a possibly-empty list of members.

```fml
object /* optional inheritence declaration goes here */
begin 
  /* optional members go here */
end;
```

This declaration is an expression that returns a reference to the object
instance. This can be assigned to variables:

```fml
let obj = object begin end;
```

*Hint*: it can also be returned from functions as if they are a ur-constructors:

```fml
function new() -> object begin end;
```

Let's look at members first. Members can be fields, methods, or operators.

### Fields

Fields are defined just like variables:

```fml
let point = object 
begin
  let x = 0;
  let y = 1;
  let z = 2;
end;
```

Fields are public. They can be accessed by the dot operator:

```fml
print("x=~, y=~, z=~\n", point.x, point.y, point.z); // x=0, y=1, z=2
```

(A field cannnot be called `print`. This is a bug, but it's too annoying to fix.)

### Methods

Methods are defined just like functions:

```fml
let point = object 
begin
  let x = 0;
  let y = 1;
  let z = 2;
  function print() -> 
  begin
    print("x=~, y=~, z=~\n", this.x, this.y, this.z);
  end
end;
```

Methods can get fields and call methods on the `this` object which is a
reference to the host object of the method.

```fml
point.print()
```

### Operators

It is possible to deifne functions that overload operators. 

```fml
function new(x) ->
  object 
  begin
    let inner = 2;
    function + (operand) -> this.inner + operand.inner
  end;
```

Most operators work as infix binary operators:

```fml
let x = new(1);
let y = new(2);
let r = x + y;
print("~\n", r);
```

They can also be called like ordinary methods, if that's how you like to
entartain yourself:

```fml
let x = new(1);
let y = new(2);
let r = x.+(y);
print("~\n", r);
```

Here's a list of all supported binary operators in FML:

```fml
object 
begin
  function  | (operand) -> begin end;
  function  & (operand) -> begin end;
  function == (operand) -> begin end;
  function != (operand) -> begin end;
  function  > (operand) -> begin end;
  function  < (operand) -> begin end;
  function >= (operand) -> begin end;
  function <= (operand) -> begin end;
  function  + (operand) -> begin end;
  function  - (operand) -> begin end;
  function  / (operand) -> begin end;
  function  * (operand) -> begin end;
  function  % (operand) -> begin end;
end
```

### Operator precedence

FML has straighforward operator precedense:

| priority | group       | operators     |
| :------- | :---------- | :------------ |
| 1        | factor      | `*`, `/`, `%` |
| 2        | additive    | `+`, `-`      |
| 3        | comparison  | `==`, `!=`    |
| 4        | conjunction | `&`           |
| 5        | disjunction | `|`           |

So the following are equivalent:

```fml
false |     2*2  +  2/2   ==  8 - 3   & true;
false | ((((2*2) + (2/2)) == (8 - 3)) & true);
```

### Array operators

In addition to those, FML also has two array operators:

```fml
let arr = array(1, null);
arr[0] <- true;             // assignment operator aka set
arr[0];                     // access operator aka get
```

We can impement those for any object by implementing methods called `get` and `set`:

```fml
object 
begin
  function get(index)        -> begin end;
  function set(index, value) -> begin end;
end    
```

They can also be called as methods:

```fml
let arr = array(1, null);
arr.set(0, true);
arr.get(0);
```

### Inheritence

The FML language supports inheritence. An object's parent is specified after
the `object` keyword with the `extends` keyword. 

```fml
let pseudo_one = object extends 1 begin end;
let pseudo_two = object extends 2 begin end;
print("~\n", pseudo_one + 2);
print("~\n", pseudo_two + 1);
```

But not `pseudo_one + pseudo_two`, because `pseudo_one` delagates to `1`, and `1
+ pseudo_two` will fail.


Method dispatch looks at methods in the current object. If none are found, it
looks in the parent object, and continues looking until there is no parent
object to look up in.

```fml
function immutable_array(len, value) ->
    object extends array(len, value)
    begin
      function set(index, value) -> 
        print("Cannot set value: immutable array\n");
    end;

let arr = immutable_array(10, 42);
arr[0] <- 6;       // Cannot set value: immutable array
print("~\n", arr); // object(..=[42, 42, 42, 42, 42, 42, 42, 42, 42, 42])
```

We can also compose functionality from current and parent object:

```fml
function math_array(len, value) -> 
    object extends array(len, value)
    begin
      let length = len;
      function + (value) -> 
      begin
        let i = 0;
        let result = array(this.length, null);
        while i < this.length do
        begin
          result[i] <- this[i] + value;
          i <- i + 1;
        end;
        result
      end
    end;

let arr1 = math_array(10, 5);
let arr2 = arr1 + 1;
arr2[0] <- 7;
print("~\n", arr2); // [7, 6, 6, 6, 6, 6, 6, 6, 6, 6]
```

If the `extends` phrase is omitted, the parent is `null`.

*Note*: there's no `super` keyword.