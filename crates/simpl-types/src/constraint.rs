use crate::{
    ty::Type,
    typed_ast::{Expr, Lit},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Constraint(Type, Type);
pub type Constraints = Vec<Constraint>;

fn lit_type(lit: Lit) -> Type {
    match lit {
        Lit::Int(_) => Type::Int,
        Lit::Bool(_) => Type::Bool,
        Lit::Float(_) => Type::Float,
    }
}

fn collect(expr: Expr) -> Constraints {
    match expr {
        Expr::Lit { ty, val } => vec![Constraint(ty, lit_type(val))],

        Expr::Var { ty, name } => vec![],

        Expr::If {
            ty,
            test,
            then_branch,
            else_branch,
        } => {
            let mut cons = vec![
                Constraint(test.ty(), Type::Bool),
                Constraint(then_branch.ty(), ty.clone()),
                Constraint(else_branch.ty(), ty),
            ];

            cons.extend(collect(*test.clone()));
            cons.extend(collect(*then_branch.clone()));
            cons.extend(collect(*else_branch.clone()));

            cons
        }

        Expr::Lambda { ty, args, body } => {
            let mut cons = vec![Constraint(
                ty.clone(),
                Type::Fn(
                    args.iter().map(|(_, t)| t).cloned().collect(),
                    box body.ty(),
                ),
            )];

            cons.extend(collect(*body));

            cons
        }

        Expr::App { ty, func, args } => {
            let mut cons = vec![Constraint(
                func.ty(),
                Type::Fn(args.iter().map(|arg| arg.ty()).collect(), box ty),
            )];

            for arg in args {
                cons.extend(collect(arg));
            }

            cons.extend(collect(*func));

            cons
        }

        Expr::Let { ty, bindings, body } => {
            let mut cons = vec![Constraint(ty, body.ty())];

            cons.extend(
                bindings
                    .iter()
                    .map(|(_name, t, val)| Constraint(t.clone(), val.ty()))
                    .collect::<Constraints>(),
            );

            for (_, _, val) in bindings {
                cons.extend(collect(val));
            }

            cons.extend(collect(*body));

            cons
        }
    }
}
