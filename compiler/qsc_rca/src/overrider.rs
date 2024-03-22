// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    common::LocalSpecId, scaffolding::InternalPackageStoreComputeProperties,
    ApplicationGeneratorSet, ArrayParamApplication, ComputeKind, PackageId, ParamApplication,
    QuantumProperties, RuntimeFeatureFlags, RuntimeKind, ValueKind,
};
use qsc_fir::{
    fir::{
        Block, BlockId, CallableImpl, Expr, ExprId, Global, Item, ItemKind, LocalItemId, Package,
        PackageStore, PackageStoreLookup, Pat, PatId, Stmt, StmtId, StmtKind,
    },
    ty::{FunctorSetValue, Ty},
    visit::{walk_block, walk_expr, walk_stmt, Visitor},
};
use rustc_hash::FxHashMap;

#[derive(Clone, Debug)]
struct SpecOverride {
    functor_set_value: FunctorSetValue,
    application_generator_set: ApplicationGeneratorSet,
}

pub struct Overrider<'a> {
    package_store: &'a PackageStore,
    package_store_compute_properties: InternalPackageStoreComputeProperties,
    overrides: FxHashMap<String, Vec<SpecOverride>>,
    current_package: Option<PackageId>,
    current_application_generator_set: Option<ApplicationGeneratorSet>,
}

