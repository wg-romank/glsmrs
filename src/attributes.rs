pub enum AttributeType {
    Scal(AttributeScalar),
    Vec2(AttributeVector2),
    Vec3(AttributeVector3),
}

impl AttributeType {
    pub fn num_components(&self) -> i32 {
        match &self {
            &AttributeType::Scal(_) => 1,
            &AttributeType::Vec2(_) => 2,
            &AttributeType::Vec3(_) => 3,
        }
    }
}

pub trait Attribute {
    type Repr: ?Sized;

    fn new(name: &'static str) -> AttributeType;

    fn pack(data: &Self::Repr) -> Vec<u8>;
}

pub struct AttributeScalar(pub &'static str);

impl Attribute for AttributeScalar {
    type Repr = [f32];

    fn new(name: &'static str) -> AttributeType {
        AttributeType::Scal(AttributeScalar(name))
    }

    fn pack(data: &Self::Repr) -> Vec<u8> {
        data.iter().flat_map(|e| e.to_ne_bytes()).collect::<Vec<u8>>()
    }
}

pub struct AttributeVector2(pub &'static str);

impl Attribute for AttributeVector2 {
    type Repr = [[f32; 2]];
    
    fn new(name: &'static str) -> AttributeType {
        AttributeType::Vec2(AttributeVector2(name))
    }
    fn pack(data: &Self::Repr) -> Vec<u8> {
        data.iter().flat_map(|ee| ee.iter().flat_map(|e| e.to_ne_bytes())).collect::<Vec<u8>>()
    }
}

pub struct AttributeVector3(pub &'static str);

impl Attribute for AttributeVector3 {
    type Repr = [[f32; 3]];

    fn new(name: &'static str) -> AttributeType {
        AttributeType::Vec3(AttributeVector3(name))
    }
    fn pack(data: &Self::Repr) -> Vec<u8> {
        data.iter().flat_map(|ee| ee.iter().flat_map(|e| e.to_ne_bytes())).collect::<Vec<u8>>()
    }
}
