import { Avatar, Flex, Typography } from "antd";
import type { ReactNode } from "react";

export type ProductBrandProps = {
  icon?: ReactNode;
  title: string;
};

/**
 * 产品品牌信息：产品图标与产品名称。
 */
export function ProductBrand({ icon, title }: ProductBrandProps) {
  return (
    <Flex align="center" gap="small">
      {icon && <Avatar shape="square" size={30} src={icon}/>}
      <Typography.Text ellipsis strong>
        {title}
      </Typography.Text>
    </Flex>
  );
}