impl<'a> Overrider<'a> {
    #[allow(clippy::too_many_lines)]
    pub fn new(
        package_store: &'a PackageStore,
        package_store_compute_properties: InternalPackageStoreComputeProperties,
    ) -> Self {
        let callable_overrides_tuples: [(String, Vec<SpecOverride>); 5] = [
            (
                "Microsoft.Quantum.Core.Length".into(),
                vec![SpecOverride {
                    functor_set_value: FunctorSetValue::Empty,
                    application_generator_set: ApplicationGeneratorSet {
                        inherent: ComputeKind::Classical,
                        dynamic_param_applications: vec![ParamApplication::Array(
                            ArrayParamApplication {
                                static_content_dynamic_size: ComputeKind::Quantum(
                                    QuantumProperties {
                                        runtime_features:
                                            RuntimeFeatureFlags::UseOfDynamicallySizedArray,
                                        value_kind: ValueKind::Element(RuntimeKind::Dynamic),
                                    },
                                ),
                                dynamic_content_static_size: ComputeKind::Quantum(
                                    QuantumProperties {
                                        runtime_features: RuntimeFeatureFlags::empty(),
                                        value_kind: ValueKind::Element(RuntimeKind::Static),
                                    },
                                ),
                                dynamic_content_dynamic_size: ComputeKind::Quantum(
                                    QuantumProperties {
                                        runtime_features:
                                            RuntimeFeatureFlags::UseOfDynamicallySizedArray,
                                        value_kind: ValueKind::Element(RuntimeKind::Dynamic),
                                    },
                                ),
                            },
                        )],
                    },
                }],
            ),
            (
                "Microsoft.Quantum.Intrinsic.H".into(),
                vec![
                    SpecOverride {
                        functor_set_value: FunctorSetValue::Ctl,
                        application_generator_set: ApplicationGeneratorSet {
                            inherent: ComputeKind::Quantum(QuantumProperties {
                                runtime_features: RuntimeFeatureFlags::empty(),
                                value_kind: ValueKind::Element(RuntimeKind::Static),
                            }),
                            dynamic_param_applications: vec![ParamApplication::Element(
                                ComputeKind::Quantum(QuantumProperties {
                                    runtime_features: RuntimeFeatureFlags::UseOfDynamicQubit,
                                    value_kind: ValueKind::Element(RuntimeKind::Static),
                                }),
                            )],
                        },
                    },
                    SpecOverride {
                        functor_set_value: FunctorSetValue::CtlAdj,
                        application_generator_set: ApplicationGeneratorSet {
                            inherent: ComputeKind::Quantum(QuantumProperties {
                                runtime_features: RuntimeFeatureFlags::empty(),
                                value_kind: ValueKind::Element(RuntimeKind::Static),
                            }),
                            dynamic_param_applications: vec![ParamApplication::Element(
                                ComputeKind::Quantum(QuantumProperties {
                                    runtime_features: RuntimeFeatureFlags::UseOfDynamicQubit,
                                    value_kind: ValueKind::Element(RuntimeKind::Static),
                                }),
                            )],
                        },
                    },
                ],
            ),
            (
                "Microsoft.Quantum.Intrinsic.X".into(),
                vec![
                    SpecOverride {
                        functor_set_value: FunctorSetValue::Ctl,
                        application_generator_set: ApplicationGeneratorSet {
                            inherent: ComputeKind::Quantum(QuantumProperties {
                                runtime_features: RuntimeFeatureFlags::empty(),
                                value_kind: ValueKind::Element(RuntimeKind::Static),
                            }),
                            dynamic_param_applications: vec![ParamApplication::Element(
                                ComputeKind::Quantum(QuantumProperties {
                                    runtime_features: RuntimeFeatureFlags::UseOfDynamicQubit,
                                    value_kind: ValueKind::Element(RuntimeKind::Static),
                                }),
                            )],
                        },
                    },
                    SpecOverride {
                        functor_set_value: FunctorSetValue::CtlAdj,
                        application_generator_set: ApplicationGeneratorSet {
                            inherent: ComputeKind::Quantum(QuantumProperties {
                                runtime_features: RuntimeFeatureFlags::empty(),
                                value_kind: ValueKind::Element(RuntimeKind::Static),
                            }),
                            dynamic_param_applications: vec![ParamApplication::Element(
                                ComputeKind::Quantum(QuantumProperties {
                                    runtime_features: RuntimeFeatureFlags::UseOfDynamicQubit,
                                    value_kind: ValueKind::Element(RuntimeKind::Static),
                                }),
                            )],
                        },
                    },
                ],
            ),
            (
                "Microsoft.Quantum.Intrinsic.Y".into(),
                vec![
                    SpecOverride {
                        functor_set_value: FunctorSetValue::Ctl,
                        application_generator_set: ApplicationGeneratorSet {
                            inherent: ComputeKind::Quantum(QuantumProperties {
                                runtime_features: RuntimeFeatureFlags::empty(),
                                value_kind: ValueKind::Element(RuntimeKind::Static),
                            }),
                            dynamic_param_applications: vec![ParamApplication::Element(
                                ComputeKind::Quantum(QuantumProperties {
                                    runtime_features: RuntimeFeatureFlags::UseOfDynamicQubit,
                                    value_kind: ValueKind::Element(RuntimeKind::Static),
                                }),
                            )],
                        },
                    },
                    SpecOverride {
                        functor_set_value: FunctorSetValue::CtlAdj,
                        application_generator_set: ApplicationGeneratorSet {
                            inherent: ComputeKind::Quantum(QuantumProperties {
                                runtime_features: RuntimeFeatureFlags::empty(),
                                value_kind: ValueKind::Element(RuntimeKind::Static),
                            }),
                            dynamic_param_applications: vec![ParamApplication::Element(
                                ComputeKind::Quantum(QuantumProperties {
                                    runtime_features: RuntimeFeatureFlags::UseOfDynamicQubit,
                                    value_kind: ValueKind::Element(RuntimeKind::Static),
                                }),
                            )],
                        },
                    },
                ],
            ),
            (
                "Microsoft.Quantum.Intrinsic.Z".into(),
                vec![
                    SpecOverride {
                        functor_set_value: FunctorSetValue::Ctl,
                        application_generator_set: ApplicationGeneratorSet {
                            inherent: ComputeKind::Quantum(QuantumProperties {
                                runtime_features: RuntimeFeatureFlags::empty(),
                                value_kind: ValueKind::Element(RuntimeKind::Static),
                            }),
                            dynamic_param_applications: vec![ParamApplication::Element(
                                ComputeKind::Quantum(QuantumProperties {
                                    runtime_features: RuntimeFeatureFlags::UseOfDynamicQubit,
                                    value_kind: ValueKind::Element(RuntimeKind::Static),
                                }),
                            )],
                        },
                    },
                    SpecOverride {
                        functor_set_value: FunctorSetValue::CtlAdj,
                        application_generator_set: ApplicationGeneratorSet {
                            inherent: ComputeKind::Quantum(QuantumProperties {
                                runtime_features: RuntimeFeatureFlags::empty(),
                                value_kind: ValueKind::Element(RuntimeKind::Static),
                            }),
                            dynamic_param_applications: vec![ParamApplication::Element(
                                ComputeKind::Quantum(QuantumProperties {
                                    runtime_features: RuntimeFeatureFlags::UseOfDynamicQubit,
                                    value_kind: ValueKind::Element(RuntimeKind::Static),
                                }),
                            )],
                        },
                    },
                ],
            ),
        ];
        let mut overrides: FxHashMap<String, Vec<SpecOverride>> = FxHashMap::default();
        for (fully_qualified_name, application_generator_set_override) in callable_overrides_tuples
        {
            overrides.insert(fully_qualified_name, application_generator_set_override);
        }
        Self {
            package_store,
            package_store_compute_properties,
            overrides,
            current_package: None,
            current_application_generator_set: None,
        }
    }

