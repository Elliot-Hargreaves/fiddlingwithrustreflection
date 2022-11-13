#![feature(trait_alias)]
use bevy_reflect::{Reflect, Struct};
use internal::{
    CommandType, Field, FieldInner, FieldParameter, IdentifiableObject, IsEmpty, ModifiableObject,
    ModificationCommand, Parameter, SoftwareVersion, VersionFilter,
};
use log::warn;
use std::{
    any::Any,
    cmp::Ordering,
    marker::PhantomData,
    ops::{Deref, DerefMut}, default,
};

pub mod internal;

const SYSTEM_VERSION: SoftwareVersion = SoftwareVersion::new(3, 188);

#[derive(Default, Clone, Reflect)]
enum UserFields {
    #[default]
    Id,
    Name,
    RoleId,
}

impl FieldParameter for UserFields {
    fn get_parameter<FieldType: Default + IsEmpty + Reflect + Clone>(
        &self,
        command_type: CommandType,
        old_value: &FieldInner<FieldType, UserFields>,
        new_value: &FieldInner<FieldType, UserFields>,
    ) -> Parameter {
        match self {
            Self::Id => unreachable!(),
            Self::Name => {
                if new_value.is_empty() {
                    Parameter::Flag("-reset_name")
                } else {
                    Parameter::Parameter("-set_name")
                }
            }
            Self::RoleId => Parameter::Parameter("-set_roleid"),
        }
    }
}

#[derive(Reflect)]
pub struct User {
    id: Field<u32, UserFields>,
    name: Field<String, UserFields>,
    role_id: Field<u32, UserFields>,
}

impl IdentifiableObject for User {
    fn get_id(&self) -> u32 {
        *self.id
    }
}

impl ModifiableObject for User {
    fn get_modify_command(&self) -> &'static str {
        "update_user"
    }
}

impl Default for User {
    fn default() -> Self {
        User {
            id: Field::new("id", UserFields::Id),
            name: Field::new("name", UserFields::Name),
            role_id: Field::new_versioned(
                "role_id",
                UserFields::RoleId,
                VersionFilter::min_version(SoftwareVersion::new(4, 12)),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut user: User = User::default();

        *user.id = 45;

        assert_eq!(*user.id, 45);
    }
}
