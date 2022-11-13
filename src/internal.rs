use std::{
    cmp::Ordering,
    default,
    marker::PhantomData,
    ops::{Deref, DerefMut}, any::Any,
};

use bevy_reflect::{Reflect, Struct};
use log::warn;

use crate::SYSTEM_VERSION;

#[derive(Default, PartialEq, Eq, Debug, Clone, Copy, Reflect)]
pub struct SoftwareVersion {
    major: u32,
    minor: u32,
}

impl PartialOrd for SoftwareVersion {
    fn ge(&self, other: &Self) -> bool {
        self == other || self > other
    }
    fn gt(&self, other: &Self) -> bool {
        self.major > other.major || (self.major == other.major && self.minor > other.major)
    }
    fn le(&self, other: &Self) -> bool {
        self == other || self < other
    }
    fn lt(&self, other: &Self) -> bool {
        self.major < other.major || (self.major == other.major && self.minor < other.minor)
    }
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self > other {
            Some(Ordering::Greater)
        } else if self == other {
            Some(Ordering::Equal)
        } else {
            Some(Ordering::Less)
        }
    }
}

impl SoftwareVersion {
    pub const fn new(major: u32, minor: u32) -> Self {
        SoftwareVersion { major, minor }
    }
}

pub trait FieldDataType = Default + Clone + Reflect;
pub trait FieldEnumType = Default + Clone + Reflect;

#[derive(Default, Reflect, Clone)]
pub struct FieldInner<InnerType: FieldDataType, FieldEnum: FieldEnumType> {
    min_version: Option<SoftwareVersion>,
    max_version: Option<SoftwareVersion>,
    field_name: String,
    field_enum: FieldEnum,
    value: InnerType,
    old_value: Option<InnerType>,
}

#[derive(Reflect, Clone)]
pub enum VersionFilter {
    MinVersion(SoftwareVersion),
    MaxVersion(SoftwareVersion),
    VersionRange(SoftwareVersion, SoftwareVersion),
}

impl VersionFilter {
    pub fn min_version(version: SoftwareVersion) -> Self {
        VersionFilter::MinVersion(version)
    }

    pub fn max_version(version: SoftwareVersion) -> Self {
        VersionFilter::MaxVersion(version)
    }

    pub fn version_range(min_version: SoftwareVersion, max_version: SoftwareVersion) -> Self {
        VersionFilter::VersionRange(min_version, max_version)
    }
}

#[derive(Reflect, Clone)]
pub enum Field<T: FieldDataType, FieldEnum: FieldEnumType> {
    Field(FieldInner<T, FieldEnum>),
    VersionedField(FieldInner<T, FieldEnum>, VersionFilter),
}

impl<T: FieldDataType, FieldEnum: FieldEnumType> Field<T, FieldEnum> {
    pub fn new(field_name: impl Into<String>, field: FieldEnum) -> Self {
        Field::Field(FieldInner::new(field_name, field))
    }

    pub fn new_versioned(field_name: impl Into<String>, field: FieldEnum, versions: VersionFilter) -> Self {
        Field::VersionedField(FieldInner::new(field_name, field), versions)
    }
}

impl<T: FieldDataType, FieldEnum: FieldEnumType> FieldInner<T, FieldEnum> {
    pub fn new(field_name: impl Into<String>, field: FieldEnum) -> Self {
        FieldInner {
            field_name: field_name.into(),
            min_version: None,
            max_version: None,
            value: T::default(),
            old_value: None,
            field_enum: field
        }
    }
}

impl<T: FieldDataType, FieldEnum: FieldEnumType> Deref for Field<T, FieldEnum> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        match self {
            Field::Field(inner) => inner,
            Field::VersionedField(inner, _) => inner,
        }
    }
}

impl<T: FieldDataType, FieldEnum: FieldEnumType> DerefMut for Field<T, FieldEnum> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Field::Field(inner) => inner,
            Field::VersionedField(inner, _) => inner,
        }
    }
}

impl<T: FieldDataType, FieldEnum: FieldEnumType> Deref for FieldInner<T, FieldEnum> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: FieldDataType, FieldEnum: FieldEnumType> DerefMut for FieldInner<T, FieldEnum> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        if self.old_value.is_none() {
            self.old_value = Some(self.value.clone());
        }
        if let Some(min_version) = self.min_version {
            if SYSTEM_VERSION < min_version {
                warn!(
                    "Field not supported on {:?}, min version is {:?}",
                    SYSTEM_VERSION, min_version
                );
            }
        }

        if let Some(max_version) = self.max_version {
            if SYSTEM_VERSION > max_version {
                warn!(
                    "Field not supported on {:?}, max version is {:?}",
                    SYSTEM_VERSION, max_version
                );
            }
        }

        &mut self.value
    }
}

pub trait IdentifiableObject {
    fn get_id(&self) -> u32;
}

pub trait ModifiableObject: IdentifiableObject {
    fn get_modify_command(&self) -> &'static str;
}

trait SystemObject: PartialEq {
    type field_enum: FieldParameter;

    fn eq(&self, rhs: &Self) -> bool {
        PartialEq::eq(self, &rhs)
    }
}

pub enum CommandType {
    Create,
    Modify,
    Delete,
}

pub enum Parameter {
    Parameter(&'static str),
    Flag(&'static str),
}

pub trait FieldParameter: Default + Reflect + Clone {
    fn get_parameter<FieldType: Default + IsEmpty + Reflect + Clone + Reflect>(
        &self,
        command_type: CommandType,
        old_value: &FieldInner<FieldType, Self>,
        new_value: &FieldInner<FieldType, Self>,
    ) -> Parameter;
}

pub trait IsEmpty {
    fn is_empty(&self) -> bool;
}

impl IsEmpty for String {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

pub struct ModificationCommand {
}

pub struct Object<Type: Reflect + Clone> {
    old: Type,
    new: Type,
}

// to do this automatically need to be able to downcast into some concrete type that can perform the check

// fn filter_changed_fields(field: &&dyn Reflect) -> bool {
//     if let Some(field) = field.downcast_ref::<Field<dyn SystemObject, _>>() {
//         match field {
//             Field::Field(inner) => inner.old_value.is_some(),
//             Field::VersionedField(inner, _) => inner.old_value.is_some()
//         }
//     } else {
//         panic!("Couldn't downcast to field type")
//     }
// }


// struct IntoModificationCommant<ObjectType: ModifiableObject + Struct + SystemObject>(ObjectType);
// impl<ObjectType: ModifiableObject + Struct + SystemObject> Into<ModificationCommand>
//     for IntoModificationCommant<ObjectType>
// {
//     fn into(self) -> ModificationCommand {
//         let inner = self.0;

//         let object_id = format!("{}", inner.get_id());
//         let command_string = String::from(inner.get_modify_command());

//         let command_arguments: Vec<&dyn Reflect> = inner.iter_fields().filter(filter_changed_fields).collect();

//         ModificationCommand {
//         }
//     }
// }