    pub fn populate_overrides(mut self) -> InternalPackageStoreComputeProperties {
        for (package_id, package) in self.package_store {
            self.populate_package_internal(package_id, package);
        }
        self.package_store_compute_properties
    }

    fn clear_current_application_generator_set(&mut self) {
        assert!(self.current_application_generator_set.is_some());
        _ = self.current_application_generator_set.take();
    }

    fn get_current_application_generator_set(&self) -> &ApplicationGeneratorSet {
        self.current_application_generator_set
            .as_ref()
            .expect("current application generator set is none")
    }

    fn get_current_package(&self) -> PackageId {
        self.current_package
            .expect("current package should be valid")
    }

    fn get_item(&self, id: LocalItemId) -> &'a Item {
        let package_id = self.get_current_package();
        self.package_store
            .get(package_id)
            .items
            .get(id)
            .expect("item not found")
    }

    fn populate_package_internal(&mut self, package_id: PackageId, package: &'a Package) {
        self.current_package = Some(package_id);
        self.visit_package(package);
        self.current_package = None;
    }

    fn populate_spec_application_generator_set(&mut self, spec_id: LocalSpecId) {
        let package_id = self.get_current_package();
        let Some(Global::Callable(callable_decl)) = self
            .package_store
            .get_global((package_id, spec_id.callable).into())
        else {
            panic!("global item not found");
        };

        // If the specialization is not intrinsic, we need to visit the implementation to populate the properties of its
        // elements.
        if let CallableImpl::Spec(spec_impl) = &callable_decl.implementation {
            let spec_decl = match spec_id.functor_set_value {
                FunctorSetValue::Empty => &spec_impl.body,
                FunctorSetValue::Adj => spec_impl
                    .adj
                    .as_ref()
                    .expect("adj specialization should exist"),
                FunctorSetValue::Ctl => spec_impl
                    .ctl
                    .as_ref()
                    .expect("ctl specialization should exist"),
                FunctorSetValue::CtlAdj => spec_impl
                    .ctl_adj
                    .as_ref()
                    .expect("ctl_adj specializatiob should exist"),
            };
            self.visit_spec_decl(spec_decl);
        }

        // Insert the application generator set to the compute properties data structure.
        let application_generator_set = self.get_current_application_generator_set().clone();
        self.package_store_compute_properties
            .insert_spec((package_id, spec_id).into(), application_generator_set);
    }

    fn set_current_application_generator_set(&mut self, value: ApplicationGeneratorSet) {
        assert!(self.current_application_generator_set.is_none());
        self.current_application_generator_set = Some(value);
    }
}

