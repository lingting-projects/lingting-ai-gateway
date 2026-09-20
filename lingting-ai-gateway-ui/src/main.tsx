import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import "@/index.css";
import { AppSidebarLayout, type User, UserStore } from "@lri";
import { menuRoutes, standaloneRoutes } from "@/router";
import { ProductBrand } from "@/components/product-brand";

const PRODUCT_ICON = "/icon.svg";
const PRODUCT_TITLE = "Lingting AI Gateway";

// 桌面端不需要登录，始终返回同一个用户信息
const fixedUser: User = {
  id: "fixed",
  nickname: PRODUCT_TITLE,
  organizations: [],
  permission: [],
  roles: [],
  tenantId: "fixed",
};

UserStore.initialize({
  getUser: async () => ({ type: "login", value: fixedUser }),
  logout: async () => ({ type: "login", value: fixedUser }),
});

const root = document.getElementById("root");

if (!root) {
  throw new Error("未找到应用挂载节点");
}

const App = () => {
  return (
    <AppSidebarLayout
      baseItems={[
        <ProductBrand
          icon={<img alt={PRODUCT_TITLE} src={PRODUCT_ICON}/>}
          key="product-brand"
          title={PRODUCT_TITLE}
        />,
      ]}
      logoutPosition="hidden"
      menuRoutes={menuRoutes}
      standaloneRoutes={standaloneRoutes}
      title={PRODUCT_TITLE}
      userPosition="hidden"
    />
  );
};

createRoot(root).render(
  <StrictMode>
    <App/>
  </StrictMode>,
);
