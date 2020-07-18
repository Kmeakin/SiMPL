# SiMPL

## The name
 - SiMPL
 - is a
 - Minimal / Mediocre
 - Programming
 - Language
 
## What is it
- SiMPL is a very basic programming language that I am creating to learn about different topics in PL design and implementation. 
- It will **not** be a big boy language that you can write actually usefull programs in. 
- It **will** be abandoned sooner or later.
- Each "stage" of the compiler pipeline is very "narrow" (eg there is no syntactic sugar, there are only Ints, Floats and Bools). This allows each stage to be implemented quickly.
  - Think of it as a depth-first search: go directly from blank repo to generating machine code first, then add extra features later.
  
## Syntax
- Implemented in `crates/simpl-syntax`
Grammar:
```
Program := Expr

Expr := LitExpr
      | VarExpr
      | IfExpr
      | LetExpr
      | LambdaExpr
      | AppExpr

LitExpr := Int | Float | Bool
Int     := [0-9]+
Float   := [0-9]+ "." [0-9]+
Bool    := "true" | "false"

VarExpr := Ident
Ident   := [a-zA-Z][a-zA-Z0-9_]*

IfExpr := "if" Expr "then" Expr "else" Expr ";"

LetExpr    := "let" Bindings "in" Expr ";"
Bindings   := (Ident "=" Expr),+

LambdaExpr := "\" "(" Params ")" "->" Expr ";"
Params     := (Ident),+

AppExpr    := Expr "(" Args ")"
Args       := (Expr),+
```
- Note that `IfExpr`, `LetExpr` and `LambdaExpr` and all terminated by semi-colons. This is temporary and will be fixed once I can be bothered to understand what shift-reduce conflicts are.
- Note also that function application is written with the more "mainstream" `f(1, 2, 3)` style from C, Java etc, not the `f 1 2 3` style from Haskell, Ml etc. This is because it's my language dammit.

## Type system
- Implemented in `crates/simpl-syntax`
- Hindley-Milner type system, add type-classes later if I get around to it.
- Functions are **not** automatically curried. I prefer explicit currying vs spewing type errors because you forgot the last argument in a call lines up.
  - TODO: syntactic sugar for automatic currying: `f(1, ...)` => `\(x, y) -> f(1, x, y)`
- Features
  - [x] Infer principal types for every expression
  - [x] Letrec (recursive and mutually-recursive functions)
    - eg `let fact = \x -> if x == 0 then 1 else x * fact(x - 1) in fact(5)`
  - [ ] Let-polymorphism:
    - eg allow `let id = \x -> x in (id(1), id(false))` to be typed. 
    Currently fails because typing `id(1)` solves the constraint `id: t1 -> t1` into `id: Int -> Int`, which then fails when trying to type `id(false)`. `id` should really be `forall t. t1 -> t1`
    - I think let-polymorphism is also called "rank-1 types"?
  - [ ] Explicit type-annotations
    - eg `let x: Int = 5`
    - A strictly HM (ie no rank-2 or higher types) can infer types of all expressions without any annotations required (though inference with rank-2 or higher is undecidiable). So annotations are purely to aid with readability
  - [ ] Type-classes
  
## Code generation
TODO: implement 
