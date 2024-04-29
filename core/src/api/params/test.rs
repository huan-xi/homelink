mod test {
    use sea_orm::{DeriveActiveEnum, EnumIter};
    use serde::{Deserialize, Serialize};
    #[sea_orm(rs_type = "i32", db_type = "Integer")]
    pub enum DeviceType {
        Normal = 1,
        Gateway = 2,
        /// 子设备
        Child = 3,
    }

    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for DeviceType {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private::Result<__S::Ok, __S::Error>
                where
                    __S: _serde::Serializer,
            {
                match *self {
                    DeviceType::Normal => {
                        _serde::Serializer::serialize_unit_variant(
                            __serializer,
                            "DeviceType",
                            0u32,
                            "Normal",
                        )
                    }
                    DeviceType::Gateway => {
                        _serde::Serializer::serialize_unit_variant(
                            __serializer,
                            "DeviceType",
                            1u32,
                            "Gateway",
                        )
                    }
                    DeviceType::Child => {
                        _serde::Serializer::serialize_unit_variant(
                            __serializer,
                            "DeviceType",
                            2u32,
                            "Child",
                        )
                    }
                }
            }
        }
    };
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for DeviceType {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __field1,
                    __field2,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "variant identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private::Ok(__Field::__field0),
                            1u64 => _serde::__private::Ok(__Field::__field1),
                            2u64 => _serde::__private::Ok(__Field::__field2),
                            _ => {
                                _serde::__private::Err(
                                    _serde::de::Error::invalid_value(
                                        _serde::de::Unexpected::Unsigned(__value),
                                        &"variant index 0 <= i < 3",
                                    ),
                                )
                            }
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                    {
                        match __value {
                            "Normal" => _serde::__private::Ok(__Field::__field0),
                            "Gateway" => _serde::__private::Ok(__Field::__field1),
                            "Child" => _serde::__private::Ok(__Field::__field2),
                            _ => {
                                _serde::__private::Err(
                                    _serde::de::Error::unknown_variant(__value, VARIANTS),
                                )
                            }
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                    {
                        match __value {
                            b"Normal" => _serde::__private::Ok(__Field::__field0),
                            b"Gateway" => _serde::__private::Ok(__Field::__field1),
                            b"Child" => _serde::__private::Ok(__Field::__field2),
                            _ => {
                                let __value = &_serde::__private::from_utf8_lossy(__value);
                                _serde::__private::Err(
                                    _serde::de::Error::unknown_variant(__value, VARIANTS),
                                )
                            }
                        }
                    }
                }
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private::Result<Self, __D::Error>
                        where
                            __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private::PhantomData<DeviceType>,
                    lifetime: _serde::__private::PhantomData<&'de ()>,
                }
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = DeviceType;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "enum DeviceType",
                        )
                    }
                    fn visit_enum<__A>(
                        self,
                        __data: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                        where
                            __A: _serde::de::EnumAccess<'de>,
                    {
                        match _serde::de::EnumAccess::variant(__data)? {
                            (__Field::__field0, __variant) => {
                                _serde::de::VariantAccess::unit_variant(__variant)?;
                                _serde::__private::Ok(DeviceType::Normal)
                            }
                            (__Field::__field1, __variant) => {
                                _serde::de::VariantAccess::unit_variant(__variant)?;
                                _serde::__private::Ok(DeviceType::Gateway)
                            }
                            (__Field::__field2, __variant) => {
                                _serde::de::VariantAccess::unit_variant(__variant)?;
                                _serde::__private::Ok(DeviceType::Child)
                            }
                        }
                    }
                }
                #[doc(hidden)]
                const VARIANTS: &'static [&'static str] = &[
                    "Normal",
                    "Gateway",
                    "Child",
                ];
                _serde::Deserializer::deserialize_enum(
                    __deserializer,
                    "DeviceType",
                    VARIANTS,
                    __Visitor {
                        marker: _serde::__private::PhantomData::<DeviceType>,
                        lifetime: _serde::__private::PhantomData,
                    },
                )
            }
        }
    };
    /// Generated by sea-orm-macros
    pub struct DeviceTypeEnum;
    #[automatically_derived]
    impl ::core::fmt::Debug for DeviceTypeEnum {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(f, "DeviceTypeEnum")
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for DeviceTypeEnum {
        #[inline]
        fn clone(&self) -> DeviceTypeEnum {
            DeviceTypeEnum
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for DeviceTypeEnum {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for DeviceTypeEnum {
        #[inline]
        fn eq(&self, other: &DeviceTypeEnum) -> bool {
            true
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for DeviceTypeEnum {}
    #[automatically_derived]
    impl ::core::cmp::Eq for DeviceTypeEnum {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {}
    }
    #[automatically_derived]
    impl sea_orm::sea_query::Iden for DeviceTypeEnum {
        fn unquoted(&self, s: &mut dyn std::fmt::Write) {
            s.write_fmt(format_args!("{0}", "DeviceType")).unwrap();
        }
    }
    #[automatically_derived]
    impl sea_orm::ActiveEnum for DeviceType {
        type Value = i32;
        type ValueVec = Vec<i32>;
        fn name() -> sea_orm::sea_query::DynIden {
            sea_orm::sea_query::SeaRc::new(DeviceTypeEnum) as sea_orm::sea_query::DynIden
        }
        fn to_value(&self) -> Self::Value {
            match self {
                Self::Normal => 1,
                Self::Gateway => 2,
                Self::Child => 3,
            }
                .to_owned()
        }
        fn try_from_value(v: &Self::Value) -> std::result::Result<Self, sea_orm::DbErr> {
            match v {
                1 => Ok(Self::Normal),
                2 => Ok(Self::Gateway),
                3 => Ok(Self::Child),
                _ => {
                    Err(
                        sea_orm::DbErr::Type({
                            let res = ::alloc::fmt::format(
                                format_args!(
                                    "unexpected value for {0} enum: {1}",
                                    "DeviceType",
                                    v,
                                ),
                            );
                            res
                        }),
                    )
                }
            }
        }
        fn db_type() -> sea_orm::ColumnDef {
            sea_orm::prelude::ColumnTypeTrait::def(sea_orm::ColumnType::Integer)
        }
    }
    #[automatically_derived]
    #[allow(clippy::from_over_into)]
    impl Into<sea_orm::sea_query::Value> for DeviceType {
        fn into(self) -> sea_orm::sea_query::Value {
            <Self as sea_orm::ActiveEnum>::to_value(&self).into()
        }
    }
    #[automatically_derived]
    impl sea_orm::TryGetable for DeviceType {
        fn try_get_by<I: sea_orm::ColIdx>(
            res: &sea_orm::QueryResult,
            idx: I,
        ) -> std::result::Result<Self, sea_orm::TryGetError> {
            let value = <<Self as sea_orm::ActiveEnum>::Value as sea_orm::TryGetable>::try_get_by(
                res,
                idx,
            )?;
            <Self as sea_orm::ActiveEnum>::try_from_value(&value)
                .map_err(sea_orm::TryGetError::DbErr)
        }
    }
    #[automatically_derived]
    impl sea_orm::sea_query::ValueType for DeviceType {
        fn try_from(
            v: sea_orm::sea_query::Value,
        ) -> std::result::Result<Self, sea_orm::sea_query::ValueTypeErr> {
            let value = <<Self as sea_orm::ActiveEnum>::Value as sea_orm::sea_query::ValueType>::try_from(
                v,
            )?;
            <Self as sea_orm::ActiveEnum>::try_from_value(&value)
                .map_err(|_| sea_orm::sea_query::ValueTypeErr)
        }
        fn type_name() -> String {
            <<Self as sea_orm::ActiveEnum>::Value as sea_orm::sea_query::ValueType>::type_name()
        }
        fn array_type() -> sea_orm::sea_query::ArrayType {
            <<Self as sea_orm::ActiveEnum>::Value as sea_orm::sea_query::ValueType>::array_type()
        }
        fn column_type() -> sea_orm::sea_query::ColumnType {
            <Self as sea_orm::ActiveEnum>::db_type().get_column_type().to_owned().into()
        }
    }
    #[automatically_derived]
    impl sea_orm::sea_query::Nullable for DeviceType {
        fn null() -> sea_orm::sea_query::Value {
            <<Self as sea_orm::ActiveEnum>::Value as sea_orm::sea_query::Nullable>::null()
        }
    }
    ///An iterator over the variants of [DeviceType]
    #[allow(missing_copy_implementations)]
    pub struct DeviceTypeIter {
        idx: usize,
        back_idx: usize,
        marker: ::core::marker::PhantomData<()>,
    }
    impl core::fmt::Debug for DeviceTypeIter {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.debug_struct("DeviceTypeIter").field("len", &self.len()).finish()
        }
    }
    impl DeviceTypeIter {
        fn get(&self, idx: usize) -> Option<DeviceType> {
            match idx {
                0usize => ::core::option::Option::Some(DeviceType::Normal),
                1usize => ::core::option::Option::Some(DeviceType::Gateway),
                2usize => ::core::option::Option::Some(DeviceType::Child),
                _ => ::core::option::Option::None,
            }
        }
    }
    impl sea_orm::strum::IntoEnumIterator for DeviceType {
        type Iterator = DeviceTypeIter;
        fn iter() -> DeviceTypeIter {
            DeviceTypeIter {
                idx: 0,
                back_idx: 0,
                marker: ::core::marker::PhantomData,
            }
        }
    }
    impl Iterator for DeviceTypeIter {
        type Item = DeviceType;
        fn next(&mut self) -> Option<<Self as Iterator>::Item> {
            self.nth(0)
        }
        fn size_hint(&self) -> (usize, Option<usize>) {
            let t = if self.idx + self.back_idx >= 3usize {
                0
            } else {
                3usize - self.idx - self.back_idx
            };
            (t, Some(t))
        }
        fn nth(&mut self, n: usize) -> Option<<Self as Iterator>::Item> {
            let idx = self.idx + n + 1;
            if idx + self.back_idx > 3usize {
                self.idx = 3usize;
                ::core::option::Option::None
            } else {
                self.idx = idx;
                self.get(idx - 1)
            }
        }
    }
    impl ExactSizeIterator for DeviceTypeIter {
        fn len(&self) -> usize {
            self.size_hint().0
        }
    }
    impl DoubleEndedIterator for DeviceTypeIter {
        fn next_back(&mut self) -> Option<<Self as Iterator>::Item> {
            let back_idx = self.back_idx + 1;
            if self.idx + back_idx > 3usize {
                self.back_idx = 3usize;
                ::core::option::Option::None
            } else {
                self.back_idx = back_idx;
                self.get(3usize - self.back_idx)
            }
        }
    }
    impl Clone for DeviceTypeIter {
        fn clone(&self) -> DeviceTypeIter {
            DeviceTypeIter {
                idx: self.idx,
                back_idx: self.back_idx,
                marker: self.marker.clone(),
            }
        }
    }
}