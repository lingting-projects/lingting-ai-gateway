import { Avatar, Flex, Typography } from "antd";
import type { ReactNode } from "react";

import { SidebarCollapsed, useSidebarLayout } from "@lri";

import "./ProductBrand.css";

export type ProductBrandProps = {
  icon?: ReactNode;
  title: string;
};

/**
 * 产品品牌信息：产品图标与产品名称。
 *
 * 布局参考 LRI 侧栏用户项：图标位于固定宽度容器中，侧栏收起时仅渲染图标。
 */
export function ProductBrand({ icon, title }: ProductBrandProps) {
  const { collapsed, sidebarDisplay } = useSidebarLayout();
  const showTitle = collapsed === SidebarCollapsed.Expanded || sidebarDisplay === "drawer";

  return (
    <Flex align="center" className="product-brand" gap={12} justify="center">
      <Flex align="center" className="product-brand-icon" justify="center">
        {icon && <Avatar shape="square" size={30} src={icon}/>}
      </Flex>
      {showTitle && (
        <Typography.Text className="product-brand-title" ellipsis strong>
          {title}
        </Typography.Text>
      )}
    </Flex>
  );
}
