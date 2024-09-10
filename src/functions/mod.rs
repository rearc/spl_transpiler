pub mod convert_fns;
pub mod eval_fns;
mod shared;
pub mod stat_fns;

use crate::pyspark::dealias::Dealias;
use crate::spl::ast;
use anyhow::{bail, Result};

impl TryFrom<ast::Expr> for i64 {
    type Error = anyhow::Error;

    fn try_from(value: ast::Expr) -> std::result::Result<i64, Self::Error> {
        match value {
            ast::Expr::Leaf(ast::LeafExpr::Constant(ast::Constant::Int(ast::IntValue(val)))) => {
                Ok(val)
            }
            _ => bail!("No default conversion from {:?} to i64", value),
        }
    }
}

impl TryFrom<ast::Expr> for f64 {
    type Error = anyhow::Error;
    fn try_from(value: ast::Expr) -> std::result::Result<f64, Self::Error> {
        match value {
            ast::Expr::Leaf(ast::LeafExpr::Constant(ast::Constant::Double(ast::DoubleValue(
                val,
            )))) => Ok(val),
            _ => bail!("No default conversion from {:?} to f64", value),
        }
    }
}

impl TryFrom<ast::Expr> for String {
    type Error = anyhow::Error;

    fn try_from(value: ast::Expr) -> std::result::Result<String, Self::Error> {
        match value {
            ast::Expr::Leaf(ast::LeafExpr::Constant(const_)) => match const_ {
                ast::Constant::Null(_) => {
                    bail!("No default conversion from {:?} to String", const_)
                }
                ast::Constant::Bool(_) => {
                    bail!("No default conversion from {:?} to String", const_)
                }
                ast::Constant::Int(_) => bail!("No default conversion from {:?} to String", const_),
                ast::Constant::Double(_) => {
                    bail!("No default conversion from {:?} to String", const_)
                }
                ast::Constant::Str(ast::StrValue(val)) => Ok(val.clone()),
                ast::Constant::SnapTime(_) => {
                    bail!("No default conversion from {:?} to String", const_)
                }
                ast::Constant::SplSpan(_) => {
                    bail!("No default conversion from {:?} to String", const_)
                }
                ast::Constant::Field(ast::Field(val)) => Ok(val.clone()),
                ast::Constant::Wildcard(ast::Wildcard(val)) => Ok(val.clone()),
                ast::Constant::Variable(ast::Variable(val)) => Ok(val.clone()),
                ast::Constant::IPv4CIDR(ast::IPv4CIDR(val)) => Ok(val.clone()),
            },
            _ => bail!("Unsupported mvindex start argument: {:?}", value),
        }
    }
}

fn map_arg<E, T>(arg: &ast::Expr) -> Result<T>
where
    E: Into<anyhow::Error>,
    T: TryFrom<ast::Expr, Error = E> + Dealias,
{
    arg.clone()
        .try_into()
        .map(|e: T| e.unaliased())
        .map_err(|e: E| e.into())
}

fn map_args<E, T>(args: Vec<ast::Expr>) -> Result<Vec<T>>
where
    E: Into<anyhow::Error>,
    T: TryFrom<ast::Expr, Error = E> + Dealias,
{
    args.iter().map(|arg| map_arg(arg)).collect()
}

macro_rules! _function_args {
    ([$args:ident, $i:expr] ()) => {};
    ([$args:ident, $i:ident] ($name:ident : $type:ty , $($tail:tt)*)) => {
        ensure!($i < $args.len(), "Expected an argument for position {} ({}:{}), but only received {}", stringify!($i), stringify!($name), type_name::<$type>(), $args.len());
        let $name: $type = map_arg(&$args[$i])?;
        $i += 1;
        _function_args!([$args,$i] ($($tail)*));
    };
    ([$args:ident, $i:ident] ($name:ident : $type:ty)) => {
        ensure!($i < $args.len(), "Expected an argument for position {} ({}:{}), but only received {}", stringify!($i), stringify!($name), type_name::<$type>(), $args.len());
        let $name: $type = map_arg(&$args[$i])?;
        $i += 1;
    };
    ([$args:ident, $i:ident] ($name:ident , $($tail:tt)*)) => {
        ensure!($i < $args.len(), "Expected an argument for position {} ({}:Expr), but only received {}", stringify!($i), stringify!($name), $args.len());
        let $name: Expr = map_arg(&$args[$i])?;
        $i += 1;
        _function_args!([$args,$i] ($($tail)*));
    };
    ([$args:ident, $i:ident] ($name:ident)) => {
        ensure!($i < $args.len(), "Expected an argument for position {} ({}:Expr), but only received {}", stringify!($i), stringify!($name), $args.len());
        let $name: Expr = map_arg(&$args[$i])?;
        $i += 1;
    };
}

macro_rules! function_transform {
    ($name:ident [$arg_vec:ident] $args:tt { $out:expr }) => {
        {
            let mut _i: usize = 0;
            _function_args!([$arg_vec, _i] $args);
            ensure!(_i == $arg_vec.len(), "Mistmatched number of arguments (code: {}, runtime: {}); fix arg list or assign remaining arguments using `eval_fn!({} [{} -> mapped_args] ...`", _i, $arg_vec.len(), stringify!($name), stringify!($arg_vec));
            Ok(column_like!([$out.unaliased()].alias(stringify!($name))).into())
        }
    };
    ($name:ident [$arg_vec:ident -> $mapped_arg_name:ident] $args:tt { $out:expr }) => {
        {
            let mut _i: usize = 0;
            _function_args!([$arg_vec, _i] $args);
            let $mapped_arg_name: Vec<Expr> = map_args($arg_vec.iter().skip(_i).cloned().collect())?;
            Ok(column_like!([$out.unaliased()].alias(stringify!($name))).into())
        }
    };
}

pub(crate) use _function_args;
pub(crate) use function_transform;
