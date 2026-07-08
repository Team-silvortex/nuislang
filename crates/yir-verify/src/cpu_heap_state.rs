#[derive(Clone, Copy)]
pub(super) enum PointerState {
    Null,
    Owned(usize),
    Borrowed(usize),
    Unknown,
}

#[derive(Clone, Copy)]
pub(super) enum HeapObjectKind {
    Node { next: PointerState },
    Buffer { len: Option<usize> },
}

#[derive(Clone, Copy)]
pub(super) struct HeapBinding {
    pub(super) live: bool,
    pub(super) kind: HeapObjectKind,
}
