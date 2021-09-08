use protoss::protoss;
use rkyv::{Archive, Serialize, Deserialize};

#[protoss(rkyv)]
#[derive(Archive, Serialize, Deserialize)]
struct Test {
    #[version = 0]
    pub a: i32,
    pub b: i32,
    #[version = 1]
    pub c: u32,
    pub d: u8,
}

#[cfg(test)]
pub mod tests {
    use protoss::{Partial, protoss};
    use rkyv::{archived_root, Archive, Deserialize, Serialize, ser::{serializers::AllocSerializer, Serializer}};

    type DefaultSerializer = AllocSerializer<256>;

    #[test]
    fn basic_archiving() {
        #[protoss(rkyv)]
        #[derive(Archive, Serialize, Deserialize)]
        struct Test {
            #[version = 0]
            pub a: i32,
            pub b: i32,
            #[version = 1]
            pub c: u32,
            pub d: u8,
        }

        let test_v0 = Test::partial_v0(1, 2);

        let mut serializer = DefaultSerializer::default();
        serializer.serialize_value(&test_v0).unwrap();
        let buf = serializer.into_serializer().into_inner();

        let archived_v0 = unsafe { archived_root::<Partial<Test>>(&buf) };
        assert_eq!(archived_v0.a(), test_v0.parts().a());
        assert_eq!(archived_v0.b(), test_v0.parts().b());
        assert_eq!(archived_v0.c(), None);
        assert_eq!(archived_v0.d(), None);
    }
}
