use serde::{Serialize, Deserialize};
use wither::Model;

#[derive(Default, Serialize, Deserialize, Model)]
#[model(write_concern(w="majority", w_timeout=10, journal=true))]
struct Model0 {
    #[serde(rename="_id", skip_serializing_if="Option::is_none")]
    pub id: Option<wither::bson::oid::ObjectId>,
}

#[derive(Default, Serialize, Deserialize, Model)]
#[model(write_concern(w(nodes=3), w_timeout=0, journal=false))]
struct Model1 {
    #[serde(rename="_id", skip_serializing_if="Option::is_none")]
    pub id: Option<wither::bson::oid::ObjectId>,
}

#[derive(Default, Serialize, Deserialize, Model)]
#[model(write_concern(w(custom="custom")))]
struct Model2 {
    #[serde(rename="_id", skip_serializing_if="Option::is_none")]
    pub id: Option<wither::bson::oid::ObjectId>,
}

#[derive(Default, Serialize, Deserialize, Model)]
#[model(write_concern(w_timeout=999))]
struct Model3 {
    #[serde(rename="_id", skip_serializing_if="Option::is_none")]
    pub id: Option<wither::bson::oid::ObjectId>,
}

#[derive(Default, Serialize, Deserialize, Model)]
#[model(write_concern(journal=true))]
struct Model4 {
    #[serde(rename="_id", skip_serializing_if="Option::is_none")]
    pub id: Option<wither::bson::oid::ObjectId>,
}

#[derive(Default, Serialize, Deserialize, Model)]
#[model(write_concern())]
struct Model5 {
    #[serde(rename="_id", skip_serializing_if="Option::is_none")]
    pub id: Option<wither::bson::oid::ObjectId>,
}

fn main() {
    use wither::mongodb::options::WriteConcern;
    use wither::mongodb::options::Acknowledgment;
    let _m0 = Model0::default();
    let _m1 = Model1::default();
    let _m2 = Model2::default();
    let _m3 = Model3::default();
    let _m4 = Model4::default();
    let _m5 = Model5::default();
    assert_eq!(Model0::write_concern(), Some(WriteConcern::builder()
        .w(Some(Acknowledgment::Majority))
        .w_timeout(Some(std::time::Duration::from_secs(10)))
        .journal(Some(true))
        .build()));
    assert_eq!(Model1::write_concern(), Some(WriteConcern::builder()
        .w(Some(Acknowledgment::Nodes(3)))
        .w_timeout(Some(std::time::Duration::from_secs(0)))
        .journal(Some(false))
        .build()));
    assert_eq!(Model2::write_concern(), Some(WriteConcern::builder()
        .w(Some(Acknowledgment::Custom(String::from("custom"))))
        .build()));
    assert_eq!(Model3::write_concern(), Some(WriteConcern::builder()
        .w_timeout(Some(std::time::Duration::from_secs(999)))
        .build()));
    assert_eq!(Model4::write_concern(), Some(WriteConcern::builder()
        .journal(Some(true)).build()));
    assert_eq!(Model5::write_concern(), Some(WriteConcern::builder().build()));
}
