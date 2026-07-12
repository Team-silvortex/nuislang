use super::{
    NirAddressClass, NirContainerKind, NirResultFamily, NirScalarKind, NirTypeRef, NirTypeShape,
    NirWindowMode,
};

impl NirTypeRef {
    pub fn is_address_type(&self) -> bool {
        self.is_ref
    }

    pub fn address_target_name(&self) -> Option<&str> {
        self.is_address_type().then_some(self.name.as_str())
    }

    pub fn supports_address_class(&self, class: NirAddressClass) -> bool {
        self.is_address_type()
            && matches!(class, NirAddressClass::Owned | NirAddressClass::Borrowed)
    }

    pub fn scalar_kind(&self) -> Option<NirScalarKind> {
        if self.is_ref || !self.generic_args.is_empty() {
            return None;
        }
        match self.name.as_str() {
            "bool" => Some(NirScalarKind::Bool),
            "i32" => Some(NirScalarKind::I32),
            "i64" => Some(NirScalarKind::I64),
            "f32" => Some(NirScalarKind::F32),
            "f64" => Some(NirScalarKind::F64),
            "String" => Some(NirScalarKind::Text),
            "Unit" => Some(NirScalarKind::Unit),
            _ => None,
        }
    }

    pub fn shape(&self) -> NirTypeShape {
        if let Some(kind) = self.scalar_kind() {
            NirTypeShape::Scalar(kind)
        } else if self.is_ref {
            NirTypeShape::Ref
        } else if !self.generic_args.is_empty() {
            NirTypeShape::Generic
        } else {
            NirTypeShape::Nominal
        }
    }

    pub fn is_integer_scalar(&self) -> bool {
        matches!(
            self.scalar_kind(),
            Some(NirScalarKind::I32 | NirScalarKind::I64)
        )
    }

    pub fn is_float_scalar(&self) -> bool {
        matches!(
            self.scalar_kind(),
            Some(NirScalarKind::F32 | NirScalarKind::F64)
        )
    }

    pub fn is_numeric_scalar(&self) -> bool {
        self.is_integer_scalar() || self.is_float_scalar()
    }

    pub fn is_bool_scalar(&self) -> bool {
        self.scalar_kind() == Some(NirScalarKind::Bool)
    }

    pub fn is_text_scalar(&self) -> bool {
        self.scalar_kind() == Some(NirScalarKind::Text)
    }

    pub fn is_unit_scalar(&self) -> bool {
        self.scalar_kind() == Some(NirScalarKind::Unit)
    }

    pub fn is_generic_named(&self, expected: &str, arity: usize) -> bool {
        self.name == expected && self.generic_args.len() == arity && !self.is_ref
    }

    pub fn container_kind(&self) -> Option<NirContainerKind> {
        match self.name.as_str() {
            "Window" | "WindowMut" if !self.is_ref => Some(NirContainerKind::Window),
            "Pipe" if !self.is_ref => Some(NirContainerKind::Pipe),
            "Instance" if !self.is_ref => Some(NirContainerKind::Instance),
            "Task" if !self.is_ref => Some(NirContainerKind::Task),
            _ => None,
        }
    }

    pub fn window_mode(&self) -> Option<NirWindowMode> {
        if self.is_ref {
            return None;
        }
        match self.name.as_str() {
            "Window" => Some(NirWindowMode::Immutable),
            "WindowMut" => Some(NirWindowMode::Mutable),
            _ => None,
        }
    }

    pub fn container_payload(&self) -> Option<&NirTypeRef> {
        if matches!(
            self.container_kind(),
            Some(
                NirContainerKind::Window
                    | NirContainerKind::Pipe
                    | NirContainerKind::Instance
                    | NirContainerKind::Task
            )
        ) {
            self.generic_args.first()
        } else {
            None
        }
    }

    pub fn is_marker_type(&self) -> bool {
        self.name == "Marker" && !self.is_ref && self.generic_args.is_empty()
    }

    pub fn is_handle_table_type(&self) -> bool {
        self.name == "HandleTable" && !self.is_ref && self.generic_args.is_empty()
    }

    pub fn is_marker_family(&self) -> bool {
        self.name == "Marker" && !self.is_ref
    }

    pub fn is_handle_table_family(&self) -> bool {
        self.name == "HandleTable" && !self.is_ref
    }

