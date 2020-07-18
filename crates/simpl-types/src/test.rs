use crate::{
    annotate::{annotate, Annotations, Annotator, Constraints},
    arena::{store, ExprArena},
};
use simpl_syntax::parse;

fn print_arena(arena: ExprArena) {
    println!("===ARENA===");
    for (id, expr) in arena.iter().rev() {
        println!("id: {}, expr: {:?}", id.index(), expr);
    }
    println!("===========");
}

fn print_anns(anns: Annotations) {
    println!("===ANNOTATIONS===");
    let mut anns_vec: Vec<_> = anns.iter().collect();
    anns_vec.sort_by_key(|(id, _)| id.clone());

    for (id, tvar) in anns_vec.iter().rev() {
        println!("id: {}, type var: {}", id.index(), tvar);
    }

    println!("=================");
}

fn print_cons(cons: Constraints) {
    println!("===CONSTRAINTS===");
    for con in cons {
        println!("{}", con);
    }
    println!("=================");
}

#[test]
fn annotate_test0() {
    let src = r"
\(is_zero) ->
    if is_zero(1)
        then 2
        else 3
    ;
;";

    let ast = parse(src).unwrap();
    let (arena, id) = store(ast);

    print_arena(arena.clone());

    let (anns, cons) = annotate(arena, id);

    print_anns(anns);
    print_cons(cons);
}

#[test]
fn annotate_test1() {
    let src = r"
let id = \(x, y) -> x; in id(1, false);
";

    let ast = parse(src).unwrap();
    let (arena, id) = store(ast);

    print_arena(arena.clone());

    let (anns, cons) = annotate(arena, id);

    print_anns(anns);
    print_cons(cons);
}
