use swc_core::{
  ecma::{
    ast::{
      Expr, Function, ExprStmt, 
      Stmt, CallExpr, Ident, Callee, ArrowExpr, BlockStmtOrExpr, ReturnStmt, BlockStmt,
      Module, ModuleItem, ModuleDecl, ImportDecl, Str, ImportSpecifier, ImportDefaultSpecifier
    },
    visit::VisitMut
  },
  common::DUMMY_SP,
};

// 生成 tracker 函数
fn create_tracker () -> Stmt{ 
  Stmt::Expr(ExprStmt { 
    span: DUMMY_SP,
    expr: Box::new(Expr::Call(CallExpr { 
      span: DUMMY_SP,
      callee: Callee::Expr(
        Box::new(Expr::Ident(
          Ident::new("tracker".into(), DUMMY_SP)
        ))
      ),
      args: vec![],
      type_args: None
    }))
  })
}

// 生成 tracker_import
fn create_tracker_import () -> ModuleItem {
  ModuleItem::ModuleDecl(
    ModuleDecl::Import(ImportDecl { 
      span: DUMMY_SP,
      specifiers: vec![ImportSpecifier::Default(ImportDefaultSpecifier {
        span: DUMMY_SP,
        local: Ident::new("tracker".into(), DUMMY_SP)
      })],
      src: Box::new(Str::from("tracker")),
      type_only: false,
      asserts: None
    })
  )
}

pub struct VisitMutTrackerFn;

impl VisitMut for VisitMutTrackerFn {
  fn visit_mut_module(&mut self, node: &mut Module) {
    let mut has_import = false;
    node.body =  node.body.iter_mut().map(|item| {
      if let ModuleItem::ModuleDecl(ModuleDecl::Import(import)) = item {
        // import ** from "tracker";
        if import.src.value.to_string() == "tracker".to_string() {
          // 是否 import 了 tracker
          has_import = true;
          // 是否使用 moduledefault 的形式引入 
          let has_default = import.specifiers.iter().any(|item| match item {
            ImportSpecifier::Default(_) => true,
            _ => false
          });
          if !has_default {
            let tracker_default = ImportSpecifier::Default(ImportDefaultSpecifier {
              span: DUMMY_SP,
              local: Ident::new("tracker".into(), DUMMY_SP)
            });
            import.specifiers.insert(0, tracker_default)
          }
        }
        item.clone()
      } else {
        item.clone()
      }
    }).collect();
    if !has_import {
      let tracker_import = create_tracker_import();
      let body = &mut node.body;
      body.insert(0, tracker_import);
    }
  }

  fn visit_mut_function(&mut self, node: &mut Function) {
    let body = node.body.as_mut();
    if let Some(blk_stmt) = body {
      let stmts = &mut blk_stmt.stmts;
      let tracker = create_tracker();

      stmts.insert(0, tracker);
    }
  }

  fn visit_mut_arrow_expr(&mut self, node: &mut ArrowExpr) {
    let body = node.body.as_mut();
    match body {
       // 有 {}, 直接加上 tracker
        BlockStmtOrExpr::BlockStmt(blk_stmt) => {
          let tracker = create_tracker();
          blk_stmt.stmts.insert(0, tracker);
        },
        // 没有 {}, 需要在外层加上代码块, 加上 tracker, 再加上 return
        BlockStmtOrExpr::Expr(expr) => {
          let tracker = create_tracker();
          let mut stmts = vec![];
          let return_stmt = Stmt::Return(ReturnStmt {
            span: DUMMY_SP,
            arg: Some(expr.clone())
          });
          stmts.push(tracker);
          stmts.push(return_stmt);
          let body = BlockStmtOrExpr::BlockStmt(BlockStmt {
            span:DUMMY_SP,
            stmts
          });
          node.body = Box::new(body);
        }
    };
  }
}