    pub fn is_thread_family(&self) -> bool {
        self.name == "Thread" && !self.is_ref
    }

    pub fn is_mutex_family(&self) -> bool {
        self.name == "Mutex" && !self.is_ref
    }

    pub fn is_mutex_guard_family(&self) -> bool {
        self.name == "MutexGuard" && !self.is_ref
    }

    pub fn is_concurrency_bridge_family(&self) -> bool {
        self.is_thread_family() || self.is_mutex_family() || self.is_mutex_guard_family()
    }

    pub fn thread_payload(&self) -> Option<&NirTypeRef> {
        if self.is_thread_family() {
            self.generic_args.first()
        } else {
            None
        }
    }

    pub fn mutex_payload(&self) -> Option<&NirTypeRef> {
        if self.is_mutex_family() {
            self.generic_args.first()
        } else {
            None
        }
    }

    pub fn mutex_guard_payload(&self) -> Option<&NirTypeRef> {
        if self.is_mutex_guard_family() {
            self.generic_args.first()
        } else {
            None
        }
    }

    pub fn marker_tag(&self) -> Option<&NirTypeRef> {
        if self.is_marker_family() {
            self.generic_args.first()
        } else {
            None
        }
    }

    pub fn handle_table_schema(&self) -> Option<&NirTypeRef> {
        if self.is_handle_table_family() {
            self.generic_args.first()
        } else {
            None
        }
    }

    pub fn is_async_boundary_safe(&self) -> bool {
        if self.is_ref || self.is_optional {
            return false;
        }
        if matches!(
            self.container_kind(),
            Some(NirContainerKind::Instance | NirContainerKind::Task)
        ) {
            return false;
        }
        if self.is_concurrency_bridge_family() {
            return false;
        }
        if self.is_result_family() {
            return false;
        }
        self.generic_args
            .iter()
            .all(NirTypeRef::is_async_boundary_safe)
    }

    pub fn is_result_family(&self) -> bool {
        self.result_family().is_some()
    }

    pub fn result_family(&self) -> Option<NirResultFamily> {
        if self.is_ref || self.generic_args.len() != 1 {
            return None;
        }
        match self.name.as_str() {
            "TaskResult" => Some(NirResultFamily::Task),
            "DataResult" => Some(NirResultFamily::Data),
            "ShaderResult" => Some(NirResultFamily::Shader),
            "KernelResult" => Some(NirResultFamily::Kernel),
            "NetworkResult" => Some(NirResultFamily::Network),
            _ => None,
        }
    }

    pub fn result_payload(&self) -> Option<&NirTypeRef> {
        self.result_family()?;
        self.generic_args.first()
    }

    fn is_nominal_semantic_payload(&self) -> bool {
        !self.is_ref
            && !self.is_optional
            && self.scalar_kind().is_none()
            && self.container_kind().is_none()
            && !self.is_marker_family()
            && !self.is_handle_table_family()
    }

