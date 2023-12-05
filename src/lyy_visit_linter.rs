use swc_core::{
    ecma::{
        visit::Visit,
        ast::{
            ForStmt, Stmt, Expr, BinExpr, BinaryOp, UpdateExpr, UpdateOp
        }
    },
};

pub struct VisitLint;

impl Visit for VisitLint {
    fn visit_for_stmt(&mut self, node: &ForStmt) {
        let mut test_op: Option<&BinaryOp> = None;
        let mut update_op: Option<&UpdateOp> = None;

        if let Some(box Expr::Bin(BinExpr{
            op,
            ..
        })) = &node.test {
            test_op = Some(op);
        }

        if let Some(box Expr::Update(UpdateExpr{
            op,
            ..
        })) = &node.update {
            update_op = Some(op);
        }

        if test_op.is_some() && update_op.is_some() {
            let t_op = *test_op.unwrap();
            let u_op = *update_op.unwrap();
            if t_op == BinaryOp::Lt || t_op == BinaryOp::LtEq {
                if u_op == UpdateOp::MinusMinus {
                    println!("error")
                }
            }

            if t_op == BinaryOp::Gt || t_op == BinaryOp::GtEq {
                if u_op == UpdateOp::PlusPlus {
                    println!("error")
                }
            }
        }
    }
}