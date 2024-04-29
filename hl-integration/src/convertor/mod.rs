use std::fmt::{Debug, Pointer};
use impl_new::New;
use crate::convertor::ext::ConvertorExtPointer;

pub mod ext_factory;
pub mod ext;

pub mod buildin;


#[derive(Clone, New)]
pub struct UnitConvertor {
    pub ext: ConvertorExtPointer,
}