    pub fn validate_container_contract(&self) -> Result<(), String> {
        for arg in &self.generic_args {
            arg.validate_container_contract()?;
        }

        match self.container_kind() {
            Some(NirContainerKind::Window) => {
                if self.generic_args.len() != 1 {
                    return Err(format!(
                        "`{}` must carry exactly one payload type argument",
                        self.name
                    ));
                }
                let payload = self.container_payload().expect("window payload");
                if payload.is_marker_type() || payload.is_handle_table_type() {
                    return Err(format!(
                        "`Window<...>` cannot carry control-plane payload `{}`",
                        payload.render()
                    ));
                }
                if payload.container_kind() == Some(NirContainerKind::Pipe) {
                    return Err("`Window<Pipe<...>>` is not a valid memory payload".to_owned());
                }
            }
            Some(NirContainerKind::Pipe) => {
                if self.generic_args.len() != 1 {
                    return Err(format!(
                        "`{}` must carry exactly one payload type argument",
                        self.name
                    ));
                }
                let payload = self.container_payload().expect("pipe payload");
                if payload.is_marker_type() || payload.is_handle_table_type() {
                    return Err(format!(
                        "`Pipe<...>` cannot carry control-plane payload `{}`",
                        payload.render()
                    ));
                }
                if payload.container_kind() == Some(NirContainerKind::Pipe) {
                    return Err("`Pipe<Pipe<...>>` is not a legal fabric primitive".to_owned());
                }
            }
            Some(NirContainerKind::Instance) => {
                if self.generic_args.len() != 1 {
                    return Err(format!(
                        "`{}` must carry exactly one payload type argument",
                        self.name
                    ));
                }
                let payload = self.container_payload().expect("instance payload");
                if payload.is_ref
                    || payload.is_optional
                    || payload.scalar_kind().is_some()
                    || payload.is_marker_type()
                    || payload.is_handle_table_type()
                    || payload.container_kind().is_some()
                {
                    return Err(format!(
                        "`Instance<...>` expects a nominal unit type, found `{}`",
                        payload.render()
                    ));
                }
            }
            Some(NirContainerKind::Task) => {
                if self.generic_args.len() != 1 {
                    return Err(format!(
                        "`{}` must carry exactly one payload type argument",
                        self.name
                    ));
                }
                let payload = self.container_payload().expect("task payload");
                if !payload.is_async_boundary_safe() {
                    return Err(format!(
                        "`Task<...>` expects an async-boundary-safe payload, found `{}`",
                        payload.render()
                    ));
                }
                if payload.container_kind() == Some(NirContainerKind::Task) {
                    return Err(
                        "`Task<Task<...>>` is not a supported explicit async primitive".to_owned(),
                    );
                }
            }
            None => {
                if self.is_marker_family() {
                    if self.generic_args.len() > 1 {
                        return Err("`Marker<...>` accepts at most one tag type".to_owned());
                    }
                    if let Some(tag) = self.marker_tag() {
                        if !tag.is_nominal_semantic_payload() {
                            return Err(format!(
                                "`Marker<...>` expects a nominal tag type, found `{}`",
                                tag.render()
                            ));
                        }
                    }
                }
                if self.is_handle_table_family() {
                    if self.generic_args.len() > 1 {
                        return Err("`HandleTable<...>` accepts at most one schema type".to_owned());
                    }
                    if let Some(schema) = self.handle_table_schema() {
                        if !schema.is_nominal_semantic_payload() {
                            return Err(format!(
                                "`HandleTable<...>` expects a nominal schema type, found `{}`",
                                schema.render()
                            ));
                        }
                    }
                }
                if self.is_thread_family() {
                    if self.generic_args.len() != 1 {
                        return Err(
                            "`Thread<...>` must carry exactly one payload type argument".to_owned()
                        );
                    }
                    let payload = self.thread_payload().expect("thread payload");
                    if !payload.is_async_boundary_safe() || payload.is_concurrency_bridge_family() {
                        return Err(format!(
                            "`Thread<...>` expects a staged join payload that is async-boundary-safe and not itself a thread/lock family, found `{}`",
                            payload.render()
                        ));
                    }
                }
                if self.is_mutex_family() {
                    if self.generic_args.len() != 1 {
                        return Err(
                            "`Mutex<...>` must carry exactly one payload type argument".to_owned()
                        );
                    }
                    let payload = self.mutex_payload().expect("mutex payload");
                    if payload.is_ref
                        || payload.is_optional
                        || payload.is_result_family()
                        || payload.is_concurrency_bridge_family()
                    {
                        return Err(format!(
                            "`Mutex<...>` expects a staged value payload, found `{}`",
                            payload.render()
                        ));
                    }
                }
                if self.is_mutex_guard_family() {
                    if self.generic_args.len() != 1 {
                        return Err(
                            "`MutexGuard<...>` must carry exactly one payload type argument"
                                .to_owned(),
                        );
                    }
                    let payload = self.mutex_guard_payload().expect("mutex guard payload");
                    if payload.is_ref
                        || payload.is_optional
                        || payload.is_result_family()
                        || payload.is_concurrency_bridge_family()
                    {
                        return Err(format!(
                            "`MutexGuard<...>` expects a staged value payload, found `{}`",
                            payload.render()
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    pub fn render(&self) -> String {
        let mut out = String::new();
        if self.is_ref {
            out.push_str("ref ");
        }
        out.push_str(&self.name);
        if !self.generic_args.is_empty() {
            out.push('<');
            for (index, arg) in self.generic_args.iter().enumerate() {
                if index > 0 {
                    out.push_str(", ");
                }
                out.push_str(&arg.render());
            }
            out.push('>');
        }
        if self.is_optional {
            out.push('?');
        }
        out
    }
}
