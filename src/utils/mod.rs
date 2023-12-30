use std::{marker::PhantomData, cell::Cell, sync::MutexGuard};

pub type PhantomUnsync = PhantomData<Cell<()>>;
pub type PhantomUnsend = PhantomData<MutexGuard<'static, ()>>;