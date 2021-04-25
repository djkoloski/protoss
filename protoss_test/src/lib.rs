#[cfg(test)]
mod tests {
    #[cfg(feature = "rkyv")]
    #[test]
    fn rkyv() {
        use protoss::Partial;
        use rkyv::ser::{Serializer, serializers::WriteSerializer};

        let mut serializer = WriteSerializer::new(Vec::new());
        let value = Partial::new(TestV1 {
            a: 42,
            c: 100,
            b: "hello world".into(),
        });
        let pos = serializer.serialize_value(&value).expect("failed to serialize value");
        let buffer = serializer.into_inner();

        let as_v0 = unsafe { rkyv::archived_value::<Box<Partial<TestV0>>>(buffer.as_slice(), pos) };
    }
}
