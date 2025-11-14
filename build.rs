use std::{future::ready, path::PathBuf};

use rspack::builder::{Builder, ExperimentsBuilder};
use rspack_core::{
    Compiler, ModuleOptions, ModuleRule, ModuleRuleEffect, ModuleType, OutputOptions,
    RuleSetCondition,
};
use rspack_tasks::within_compiler_context_for_testing;

fn main() {
    println!("cargo:rerun-if-changed=frontend/src");

    let dist_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("frontend")
        .join("dist");

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to create tokio runtime for Rspack");

    rt.block_on(async { run_rspack_bundle().await });

    println!("cargo:rustc-env=FRONTEND_DIST_DIR={}", dist_dir.display());
}

async fn run_rspack_bundle() {
    let context = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("frontend");

    within_compiler_context_for_testing(async move {
        let svg_rule = ModuleRule {
            test: Some(RuleSetCondition::Func(Box::new(|ctx| {
                Box::pin(ready(Ok(
                    ctx.as_str()
                        .map(|data| data.ends_with(".svg"))
                        .unwrap_or_default(),
                )))
            }))),
            effect: ModuleRuleEffect {
                r#type: Some(ModuleType::AssetResource),
                ..Default::default()
            },
            ..Default::default()
        };

        let mut module_builder = ModuleOptions::builder();
        module_builder.rule(svg_rule);

        let mut compiler = Compiler::builder()
            .context(context)
            .entry("main", "./src/index.js")
            .experiments(ExperimentsBuilder::default().css(true))
            .output(
                OutputOptions::builder()
                    .filename("[name].js".into())
                    .css_filename("[name].css".into())
                    .asset_module_filename("[name][ext]".into()),
            )
            .module(module_builder)
            .build()
            .expect("failed to build Rspack compiler");

        compiler
            .run()
            .await
            .expect("failed to run Rspack compiler");

        let errors: Vec<_> = compiler.compilation.get_errors().collect();
        if !errors.is_empty() {
            panic!("Rspack compilation failed: {errors:#?}");
        }

    })
    .await;
}
