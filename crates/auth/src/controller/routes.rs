use phf::{phf_map, Map};

pub(super) static ROUTES: Map<&'static str, bool> = phf_map! {
  "/service.v1.ChatyService/UsersCreate" =>  false,
  "/service.v1.ChatyService/UsersLogin" =>  false,
};
