use serde::{Serialize, Deserialize};
use wither::Model;

#[derive(Default, Serialize, Deserialize, Model)]
#[model(read_concern="local")]
struct Model0 {
    #[serde(rename="_id", skip_serializing_if="Option::is_none")]
    pub id: Option<wither::bson::oid::ObjectId>,
}

#[derive(Default, Serialize, Deserialize, Model)]
#[model(read_concern="majority")]
struct Model1 {
    #[serde(rename="_id", skip_serializing_if="Option::is_none")]
    pub id: Option<wither::bson::oid::ObjectId>,
}

#[derive(Default, Serialize, Deserialize, Model)]
#[model(read_concern="linearizable")]
struct Model2 {
    #[serde(rename="_id", skip_serializing_if="Option::is_none")]
    pub id: Option<wither::bson::oid::ObjectId>,
}

#[derive(Default, Serialize, Deserialize, Model)]
#[model(read_concern="available")]
struct Model3 {
    #[serde(rename="_id", skip_serializing_if="Option::is_none")]
    pub id: Option<wither::bson::oid::ObjectId>,
}

#[derive(Default, Serialize, Deserialize, Model)]
#[model(read_concern(custom="custom-concern"))]
struct Model4 {
    #[serde(rename="_id", skip_serializing_if="Option::is_none")]
    pub id: Option<wither::bson::oid::ObjectId>,
}

fn main() {
    use wither::mongodb::options::ReadConcern;
    let _m0 = Model0::default();
    let _m1 = Model1::default();
    let _m2 = Model2::default();
    let _m3 = Model3::default();
    let _m4 = Model4::default();
    assert_eq!(Model0::read_concern(), Some(ReadConcern::local()));
    assert_eq!(Model1::read_concern(), Some(ReadConcern::majority()));
    assert_eq!(Model2::read_concern(), Some(ReadConcern::linearizable()));
    assert_eq!(Model3::read_concern(), Some(ReadConcern::available()));
    assert_eq!(Model4::read_concern(), Some(ReadConcern::custom(String::from("custom-concern"))));
}