impl<'a> Visitor<'a> for Overrider<'a> {
    fn get_block(&self, id: BlockId) -> &'a Block {
        let package_id = self.get_current_package();
        self.package_store.get_block((package_id, id).into())
    }

    fn get_expr(&self, id: ExprId) -> &'a Expr {
        let package_id = self.get_current_package();
        self.package_store.get_expr((package_id, id).into())
    }

    fn get_pat(&self, id: PatId) -> &'a Pat {
        let package_id = self.get_current_package();
        self.package_store.get_pat((package_id, id).into())
    }

    fn get_stmt(&self, id: StmtId) -> &'a Stmt {
        let package_id = self.get_current_package();
        self.package_store.get_stmt((package_id, id).into())
    }

    fn visit_block(&mut self, id: BlockId) {
        walk_block(self, id);
        let package_id = self.get_current_package();
        let block = self.get_block(id);
        let application_generator_set = adapt_application_generator_set_to_type(
            self.get_current_application_generator_set(),
            &block.ty,
        );
        self.package_store_compute_properties
            .insert_block((package_id, id).into(), application_generator_set);
    }

    fn visit_expr(&mut self, id: ExprId) {
        walk_expr(self, id);
        let package_id = self.get_current_package();
        let expr = self.get_expr(id);
        let application_generator_set = adapt_application_generator_set_to_type(
            self.get_current_application_generator_set(),
            &expr.ty,
        );
        self.package_store_compute_properties
            .insert_expr((package_id, id).into(), application_generator_set);
    }

    fn visit_package(&mut self, package: &'a Package) {
        // Go through each namespace, identifying the callables for which we have overrides.
        let namespaces = package
            .items
            .iter()
            .filter_map(|(_, item)| match &item.kind {
                ItemKind::Namespace(ident, items) => Some((ident.name.to_string(), items)),
                _ => None,
            });
        for (namespace_ident, namespace_items) in namespaces {
            let callables = namespace_items
                .iter()
                .filter_map(|item_id| {
                    let item = self.get_item(*item_id);
                    match &item.kind {
                        ItemKind::Callable(decl) => Some((item.id, decl.name.name.to_string())),
                        _ => None,
                    }
                })
                .collect::<Vec<_>>();

            // If a callable has overrides, populate them.
            for (callable_id, callable_name) in callables {
                let fully_qualified_name = namespace_ident.clone() + "." + &callable_name;
                if let Some(spec_overrides) = self.overrides.get(&fully_qualified_name) {
                    let spec_overrides = spec_overrides.clone();
                    for spec_override in spec_overrides {
                        self.set_current_application_generator_set(
                            spec_override.application_generator_set,
                        );
                        self.populate_spec_application_generator_set(
                            (callable_id, spec_override.functor_set_value).into(),
                        );
                        self.clear_current_application_generator_set();
                    }
                }
            }
        }
    }

    fn visit_stmt(&mut self, id: StmtId) {
        walk_stmt(self, id);
        let package_id = self.get_current_package();
        let stmt = self.get_stmt(id);
        let stmt_type = match stmt.kind {
            StmtKind::Expr(expr_id) => self.get_expr(expr_id).ty.clone(),
            StmtKind::Item(_) | StmtKind::Local(_, _, _) | StmtKind::Semi(_) => Ty::UNIT,
        };
        let application_generator_set = adapt_application_generator_set_to_type(
            self.get_current_application_generator_set(),
            &stmt_type,
        );
        self.package_store_compute_properties
            .insert_stmt((package_id, id).into(), application_generator_set);
    }
}

fn adapt_application_generator_set_to_type(
    application_generator_set: &ApplicationGeneratorSet,
    ty: &Ty,
) -> ApplicationGeneratorSet {
    let inherent = adapt_compute_kind_to_type(application_generator_set.inherent, ty);
    let mut dynamic_param_applications = Vec::new();
    for param_application in &application_generator_set.dynamic_param_applications {
        let param_application = adapt_param_application_to_type(param_application, ty);
        dynamic_param_applications.push(param_application);
    }
    ApplicationGeneratorSet {
        inherent,
        dynamic_param_applications,
    }
}

fn adapt_compute_kind_to_type(compute_kind: ComputeKind, ty: &Ty) -> ComputeKind {
    match compute_kind {
        ComputeKind::Classical => ComputeKind::Classical,
        ComputeKind::Quantum(quantum_properties) => {
            let runtime_features = quantum_properties.runtime_features;
            let mut value_kind = ValueKind::new_static_from_type(ty);
            quantum_properties
                .value_kind
                .project_onto_variant(&mut value_kind);
            ComputeKind::Quantum(QuantumProperties {
                runtime_features,
                value_kind,
            })
        }
    }
}

fn adapt_param_application_to_type(
    param_application: &ParamApplication,
    ty: &Ty,
) -> ParamApplication {
    match param_application {
        ParamApplication::Array(array_param_application) => {
            let static_content_dynamic_size =
                adapt_compute_kind_to_type(array_param_application.static_content_dynamic_size, ty);
            let dynamic_content_static_size =
                adapt_compute_kind_to_type(array_param_application.dynamic_content_static_size, ty);
            let dynamic_content_dynamic_size = adapt_compute_kind_to_type(
                array_param_application.dynamic_content_dynamic_size,
                ty,
            );
            ParamApplication::Array(ArrayParamApplication {
                static_content_dynamic_size,
                dynamic_content_static_size,
                dynamic_content_dynamic_size,
            })
        }
        ParamApplication::Element(compute_kind) => {
            let compute_kind = adapt_compute_kind_to_type(*compute_kind, ty);
            ParamApplication::Element(compute_kind)
        }
    }
}
