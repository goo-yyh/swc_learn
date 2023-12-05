use std::{collections::HashMap, vec};

use swc_core::{
  ecma::{
    ast::{ 
      KeyValueProp, Expr, Lit, Str, PropName, Ident, 
      JSXAttr, JSXAttrValue, CallExpr, Tpl, TplElement,
      ExprOrSpread, Callee, JSXExprContainer, JSXExpr
    },
    visit::{VisitMut, VisitMutWith},
    atoms::Atom
  },
  common::{DUMMY_SP, comments::Comments},
};

pub struct VisitLocalMut {
  pub intl_map: HashMap<String, String>
}

impl VisitLocalMut {
    pub fn new() -> Self {
      VisitLocalMut {
        intl_map: HashMap::new()
      }
    }
}

impl VisitMut for VisitLocalMut {
    // 解析 local.js
    fn visit_mut_key_value_prop(&mut self, node: &mut KeyValueProp) {
      if let KeyValueProp {
        key: PropName::Ident(Ident {
          sym,
          ..
        }),
        value: box Expr::Lit(Lit::Str(Str {
          value,
          ..
        }))
      } = node {
        self.intl_map.insert(value.to_string(), sym.to_string());
      }
    }
}

pub struct VisitIntlMut<'a> {
  pub map: HashMap<String, String>,
  pub comments: &'a dyn Comments
}

// 创建 intl("intl1") 的 callee 函数
fn create_intl_expr(v: String) -> Expr {
  Expr::Call(CallExpr {
    span: DUMMY_SP,
      callee: Callee::Expr(Box::new(Expr::Ident(Ident::new("intl".into(), DUMMY_SP)))),
      args: vec![ExprOrSpread {
        spread: None,
        expr: Box::new(Expr::Lit(Lit::Str(Str::from(v))))
      }],
      type_args: None
  })
}

// 创建 jsx 的 attr
fn create_intl_jsx(v: String) -> JSXExpr {
  JSXExpr::Expr(Box::new(create_intl_expr(v)))
}

// 创建空的 quasis
fn create_empty_tpl_element() -> TplElement {
  TplElement {
      span: DUMMY_SP,
      tail: Default::default(),
      cooked: Some(Atom::new("")),
      raw: Default::default(),
  }
}

impl VisitMut for VisitIntlMut<'_> {
    // jsx attr 中包含的字符串
    fn visit_mut_jsx_attr(&mut self, node: &mut JSXAttr) {
      if let JSXAttr {
        value: Some(JSXAttrValue::Lit(Lit::Str(str))),
        ..
      } = node {
        let v = str.value.to_string();
        let k = self.map.get(&v);
        if let Some(key) = k {
          let expr = create_intl_jsx(key.clone());
          node.value = Some(JSXAttrValue::JSXExprContainer(JSXExprContainer { span: DUMMY_SP, expr }));
        }
      }

      node.visit_mut_children_with(self);
    }

    // 字符串
    fn visit_mut_expr(&mut self, node: &mut Expr) {
      if let Expr::Lit(Lit::Str(Str {
        value: v,
        span,
        ..
      })) = node {
        let comments = self.comments.get_leading(span.lo()); 
        
        match comments { 
          Some(comments) => println!("comments: {:?}", comments), 
          None => (), 
        }; 

        let k = self.map.get(&v.to_string());
        if let Some(key) = k {
          let expr = create_intl_expr(key.clone());
          *node = expr;
        }
      }

      node.visit_mut_children_with(self);
    }

    // 模板字符串
    fn visit_mut_tpl(&mut self, node: &mut Tpl) {
      let mut exprs = node.exprs.clone();
      let mut quasis = node.quasis.clone();
      let mut index_arr: Vec<(usize, String)> = vec![];

      // 找到哪些 quasis 的下标需要被国际化
      for (i, ele) in quasis.iter().enumerate() {
        if let Some(v) = &ele.cooked {
          let k = self.map.get(&v.to_string());
          if let Some(key) = k {
            index_arr.push((i, key.clone()));
          }
        }
      }
      
      // 进行国际化替换, 每一个 quasis 被替换, 都需要生成 2 个空的 quasis
      let mut insert_time = 0;
      index_arr.iter().for_each(|(i, k)| {
        let expr = create_intl_expr(k.clone());
        let ind = i + insert_time;
        exprs.insert(ind, Box::new(expr));
        quasis.insert(ind, create_empty_tpl_element());
        quasis.insert(ind, create_empty_tpl_element());
        quasis.remove(ind + 2);
        insert_time += 1;
      });

      node.exprs = exprs;
      node.quasis = quasis;

      node.visit_mut_children_with(self);
    }
}
