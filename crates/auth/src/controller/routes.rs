use phf::{phf_map, Map};

pub(super) static ROUTES: Map<&'static str, bool> = phf_map! {
  "/service.v1.ChatyService/UsersCreate" =>  false,
  "/service.v1.ChatyService/UsersLogin" =>  false,
  "/service.v1.ChatyService/UsersEmailConfirmation" =>  false,
  "/service.v1.ChatyService/UsersForgotPassword" =>  false,
  "/service.v1.ChatyService/UsersResetPassword" =>  false,
  "/service.v1.ChatyService/GroupsCreate" =>  true,
  "/service.v1.ChatyService/GroupsList" =>  true,
  "/service.v1.ChatyService/SearchUsernames" =>  true,
};
