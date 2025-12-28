use phf::{phf_map, Map};

pub(super) static ROUTES: Map<&'static str, bool> = phf_map! {
  "/users.v1.UsersService/CreateSupplier" =>  false,
  "/users.v1.UsersService/Login" =>  false,
  "/users.v1.UsersService/GetCustomerProfile" =>  true,
  "/users.v1.UsersService/GetSupplierProfile" =>  true,
  "/users.v1.UsersService/GetSupplierDashboard" =>  true,

  "/products.v1.ProductsService/ProductData" => true,
  "/products.v1.ProductsService/ProductCreate" => true,
  "/products.v1.ProductsService/ProductList" => true,
  "/products.v1.ProductsService/BestSellingProducts" => false,
  "/products.v1.ProductsService/BigDiscountProducts" => false,
  "/products.v1.ProductsService/NewlyAddedProducts" => false,
  "/products.v1.ProductsService/HeroProducts" => false,
  "/products.v1.ProductsService/ProductsToLike" => false,
  "/products.v1.ProductsService/ProductDetails" => false,
  "/products.v1.ProductsService/CategoryNavbar" => false,
  "/products.v1.ProductsService/ProductsCategory" => false,
  "/products.v1.ProductsService/ProductsList" => true,

  "/orders.v1.OrdersService/OrdersList" => true,
  "/orders.v1.OrdersService/PaymentAddMethod" => true,
  "/orders.v1.OrdersService/PaymentRemoveMethod" => true,
  "/orders.v1.OrdersService/PaymentMakeDefault" => true,
  "/orders.v1.OrdersService/PaymentsList" => true,

  "/inventory.v1.InventoryService/InventoryList" => true,
  "/inventory.v1.InventoryService/InventoryGet" => true,
};
