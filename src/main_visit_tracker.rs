#![feature(box_patterns)]

mod lyy_fold_test;
mod lyy_visit_tracker;

use std::{
    path::Path,
    sync::Arc,
};


use swc_core::{
    base::{
        config::ParseOptions,
        Compiler,
    },
    common::{sync::Lazy, FilePathMapping, SourceMap, source_map::SourceMapGenConfig},
    ecma::{
        ast::{ EsVersion, Module },
        visit::{FoldWith, VisitWith, VisitMutWith}
    }
};

use swc_common::{input::SourceFileInput, FileName};
use swc_ecma_parser::{
    lexer::Lexer,
    Parser
};

use swc_ecma_codegen::{self, text_writer::WriteJs, Emitter};

use crate::lyy_visit_tracker::VisitMutTrackerFn;

static COMPILER: Lazy<Arc<Compiler>> = Lazy::new(|| {
    let cm = Arc::new(SourceMap::new(FilePathMapping::empty()));

    Arc::new(Compiler::new(cm))
});

fn get_compiler() -> Arc<Compiler> {
    COMPILER.clone()
}

struct SwcSourceMapConfig;
impl SourceMapGenConfig for SwcSourceMapConfig {
    fn file_name_to_source(&self, f: &swc_common::FileName) -> String {
        f.to_string()
    }

    /// 生成 sourceContents
    fn inline_sources_content(&self, _f: &FileName) -> bool {
        true
    }
}


fn parse() -> (Arc<Compiler>, Module){
    // 获取 Compiler 实例，但是其实用到的也就 c.cm, cm 可以直接用 SourceMap::new(FilePathMapping::empty()) 代替
    let c = get_compiler();
    // 声明路径
    let path: String = "./tracker.js".to_string();
    // 根据 load 文件
    let fm = 
        c.cm.load_file(Path::new(path.as_str()))
            .expect("failed to read program file");
    // 使用默认的 ParseOptions
    let opt = ParseOptions {
        // syntax: Syntax::Typescript(TsConfig {
        //     tsx: true,
        //     ..Default::default()
        // }),
        comments: true,
        ..Default::default()
    };
    //  根据 ParseOptions 创建 Lexer
    let lexer = Lexer::new(opt.syntax, opt.target, SourceFileInput::from(&*fm), None);
    // 根据 Lexer 创建 Parser
    let mut parser = Parser::new_from(lexer);
    // 解析模块
    let mut result = parser.parse_module().unwrap();
    
    result.visit_mut_with(&mut VisitMutTrackerFn);

    // println!("result: {:#?}", result);

    (c, result)

}

fn get_code (c: Arc<Compiler>, result: Module) -> String {
    // 声明 buf
    let mut buf = vec![];
    let mut map_buf = vec![];
    {
        // 创建wr
        let w = swc_ecma_codegen::text_writer::JsWriter::new(
            c.cm.clone(),
            "\n",
            &mut buf,
            Some(&mut map_buf),
        );
        let wr = Box::new(w) as Box<dyn WriteJs>;
        // 根据 wr 创建 Emitter
        let mut emitter = Emitter {
            cfg: swc_ecma_codegen::Config {
                minify: false,
                target: EsVersion::Es3,
                ..Default::default()
            },
            comments: None,
            cm: c.cm.clone(),
            wr,
        };

        emitter.emit_module(&result).expect("failed to emit module");
    }

    let code = String::from_utf8(buf.clone()).unwrap();

    code
}


fn main () {
    let (c, result) = parse();
    let code = get_code(c, result);
    println!("code:\n{}", code);
}