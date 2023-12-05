use swc_core::{
  ecma::{
    ast::{Expr, ExprOrSpread, Lit, Number, ArrayLit},
    visit::{Fold, FoldWith, VisitMut, VisitMutWith, VisitWith, Visit},
  },
  common::DUMMY_SP
};

pub struct FoldTest;

impl Fold for FoldTest {
    fn fold_array_lit(&mut self, node: ArrayLit) -> ArrayLit {
        ArrayLit {
            span: DUMMY_SP,
            elems: node.elems.into_iter().map(|ele| {
                match ele {
                    Some(ExprOrSpread {
                        spread: None,
                        expr: box Expr::Lit(Lit::Num(num)),
                    }) => {
                        let v = num.value + 1.0;
                        Some(ExprOrSpread {
                            spread: None,
                            expr: Box::new(Expr::Lit(Lit::Num(Number {
                                span: DUMMY_SP,
                                value: v,
                                raw: None
                            }))),
                        })
                    },
                    Some(expr) => Some(expr.fold_children_with(self)),
                    _  => ele
                }
            }).collect()
            
        }
    }
}

pub struct VisitTest;

impl Visit for VisitTest {
    fn visit_array_lit(&mut self, node: &ArrayLit) {
        println!("visit node: {:?} \n", node);
        node.visit_children_with(self);
    }
}

pub struct VisitMutTest;
impl VisitMut for VisitMutTest {
    fn visit_mut_array_lit(&mut self, node: &mut ArrayLit) {
        node.elems = node.elems.iter_mut().map(|ele| {
            match ele {
                Some(ExprOrSpread {
                    spread: None,
                    expr: box Expr::Lit(Lit::Num(num)),
                }) => {
                    let v = num.value + 1.0;
                    Some(ExprOrSpread {
                        spread: None,
                        expr: Box::new(Expr::Lit(Lit::Num(Number {
                            span: DUMMY_SP,
                            value: v,
                            raw: None
                        }))),
                    })
                },
                _  => ele.take()
            }
        }).collect();
        
        node.visit_mut_children_with(self);
    }
}