use phf::{phf_map, Map};

pub(super) static ROUTES: Map<&'static str, bool> = phf_map! {
  "/users.v1.UsersService/CreateSupplier" =>  false,
};
