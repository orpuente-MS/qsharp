// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use criterion::{criterion_group, criterion_main, Criterion};
use qsc::incremental::Compiler;
use qsc_data_structures::language_features::LanguageFeatures;
use qsc_eval::{debug::map_hir_package_to_fir, lower::Lowerer};
use qsc_fir::fir::PackageStore;
use qsc_frontend::compile::{PackageStore as HirPackageStore, RuntimeCapabilityFlags, SourceMap};
use qsc_passes::PackageType;
use qsc_rca::{Analyzer, PackageStoreComputeProperties};

const TELEPORT: &str = include_str!("../../../samples/algorithms/Teleportation.qs");
const DEUTSCHJOZSA: &str = include_str!("../../../samples/algorithms/DeutschJozsa.qs");
const LARGE: &str = include_str!("./large.qs");

pub fn core_and_std(c: &mut Criterion) {
    c.bench_function(
        "Perform Runtime Capabilities Analysis (RCA) on the core and std libraries",
        |b| {
            let mut compilation_context = CompilationContext::new();
            b.iter(|| {
                compilation_context.analyze_all();
            });
        },
    );
}

pub fn teleport(c: &mut Criterion) {
    c.bench_function(
        "Perform Runtime Capabilities Analysis (RCA) on teleport sample",
        |b| {
            // First, compile and analyze the packages included by default (core & std).
            let mut compilation_context = CompilationContext::new();
            compilation_context.analyze_all();

            // Now, update the compilation with the sample, and analyze only the updated package.
            compilation_context.update_compilation(TELEPORT);
            b.iter(|| {
                compilation_context.analyze_open_package();
            });
        },
    );
}

pub fn deutsch_jozsa(c: &mut Criterion) {
    c.bench_function(
        "Perform Runtime Capabilities Analysis (RCA) on Deutsch-Jozsa sample",
        |b| {
            // First, compile and analyze the packages included by default (core & std).
            let mut compilation_context = CompilationContext::new();
            compilation_context.analyze_all();

            // Now, update the compilation with the sample, and analyze only the updated package.
            compilation_context.update_compilation(DEUTSCHJOZSA);
            b.iter(|| {
                compilation_context.analyze_open_package();
            });
        },
    );
}

pub fn large_file(c: &mut Criterion) {
    c.bench_function(
        "Perform Runtime Capabilities Analysis (RCA) on large file sample",
        |b| {
            // First, compile and analyze the packages included by default (core & std).
            let mut compilation_context = CompilationContext::new();
            compilation_context.analyze_all();

            // Now, update the compilation with the sample, and analyze only the updated package.
            compilation_context.update_compilation(LARGE);
            b.iter(|| {
                compilation_context.analyze_open_package();
            });
        },
    );
}

struct CompilationContext {
    compiler: Compiler,
    lowerer: Lowerer,
    fir_store: PackageStore,
    compute_properties: Option<PackageStoreComputeProperties>,
}

impl CompilationContext {
    fn new() -> Self {
        Self::default()
    }

    fn analyze_all(&mut self) {
        let analyzer = Analyzer::init(&self.fir_store);
        let compute_properties = analyzer.analyze_all();
        self.compute_properties = Some(compute_properties);
    }

    fn analyze_open_package(&mut self) {
        let Some(compute_properties) = &mut self.compute_properties else {
            panic!("cannot analyze open package if the other packages have not been analyzed");
        };

        // Clear the compute properties of the open package.
        let open_package_id = map_hir_package_to_fir(self.compiler.package_id());
        let package_compute_properties = compute_properties.get_mut(open_package_id);
        package_compute_properties.clear();

        // Analyze the open package without re-analyzing the other packages.
        let analyzer =
            Analyzer::init_with_compute_properties(&self.fir_store, compute_properties.clone());
        self.compute_properties = Some(analyzer.analyze_package(open_package_id));
    }

    fn update_compilation(&mut self, source: &str) {
        let increment = self
            .compiler
            .compile_fragments_fail_fast("rca-test", source)
            .expect("code should compile");
        let package_id = map_hir_package_to_fir(self.compiler.package_id());
        let fir_package = self.fir_store.get_mut(package_id);
        self.lowerer
            .lower_and_update_package(fir_package, &increment.hir);
        self.compiler.update(increment);
    }
}

impl Default for CompilationContext {
    fn default() -> Self {
        let compiler = Compiler::new(
            true,
            SourceMap::default(),
            PackageType::Lib,
            RuntimeCapabilityFlags::all(),
            LanguageFeatures::default(),
        )
        .expect("should be able to create a new compiler");
        let mut lowerer = Lowerer::new();
        let fir_store = lower_hir_package_store(&mut lowerer, compiler.package_store());
        Self {
            compiler,
            lowerer,
            fir_store,
            compute_properties: None,
        }
    }
}

fn lower_hir_package_store(
    lowerer: &mut Lowerer,
    hir_package_store: &HirPackageStore,
) -> PackageStore {
    let mut fir_store = PackageStore::new();
    for (id, unit) in hir_package_store {
        fir_store.insert(
            map_hir_package_to_fir(id),
            lowerer.lower_package(&unit.package),
        );
    }
    fir_store
}

criterion_group!(benches, core_and_std, teleport, deutsch_jozsa, large_file);
criterion_main!(benches);
